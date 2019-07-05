mod http;

use std::{error::Error, fs, io::{self, Write}, path::PathBuf};

use serde_json;

use {config, BeaconPacket};


pub trait Destination {
    fn new_measurement(&mut self, measurement: &BeaconPacket) -> Result<(), Box<dyn Error>>;
}

pub fn new_destination(
    config: config::MeasurementDestination
) -> Result<Box<dyn Destination>, Box<dyn Error>> {
    match config {
        config::MeasurementDestination::Http { config, retry_attempts, queue_rate_ms } => {
            let dest = http::HttpDestination::new(config, retry_attempts, queue_rate_ms)?;
            Ok(Box::new(dest) as Box<dyn Destination>)
        },
        config::MeasurementDestination::WebSocket { endpoint: _endpoint } => {
            unimplemented!()
        },
        config::MeasurementDestination::File { path, append } => {
            Ok(Box::new(FileDestination::new(path, append)?) as Box<dyn Destination>)
        },
        config::MeasurementDestination::Stdout => {
            Ok(Box::new(StdoutDestination) as Box<dyn Destination>)
        },
    }
}

struct FileDestination {
    file: fs::File,
    buffer: io::Cursor<Vec<u8>>,
}

impl FileDestination {
    fn new(path: PathBuf, append: bool) -> io::Result<FileDestination> {
        let file = fs::OpenOptions::new()
            .write(true)
            .append(append)
            .truncate(!append)
            .create(true)
            .open(path)?;
        Ok(FileDestination { file, buffer: io::Cursor::new(vec![]) })
    }
}

impl Destination for FileDestination {
    fn new_measurement(&mut self, measurement: &BeaconPacket) -> Result<(), Box<dyn Error>> {
        serde_json::to_writer(&mut self.buffer, measurement)?;
        self.buffer.write(&[b'\n'])?;

        self.file.write_all(&self.buffer.get_ref()[..self.buffer.position() as usize])?;
        self.buffer.set_position(0);

        Ok(())
    }
}

struct StdoutDestination;

impl Destination for StdoutDestination {
    fn new_measurement(&mut self, measurement: &BeaconPacket) -> Result<(), Box<dyn Error>> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer(&mut handle, measurement)?;
        handle.write(&[b'\n'])?;
        Ok(())
    }
}
