//! Filecoin 交易签名模块
//!
//! 支持 Filecoin 使用的 secp256k1 和 BLS 两种签名方案。

use crate::chain::{cbor, Message, Signature};
use crate::crypto;
use crate::db::Store;
use anyhow::Result;
use blake2b_simd::Params;
use secp256k1::{Message as SecpMsg, Secp256k1, SecretKey};

// Filecoin BLS 域分离标签，用于 BLS 签名
// 此标签确保签名具有域分离性，不能跨协议重用
const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

/// 钱包结构体，管理私钥并签名 Filecoin 消息
pub struct Wallet<'a> {
    store: &'a Store,       // 数据库存储（加密的密钥）
    enc_key: [u8; 32],      // 从密码派生的加密密钥
}

impl<'a> Wallet<'a> {
    /// 创建新的钱包实例，使用密码派生的加密密钥
    pub fn new(store: &'a Store, password: &str) -> Self {
        Self {
            store,
            enc_key: crypto::derive_key(password),
        }
    }

    /// 使用 'from' 地址关联的私钥签名 Filecoin 消息
    /// 返回带有相应类型的签名（1=secp256k1, 2=BLS）
    pub fn sign(&self, msg: &Message, from: &str) -> Result<Signature> {
        let key = self.store.get_key(from)?
            .ok_or_else(|| anyhow::anyhow!("key not found: {}", from))?;

        let private_key = crypto::decrypt(&key.encrypted_key, &self.enc_key)?;
        let cid_bytes = self.message_cid_bytes(msg)?;

        match key.key_type.as_str() {
            "secp256k1" => self.sign_secp256k1(&private_key, &cid_bytes),
            "bls" => self.sign_bls(&private_key, &cid_bytes),
            _ => Err(anyhow::anyhow!("unsupported key type")),
        }
    }

    #[allow(dead_code)]
    pub fn has_key(&self, address: &str) -> Result<bool> {
        self.store.has_key(address)
    }

    /// 使用 secp256k1 ECDSA 签名（带恢复 ID）
    /// Filecoin 使用 CID 字节的 blake2b-256 哈希作为消息摘要
    fn sign_secp256k1(&self, key: &[u8], data: &[u8]) -> Result<Signature> {
        let secp = Secp256k1::new();
        let secret = SecretKey::from_slice(key)?;

        let hash = blake2b_hash(data, 32);
        let msg = SecpMsg::from_digest_slice(&hash)?;
        let sig = secp.sign_ecdsa_recoverable(&msg, &secret);
        let (rec_id, sig_bytes) = sig.serialize_compact();

        let mut data = sig_bytes.to_vec();
        data.push(rec_id.to_i32() as u8);

        Ok(Signature { sig_type: 1, data })
    }

    /// 使用 BLS12-381 签名方案签名
    /// 注意：Filecoin 使用小端存储 BLS 密钥，blst 库使用大端
    fn sign_bls(&self, key: &[u8], data: &[u8]) -> Result<Signature> {
        use blst::min_pk::{SecretKey as BlsSecretKey};

        if key.len() != 32 {
            return Err(anyhow::anyhow!("invalid BLS private key length"));
        }

        // Filecoin uses little-endian, blst uses big-endian, so reverse bytes
        let mut key_reversed = [0u8; 32];
        for i in 0..32 {
            key_reversed[i] = key[31 - i];
        }

        let sk = BlsSecretKey::from_bytes(&key_reversed)
            .map_err(|e| anyhow::anyhow!("invalid BLS key: {:?}", e))?;

        let sig = sk.sign(data, BLS_DST, &[]);
        let sig_bytes = sig.to_bytes();

        Ok(Signature { sig_type: 2, data: sig_bytes.to_vec() })
    }

    /// 计算消息的 CID 字节（用于签名）
    /// 步骤：CBOR 序列化消息 -> 计算 CID 字节
    fn message_cid_bytes(&self, msg: &Message) -> Result<Vec<u8>> {
        let cbor_data = cbor::serialize_message(msg)?;
        Ok(cbor::compute_cid_bytes(&cbor_data))
    }
}

/// 计算指定长度的 blake2b 哈希
fn blake2b_hash(data: &[u8], size: usize) -> Vec<u8> {
    Params::new()
        .hash_length(size)
        .hash(data)
        .as_bytes()
        .to_vec()
}
