use std::{num, time::SystemTime};
use hex;
use serde::{Serializer, Deserialize, Deserializer, de::Error};

/// A generic helper method for serialization of a byte array as a hex string
pub fn to_hex<T: AsRef<[u8]>, S: Serializer>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&hex::encode(&buffer.as_ref()))
}

/// A generic helper method for hex string deserialization of a byte array
#[allow(dead_code)]
pub fn from_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    String::deserialize(deserializer)
        .and_then(|string| hex::decode(&string).map_err(|err| Error::custom(err.to_string())))
}

pub fn u8x6_from_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 6], D::Error> {
    fn parse(string: &str) -> Result<[u8; 6], num::ParseIntError> {
        Ok([
            u8::from_str_radix(&string[0..2], 16)?,
            u8::from_str_radix(&string[2..4], 16)?,
            u8::from_str_radix(&string[4..6], 16)?,
            u8::from_str_radix(&string[6..8], 16)?,
            u8::from_str_radix(&string[8..10], 16)?,
            u8::from_str_radix(&string[10..12], 16)?,
        ])
    }

    String::deserialize(deserializer)
        .and_then(|string| parse(&string).map_err(|err| Error::custom(err.to_string())))
}

pub fn time_now_ms() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() * 1000 + duration.subsec_millis() as u64,
        Err(_) => 0,
    }
}
