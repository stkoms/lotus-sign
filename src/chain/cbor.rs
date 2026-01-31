//! Filecoin CBOR 序列化模块
//!
//! 本模块实现 Filecoin 特定的 CBOR 编码，用于消息序列化和 CID 计算。
//! Filecoin 使用自定义 CBOR 格式，消息被编码为固定的 10 元素数组。

use anyhow::Result;
use serde::Serialize;
use super::{Message, Address, BigInt};

/// 通用 CBOR 序列化（使用 ciborium 库）
/// 注意：此函数不用于 Filecoin 消息，请使用 serialize_message()
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf)?;
    Ok(buf)
}

/// 将消息序列化为 Filecoin CBOR 格式（10 元素数组）
///
/// Filecoin 消息格式：
/// [Version, To, From, Nonce, Value, GasLimit, GasFeeCap, GasPremium, Method, Params]
///
/// 这与标准 CBOR map 序列化不同 - Filecoin 要求固定的数组格式
pub fn serialize_message(msg: &Message) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    // 数组头：0x8a = 10 个元素
    buf.push(0x8a);

    // 1. 版本号 (uint)
    write_cbor_uint(&mut buf, msg.version);

    // 2. 接收地址 (address bytes)
    write_address(&mut buf, &msg.to);

    // 3. 发送地址 (address bytes)
    write_address(&mut buf, &msg.from);

    // 4. 序号 (uint)
    write_cbor_uint(&mut buf, msg.nonce);

    // 5. 金额 (BigInt bytes)
    write_bigint(&mut buf, &msg.value);

    // 6. Gas限制 (int64)
    write_cbor_int(&mut buf, msg.gas_limit);

    // 7. Gas费用上限 (BigInt bytes)
    write_bigint(&mut buf, &msg.gas_fee_cap);

    // 8. Gas优先费 (BigInt bytes)
    write_bigint(&mut buf, &msg.gas_premium);

    // 9. 方法号 (uint)
    write_cbor_uint(&mut buf, msg.method);

    // 10. 参数 (byte string)
    write_cbor_bytes(&mut buf, &msg.params);

    Ok(buf)
}

/// 返回 CID 原始字节（用于签名）
///
/// CID 格式：[version(1), codec(varint), multihash]
/// - version: 0x01 (CIDv1)
/// - codec: 0x71 (dag-cbor, varint 编码为 0xa0 0xe4 0x02)
/// - multihash: [hash_type(0x20=blake2b-256), length(0x20=32), hash_bytes]
pub fn compute_cid_bytes(data: &[u8]) -> Vec<u8> {
    use blake2b_simd::Params;
    let hash = Params::new()
        .hash_length(32)
        .hash(data);

    // CIDv1，使用 blake2b-256 和 dag-cbor 编解码器
    let mut cid = vec![0x01, 0x71, 0xa0, 0xe4, 0x02, 0x20];
    cid.extend_from_slice(hash.as_bytes());
    cid
}

/// 返回 CID 的 multibase 编码字符串（用于显示）
#[allow(dead_code)]
pub fn compute_cid(data: &[u8]) -> String {
    multibase_encode(&compute_cid_bytes(data))
}

#[allow(dead_code)]
fn multibase_encode(data: &[u8]) -> String {
    // Base32 小写编码，带 'b' 前缀（multibase 格式）
    format!("b{}", base32_encode(data))
}

#[allow(dead_code)]
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut result = String::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for &byte in data {
        buffer = (buffer << 8) | byte as u32;
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

// CBOR 编码辅助函数
// CBOR 使用高 3 位表示主类型：0=无符号整数, 1=负整数, 2=字节串, 3=文本, 4=数组, 5=映射

/// 写入无符号整数（CBOR 主类型 0）
fn write_cbor_uint(buf: &mut Vec<u8>, val: u64) {
    if val < 24 {
        buf.push(val as u8);
    } else if val < 256 {
        buf.push(0x18);
        buf.push(val as u8);
    } else if val < 65536 {
        buf.push(0x19);
        buf.extend_from_slice(&(val as u16).to_be_bytes());
    } else if val < 4294967296 {
        buf.push(0x1a);
        buf.extend_from_slice(&(val as u32).to_be_bytes());
    } else {
        buf.push(0x1b);
        buf.extend_from_slice(&val.to_be_bytes());
    }
}

/// 写入有符号整数（正数用主类型 0，负数用主类型 1）
fn write_cbor_int(buf: &mut Vec<u8>, val: i64) {
    if val >= 0 {
        write_cbor_uint(buf, val as u64);
    } else {
        let neg = (-val - 1) as u64;
        if neg < 24 {
            buf.push(0x20 + neg as u8);
        } else if neg < 256 {
            buf.push(0x38);
            buf.push(neg as u8);
        } else if neg < 65536 {
            buf.push(0x39);
            buf.extend_from_slice(&(neg as u16).to_be_bytes());
        } else if neg < 4294967296 {
            buf.push(0x3a);
            buf.extend_from_slice(&(neg as u32).to_be_bytes());
        } else {
            buf.push(0x3b);
            buf.extend_from_slice(&neg.to_be_bytes());
        }
    }
}

/// 写入字节串（CBOR 主类型 2）
fn write_cbor_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() as u64;
    if len < 24 {
        buf.push(0x40 + len as u8);
    } else if len < 256 {
        buf.push(0x58);
        buf.push(len as u8);
    } else if len < 65536 {
        buf.push(0x59);
        buf.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        buf.push(0x5a);
        buf.extend_from_slice(&(len as u32).to_be_bytes());
    }
    buf.extend_from_slice(data);
}

/// 写入 Filecoin 地址为 CBOR 字节串
/// 格式：[协议字节 || 载荷]
fn write_address(buf: &mut Vec<u8>, addr: &Address) {
    // 地址序列化为字节：[协议字节 || 载荷]
    let protocol_byte = match addr.protocol {
        super::address::Protocol::ID => 0u8,
        super::address::Protocol::Secp256k1 => 1u8,
        super::address::Protocol::Actor => 2u8,
        super::address::Protocol::BLS => 3u8,
    };
    let mut addr_bytes = vec![protocol_byte];
    addr_bytes.extend_from_slice(&addr.payload);
    write_cbor_bytes(buf, &addr_bytes);
}

/// 写入 Filecoin BigInt 为 CBOR 字节串
/// 格式：[符号字节 || 大端字节]，零值为空
fn write_bigint(buf: &mut Vec<u8>, val: &BigInt) {
    // Filecoin BigInt 序列化为带符号前缀的字节
    use num_bigint::Sign;
    let (sign, bytes) = val.0.to_bytes_be();

    if bytes.is_empty() || (bytes.len() == 1 && bytes[0] == 0) {
        // 零值：空字节串
        write_cbor_bytes(buf, &[]);
    } else {
        let sign_byte: u8 = match sign {
            Sign::Plus | Sign::NoSign => 0x00,
            Sign::Minus => 0x01,
        };
        let mut bigint_bytes = vec![sign_byte];
        bigint_bytes.extend_from_slice(&bytes);
        write_cbor_bytes(buf, &bigint_bytes);
    }
}
