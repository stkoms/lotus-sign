//! Filecoin 地址处理模块
//!
//! Filecoin 地址有 4 种协议类型：
//! - f0: ID 地址（Actor ID）
//! - f1: secp256k1 地址（未压缩公钥的 20 字节 blake2b 哈希）
//! - f2: Actor 地址
//! - f3: BLS 地址（48 字节公钥）

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Filecoin 地址协议类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    ID = 0,         // f0 - Actor ID
    Secp256k1 = 1,  // f1 - secp256k1 公钥哈希
    Actor = 2,      // f2 - Actor 地址
    BLS = 3,        // f3 - BLS 公钥
}

/// Filecoin 地址结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub protocol: Protocol,  // 地址类型 (f0-f3)
    pub payload: Vec<u8>,    // 地址特定数据
}

impl Address {
    /// 从未压缩公钥（65 字节）创建 secp256k1 地址
    /// 载荷是公钥的 20 字节 blake2b 哈希
    pub fn new_secp256k1(pubkey: &[u8]) -> Result<Self> {
        let hash = blake2b_hash(pubkey, 20);
        Ok(Self {
            protocol: Protocol::Secp256k1,
            payload: hash,
        })
    }

    /// 从公钥（48 字节）创建 BLS 地址
    /// 载荷是原始公钥
    pub fn new_bls(pubkey: &[u8]) -> Result<Self> {
        Ok(Self {
            protocol: Protocol::BLS,
            payload: pubkey.to_vec(),
        })
    }

    /// 从字符串格式解析地址（如 "f1abc..." 或 "t1abc..."）
    /// 格式：[网络][协议][base32_载荷_带校验和]
    pub fn from_string(s: &str) -> Result<Self> {
        if s.len() < 3 {
            return Err(anyhow!("invalid address"));
        }

        let network = &s[0..1];
        if network != "f" && network != "t" {
            return Err(anyhow!("invalid network prefix"));
        }

        let protocol = match &s[1..2] {
            "0" => Protocol::ID,
            "1" => Protocol::Secp256k1,
            "2" => Protocol::Actor,
            "3" => Protocol::BLS,
            _ => return Err(anyhow!("invalid protocol")),
        };

        let payload = base32_decode(&s[2..])?;
        Ok(Self { protocol, payload })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol_byte = match self.protocol {
            Protocol::ID => 0u8,
            Protocol::Secp256k1 => 1u8,
            Protocol::Actor => 2u8,
            Protocol::BLS => 3u8,
        };
        let prefix = format!("f{}", protocol_byte);
        write!(f, "{}{}", prefix, base32_encode_with_checksum(protocol_byte, &self.payload))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Address::from_string(&s).map_err(serde::de::Error::custom)
    }
}

fn blake2b_hash(data: &[u8], size: usize) -> Vec<u8> {
    use blake2b_simd::Params;
    Params::new()
        .hash_length(size)
        .hash(data)
        .as_bytes()
        .to_vec()
}

/// 使用 base32 编码载荷并附加校验和
/// 校验和 = blake2b-32([协议字节 || 载荷])
fn base32_encode_with_checksum(protocol: u8, payload: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

    // 校验和是 [协议 || 载荷] 的 blake2b-32
    let mut checksum_input = vec![protocol];
    checksum_input.extend_from_slice(payload);
    let checksum = blake2b_hash(&checksum_input, 4);

    // 编码 载荷 + 校验和
    let mut data = payload.to_vec();
    data.extend_from_slice(&checksum);

    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for &byte in &data {
        buffer = (buffer << 8) | byte as u64;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        result.push(ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    result
}

#[allow(dead_code)]
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let checksum = blake2b_hash(data, 4);
    let mut payload = data.to_vec();
    payload.extend_from_slice(&checksum);

    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for &byte in &payload {
        buffer = (buffer << 8) | byte as u64;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        result.push(ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    result
}

/// 解码 base32 字符串并去除校验和（最后 4 字节）
fn base32_decode(s: &str) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for c in s.chars() {
        let val = match c {
            'a'..='z' => c as u64 - 'a' as u64,
            '2'..='7' => c as u64 - '2' as u64 + 26,
            _ => return Err(anyhow!("invalid base32 char")),
        };
        buffer = (buffer << 5) | val;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }
    if result.len() >= 4 {
        result.truncate(result.len() - 4);
    }
    Ok(result)
}
