use anyhow::Result;
use rand::rngs::OsRng;
use secp256k1::Secp256k1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Secp256k1,
    BLS,
}

impl KeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Secp256k1 => "secp256k1",
            KeyType::BLS => "bls",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "secp256k1" => Ok(KeyType::Secp256k1),
            "bls" => Ok(KeyType::BLS),
            _ => anyhow::bail!("unknown key type: {}", s),
        }
    }
}

#[allow(dead_code)]
pub struct PrivateKey {
    pub key_type: KeyType,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl PrivateKey {
    pub fn generate(key_type: KeyType) -> Result<Self> {
        match key_type {
            KeyType::Secp256k1 => Self::generate_secp256k1(),
            KeyType::BLS => Self::generate_bls(),
        }
    }

    fn generate_secp256k1() -> Result<Self> {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        Ok(Self {
            key_type: KeyType::Secp256k1,
            private_key: secret_key.secret_bytes().to_vec(),
            public_key: public_key.serialize_uncompressed().to_vec(),
        })
    }

    fn generate_bls() -> Result<Self> {
        use blst::min_pk::{SecretKey as BlsSecretKey};

        // Generate random 32 bytes for private key
        let mut ikm = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut ikm);

        // Derive BLS secret key
        let sk = BlsSecretKey::key_gen(&ikm, &[])
            .map_err(|e| anyhow::anyhow!("BLS key generation failed: {:?}", e))?;

        // Get private key bytes (big-endian from blst)
        let sk_bytes = sk.to_bytes();

        // Reverse to little-endian for Filecoin storage
        let mut private_key = vec![0u8; 32];
        for i in 0..32 {
            private_key[i] = sk_bytes[31 - i];
        }

        // Derive public key (G1 compressed, 48 bytes)
        let pk = sk.sk_to_pk();
        let public_key = pk.to_bytes().to_vec();

        Ok(Self {
            key_type: KeyType::BLS,
            private_key,
            public_key,
        })
    }
}
