#[macro_use] extern crate crossbeam_channel;
extern crate env_logger;
extern crate hex;
#[macro_use] extern crate log;
extern crate serialport;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate ws;
extern crate reqwest;

mod config;
mod dest;
mod error;
mod source;
mod util;

use std::{env, error::Error, fs, io, path::PathBuf, process, thread};
use crossbeam_channel as channel;

fn configure_logger(config: &str) {
    let mut builder = env_logger::Builder::new();
    builder.target(env_logger::Target::Stdout);

    match env::var("BASE_STATION_LOG") {
        Ok(var) => builder.parse_filters(&var),
        Err(_) => builder.parse_filters(config),
    };

    builder.init();
}

fn load_config() -> config::AppConfig {
    match fs::read("config.toml") {
        Ok(bytes) => match toml::from_slice(&bytes) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing config: {}", e);
                process::exit(-1);
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            let config = config::default_config();
            fs::write("config.toml", toml::to_vec(&config).unwrap()).unwrap();
            config
        },
        Err(e) => {
            eprintln!("Error loading config file: {}", e);
            process::exit(-1);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    match env::args().nth(1) {
        Some(ref arg) if arg == "--config" => {
            let config = config::default_config();
            println!("{}", toml::to_string_pretty(&config).unwrap());
            return Ok(());
        },
        _ => {}
    }

    let config = load_config();
    configure_logger(&config.log);
    info!("Started");

    let mut outputs = vec![];
    for d in config.destination {
        outputs.push(dest::new_destination(d)?);
    }

    let (packet_tx, packet_rx) = channel::bounded(255);

    match config.source {
        config::MeasurementSource::Serial { path, version, options } => {
            let path = match path {
                Some(p) => p,
                None => {
                    let ports = serialport::available_ports()?;
                    if ports.is_empty() {
                        return Err("No serial ports found".into());
                    }
                    else if ports.len() > 1 {
                        let names: Vec<_> = ports.into_iter().map(|x| x.port_name).collect();
                        let error_message = format!(
                            r#"Multiple serial ports found: [{}]. You must include a `path` entry in the [source] section of the config file (e.g. path = "{}")."#,
                            names.join(", "), names[0]
                        );
                        return Err(error_message.into());
                    }
                    info!("Found serial port: {}", ports[0].port_name);
                    PathBuf::from(&ports[0].port_name)
                }
            };
            thread::spawn(move || source::serial_source(path, version, options.into(), packet_tx));
        },

        config::MeasurementSource::File { path, repeat } => {
            thread::spawn(move || source::file_source(path, repeat, packet_tx));
        },

        _ => unimplemented!(),
    }


    for packet in packet_rx {
        for output in &mut outputs {
            if let Err(e) = output.new_measurement(&packet) {
                error!("Error sending measurement to destination: {}", e);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPacket {
    time: u64,
    #[serde(serialize_with="util::to_hex", deserialize_with="util::u8x6_from_hex")]
    mac: [u8; 6],
    rssi: i8,
    sequence: u8,
    session: u8,
}
