#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPacket {
    pub time: u64,
    pub mac: String,
    pub rssi: i8,
    pub sequence: u8,
    pub session: u8,
}
