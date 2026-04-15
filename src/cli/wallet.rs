use crate::config::Config;
use crate::db::{Store, WalletKey};
use crate::wallet::{KeyType, PrivateKey};
use crate::crypto;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct WalletCmd {
    #[command(subcommand)]
    pub command: WalletSubCmd,
}

#[derive(Subcommand)]
pub enum WalletSubCmd {
    New {
        #[arg(short, long, default_value = "secp256k1")]
        key_type: String,
    },
    List,
    Balance {
        address: String,
    },
    Export {
        address: String,
    },
    Import {
        private_key: String,
        #[arg(short, long, default_value = "hex")]
        format: String,
    },
    Importnew {
        private_key: String,
    },
}

pub async fn run(cmd: WalletCmd, cfg: &Config, store: &Store) -> Result<()> {
    match cmd.command {
        WalletSubCmd::New { key_type } => {
            use crate::chain::Address;
            let kt = KeyType::from_str(&key_type)?;
            let key = PrivateKey::generate(kt)?;
            let password = cfg.get_password();
            let enc_key = crypto::derive_key(&password);
            let encrypted = crypto::encrypt(&key.private_key, &enc_key)?;

            let addr = match kt {
                KeyType::Secp256k1 => {
                    Address::new_secp256k1(&key.public_key)?.to_string()
                }
                KeyType::BLS => {
                    Address::new_bls(&key.public_key)?.to_string()
                }
            };
            let wk = WalletKey::new(addr.clone(), kt.as_str().to_string(), encrypted);
            store.insert_key(&wk)?;

            println!("Created: {}", addr);
        }
        WalletSubCmd::List => {
            use crate::rpc::LotusApi;
            use crate::chain::format_fil;
            let api = LotusApi::new(&cfg.lotus.host, cfg.lotus.token.clone());
            let keys = store.list_keys()?;

            println!("{:<50} {:<12} {:<20} {:<10}", "Address", "Type", "Balance", "Nonce");
            println!("{}", "-".repeat(95));

            for k in keys {
                let balance = api.wallet_balance(&k.address).await.unwrap_or_default();
                let nonce = api.mpool_get_nonce(&k.address).await.unwrap_or(0);
                let bal_str = format_fil(&balance.0);
                println!("{:<50} {:<12} {:<20} {:<10}", k.address, k.key_type, bal_str, nonce);
            }
        }
        WalletSubCmd::Balance { address } => {
            use crate::rpc::LotusApi;
            let api = LotusApi::new(&cfg.lotus.host, cfg.lotus.token.clone());
            let bal = api.wallet_balance(&address).await?;
            println!("{}: {} attoFIL", address, bal);
        }
        WalletSubCmd::Export { address } => {
            let key = store.get_key(&address)?
                .ok_or_else(|| anyhow::anyhow!("key not found"))?;
            let password = cfg.get_password();
            let enc_key = crypto::derive_key(&password);
            let pk = crypto::decrypt(&key.encrypted_key, &enc_key)?;
            println!("{}", hex::encode(&pk));
        }
        WalletSubCmd::Import { private_key, format } => {
            use crate::chain::Address;
            use base64::Engine;

            // Auto-detect format: hex-encoded JSON starts with "7b22" (which is `{"`)
            let (pk, key_type) = if private_key.starts_with("7b22") {
                // Hex-encoded JSON format
                let json_bytes = hex::decode(&private_key)?;
                let json_str = String::from_utf8(json_bytes)?;
                let v: serde_json::Value = serde_json::from_str(&json_str)?;
                let key_type = v["Type"].as_str().unwrap_or("secp256k1").to_string();
                let key_str = v["PrivateKey"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid json format"))?;
                let pk = base64::engine::general_purpose::STANDARD.decode(key_str)?;
                (pk, key_type)
            } else if format == "json" {
                // Plain JSON format
                let v: serde_json::Value = serde_json::from_str(&private_key)?;
                let key_type = v["Type"].as_str().unwrap_or("secp256k1").to_string();
                let key_str = v["PrivateKey"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid json format"))?;
                let pk = base64::engine::general_purpose::STANDARD.decode(key_str)?;
                (pk, key_type)
            } else {
                // Raw hex format
                (hex::decode(&private_key)?, "secp256k1".to_string())
            };

            // Derive public key and address based on key type
            let addr = if key_type == "bls" {
                use blst::min_pk::SecretKey as BlsSecretKey;
                let mut key_be = [0u8; 32];
                for i in 0..32 { key_be[i] = pk[31 - i]; }
                let sk = BlsSecretKey::from_bytes(&key_be)
                    .map_err(|e| anyhow::anyhow!("invalid BLS key: {:?}", e))?;
                let pubkey = sk.sk_to_pk().to_bytes().to_vec();
                Address::new_bls(&pubkey)?.to_string()
            } else {
                // secp256k1
                let secp = secp256k1::Secp256k1::new();
                let secret = secp256k1::SecretKey::from_slice(&pk)?;
                let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret);
                Address::new_secp256k1(&pubkey.serialize_uncompressed())?.to_string()
            };

            let password = cfg.get_password();
            let enc_key = crypto::derive_key(&password);
            let encrypted = crypto::encrypt(&pk, &enc_key)?;
            let wk = WalletKey::new(addr.clone(), key_type, encrypted);
            store.insert_key(&wk)?;
            println!("Imported: {}", addr);
        }
        WalletSubCmd::Importnew { private_key } => {
            use crate::chain::Address;
            let pk = hex::decode(&private_key)?;

            // Derive public key and address (secp256k1)
            let secp = secp256k1::Secp256k1::new();
            let secret = secp256k1::SecretKey::from_slice(&pk)?;
            let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret);
            let addr = Address::new_secp256k1(&pubkey.serialize_uncompressed())?.to_string();

            let password = cfg.get_password();
            let enc_key = crypto::derive_key(&password);
            let encrypted = crypto::encrypt(&pk, &enc_key)?;
            let wk = WalletKey::new(addr.clone(), "secp256k1".to_string(), encrypted);
            store.insert_key(&wk)?;
            println!("{}", addr);
        }
    }
    Ok(())
}
