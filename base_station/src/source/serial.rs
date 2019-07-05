use std::{path::PathBuf, thread, time::Duration};

use crossbeam_channel as channel;
use serialport::{self, SerialPort};

use {util, BeaconPacket};


pub fn serial_source(
    path: PathBuf,
    version: usize,
    settings: serialport::SerialPortSettings,
    mut output: channel::Sender<BeaconPacket>
) {
    let mut reader = SerialReader::new(version);
    loop {
        let mut port = match serialport::open_with_settings(&path, &settings) {
            Ok(port) => port,
            Err(e) => {
                error!("Error opening serial port: {}", e);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };
        reader.parse_packets(&mut port, &mut output);
    }
}

struct SerialReader {
    decoder: SerialDecoder,
    parser: BeaconPacketParser,
    buf: Vec<u8>
}

impl SerialReader {
    fn new(version: usize) -> SerialReader {
        let parser = match version {
            1 => BeaconPacketParser {
                min_length: 0x18,
                header: vec![0x0c, 0xff, 0x00, 0x00],
                header_start: 0x0B,
            },
            2 => BeaconPacketParser {
                min_length: 0x29,
                header: vec![0x0c, 0xff, 0x00, 0x00],
                header_start: 0x1D,
            },
            _ => panic!("Unsupported serial reader version"),
        };

        SerialReader {
            decoder: SerialDecoder::new(version),
            parser,
            buf: vec![0; 256],
        }
    }

    fn parse_packets(
        &mut self,
        port: &mut Box<dyn SerialPort>,
        output: &mut channel::Sender<BeaconPacket>
    ) {
        loop {
            let len = match port.read(&mut self.buf[..]) {
                Ok(0) => {
                    warn!("0 bytes read from serial port -- device might be closed");
                    continue;
                },
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from serial port: {}", e);
                    return;
                }
            };

            for &byte in &self.buf[..len] {
                let parser = &mut self.parser;
                if let Some(packet) = self.decoder.next(byte).and_then(|p| parser.parse_packet(p)) {
                    select! {
                        send(output, packet) -> _ => {},
                        default => {
                            warn!("Packet receiver not ready, dropping packet");
                        }
                    }
                }
            }
        }
    }
}

enum SerialDecoder {
    V1(SerialDecoderV1),
    V2(SerialDecoderV2)
}

impl SerialDecoder {
    fn new(version: usize) -> SerialDecoder {
        match version {
            1 => SerialDecoder::V1(SerialDecoderV1::new()),
            2 => SerialDecoder::V2(SerialDecoderV2::new()),
            _ => panic!("Unsupported serial decoder version"),
        }
    }

    fn next(&mut self, byte: u8) -> Option<&[u8]> {
        match self {
            SerialDecoder::V1(decoder) => decoder.next(byte),
            SerialDecoder::V2(decoder) => decoder.next(byte),
        }
    }
}

struct SerialDecoderV1 {
    buffer: Vec<u8>,
    next_char_escaped: bool,
    complete: bool,
}

impl SerialDecoderV1 {
    fn new() -> SerialDecoderV1 {
        SerialDecoderV1 { buffer: vec![], next_char_escaped: false, complete: false, }
    }

    fn next(&mut self, byte: u8) -> Option<&[u8]> {
        if self.complete {
            self.buffer.clear();
            self.next_char_escaped = false;
            self.complete = false;
        }

        if byte == 0xFF {
            self.complete = true;
            match self.buffer.len() > 4 {
                true => return Some(&self.buffer),
                false => return None,
            }
        }

        if self.next_char_escaped {
            self.buffer.push(byte + 1);
            self.next_char_escaped = false;
        }
        else if byte == 0xFE {
            self.next_char_escaped = true;
        }
        else {
            self.buffer.push(byte);
        }

        None
    }
}

const SLIP_START: u8 = 0xAB;
const SLIP_END: u8 = 0xBC;
const SLIP_ESC: u8 = 0xCD;

struct SerialDecoderV2 {
    buffer: Vec<u8>,
    next_char_escaped: bool,
    complete: bool,
}

impl SerialDecoderV2 {
    fn new() -> SerialDecoderV2 {
        SerialDecoderV2 { buffer: vec![], next_char_escaped: false, complete: false, }
    }

    fn next(&mut self, byte: u8) -> Option<&[u8]> {
        if byte == SLIP_START {
            self.complete = false;
            self.buffer.clear();
            return None;
        }
        else if byte == SLIP_END {
            if !self.complete && self.buffer.len() >= 4 {
                self.complete = true;
                return Some(&self.buffer);
            }
            self.complete = true;
            return None;
        }

        if self.complete {
            // After receving a slip end we haven't yet received a slip start message
            return None;
        }

        if byte == SLIP_ESC {
            self.next_char_escaped = true;
        }
        else if self.next_char_escaped {
            self.buffer.push(byte + 1);
            self.next_char_escaped = false;
        }
        else {
            self.buffer.push(byte);
        }

        None
    }
}

struct BeaconPacketParser {
    min_length: usize,
    header: Vec<u8>,
    header_start: usize,
}

impl BeaconPacketParser {
    fn parse_packet(&self, packet: &[u8]) -> Option<BeaconPacket> {
        if packet.len() < self.min_length {
            return None;
        }

        let header_end = self.header_start + self.header.len();
        if &packet[self.header_start..header_end] != &self.header[..] {
            return None;
        }

        let body = &packet[header_end..];
        Some(BeaconPacket {
            time: util::time_now_ms(),
            mac: [body[5], body[4], body[3], body[2], body[1], body[0]],
            rssi: body[6] as i8,
            sequence: body[7],
            session: body[8],
        })
    }
}

