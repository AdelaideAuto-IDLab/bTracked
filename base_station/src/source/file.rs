use std::{error::Error, fs, io::{self, BufRead, Seek}, path::{PathBuf, Path}, thread, time::Duration};

use crossbeam_channel as channel;
use serde_json;

use BeaconPacket;


pub fn file_source(path: PathBuf, repeat: bool, mut output: channel::Sender<BeaconPacket>) {
    loop {
        let mut loader = match FileLoader::new(&path) {
            Ok(loader) => loader,
            Err(e) => {
                error!("Error opening file: {}", e);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        if let Err(e) = read_packets(&mut loader, repeat, &mut output) {
            error!("Error reading from file: {}", e);
            thread::sleep(Duration::from_secs(5));
        }
    }
}

fn read_packets(
    loader: &mut FileLoader,
    repeat: bool,
    output: &mut channel::Sender<BeaconPacket>
) -> Result<(), Box<dyn Error>> {
    loop {
        match loader.next()? {
            Some(packet) => {
                select! {
                    send(output, packet) -> _ => {},
                    default => {
                        warn!("Packet receiver not ready, dropping packet");
                    }
                }
            },
            None => {
                if repeat {
                    loader.reader.seek(io::SeekFrom::Start(0))?;
                }
                else {
                    thread::sleep(Duration::from_millis(1000));
                }
            }
        }
    }
}

struct FileLoader {
    reader: io::BufReader<fs::File>,
    buffer: String,
    prev_time: u64,
}

impl FileLoader {
    fn new(path: &Path) -> Result<FileLoader, Box<dyn Error>> {
        Ok(FileLoader {
            reader: io::BufReader::new(fs::File::open(path)?),
            buffer: String::new(),
            prev_time: 0,
        })
    }

    fn next(&mut self) -> Result<Option<BeaconPacket>, Box<dyn Error>> {
        let num_bytes = self.reader.read_line(&mut self.buffer)?;
        if num_bytes == 0 {
            // End of file
            return Ok(None);
        }

        let packet: BeaconPacket = serde_json::from_str(&self.buffer[..num_bytes])?;
        self.buffer.clear();

        if self.prev_time != 0 && self.prev_time < packet.time {
            thread::sleep(Duration::from_millis(packet.time - self.prev_time));
        }
        self.prev_time = packet.time;

        Ok(Some(packet))
    }
}