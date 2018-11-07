use std::time::Duration;
use serialport::{self, SerialPortSettings};

#[derive(Serialize, Deserialize)]
#[serde(remote = "serialport::DataBits")]
enum DataBitsDef {
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", remote = "serialport::Parity")]
enum ParityDef {
    None,
    Odd,
    Even,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", remote = "serialport::FlowControl")]
enum FlowControlDef {
    None,
    Software,
    Hardware,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "serialport::StopBits")]
pub enum StopBitsDef {
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
}

#[derive(Serialize, Deserialize)]
pub struct SerialOptions {
    pub baud_rate: u32,

    #[serde(with = "DataBitsDef")]
    pub data_bits: serialport::DataBits,

    #[serde(with = "FlowControlDef")]
    pub flow_control: serialport::FlowControl,

    #[serde(with = "ParityDef")]
    pub parity: serialport::Parity,

    #[serde(with = "StopBitsDef")]
    pub stop_bits: serialport::StopBits,
}

impl From<SerialOptions> for SerialPortSettings {
    fn from(opt: SerialOptions) -> SerialPortSettings {
        SerialPortSettings {
            baud_rate: opt.baud_rate,
            data_bits: opt.data_bits,
            stop_bits: opt.stop_bits,
            parity: opt.parity,
            flow_control: opt.flow_control,
            timeout: Duration::from_secs(60),
        }
    }
}
