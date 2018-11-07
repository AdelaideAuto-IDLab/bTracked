mod serial_config;

use std::path::PathBuf;

use serialport;

fn default_true() -> bool { true }
fn default_1() -> u64 { 1 }
fn default_100() -> u64 { 1 }
fn default_some_30() -> Option<u64> { Some(30) }
fn default_log() -> String { "warn".into() }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum MeasurementSource {
    Serial {
        path: Option<PathBuf>,
        version: usize,
        #[serde(flatten)]
        options: serial_config::SerialOptions,
    },

    WebSocket { url: String },

    File {
        path: PathBuf,

        #[serde(default = "default_true")]
        repeat: bool,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct HttpConfig {
    pub endpoint: String,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub identity_cert: Option<PathBuf>,
    pub identity_cert_pass: Option<String>,
    #[serde(default)]
    pub root_certs: Vec<PathBuf>,
    #[serde(default = "default_some_30")]
    pub timeout_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum MeasurementDestination {
    /// Measurements will be serialized as a json array and sent as HTTP POST messages to `endpoint`
    Http {
        #[serde(flatten)]
        config: HttpConfig,

        /// The number of times to retry sending each request (default = 1)
        #[serde(default = "default_1")]
        retry_attempts: u64,

        /// The rate at which http requests are sent at (default = 100ms)
        #[serde(default = "default_100")]
        queue_rate_ms: u64,
    },

    /// Measurements will be serialized as json and sent as websocket text messages to `endpoint`
    WebSocket { endpoint: String },

    /// Measurements will be serialized as json and saved to a file separated by `\n` characters
    File {
        path: PathBuf,
        #[serde(default)]
        append: bool,
    },

    /// Measurements are written to stdout in json format
    Stdout,
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    /// Specifies the logging configuration using the `env_logger` syntax (default = 'warn')
    /// Example: `log = 'warn,basestation::init=debug'`
    #[serde(default = "default_log")]
    pub log: String,

    /// Specifies where measurements are obtained from
    pub source: MeasurementSource,

    /// Specifies where measurements are sent to
    pub destination: Vec<MeasurementDestination>,
}

pub fn default_config() -> AppConfig {
    AppConfig {
        log: "warn".into(),
        source: MeasurementSource::Serial {
            path: Some("COM1".into()),
            version: 1,
            options: serial_config::SerialOptions {
                baud_rate: 921600,
                data_bits: serialport::DataBits::Eight,
                stop_bits: serialport::StopBits::One,
                parity: serialport::Parity::None,
                flow_control: serialport::FlowControl::None,
            }
        },
        destination: vec![MeasurementDestination::Stdout],
    }
}
