use super::{Address, BigInt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "Version")]
    pub version: u64,
    #[serde(rename = "To")]
    pub to: Address,
    #[serde(rename = "From")]
    pub from: Address,
    #[serde(rename = "Nonce")]
    pub nonce: u64,
    #[serde(rename = "Value")]
    pub value: BigInt,
    #[serde(rename = "GasLimit")]
    pub gas_limit: i64,
    #[serde(rename = "GasFeeCap")]
    pub gas_fee_cap: BigInt,
    #[serde(rename = "GasPremium")]
    pub gas_premium: BigInt,
    #[serde(rename = "Method")]
    pub method: u64,
    #[serde(rename = "Params", with = "base64_bytes")]
    pub params: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    #[serde(rename = "Type")]
    pub sig_type: u8,
    #[serde(rename = "Data", with = "base64_bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    #[serde(rename = "Message")]
    pub message: Message,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        s.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(d)?;
        STANDARD.decode(&s).map_err(serde::de::Error::custom)
    }
}
