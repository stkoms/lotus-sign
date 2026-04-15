use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WalletKey {
    pub id: i64,
    pub address: String,
    pub key_type: String,
    pub encrypted_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WalletKey {
    pub fn new(address: String, key_type: String, encrypted_key: Vec<u8>) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            address,
            key_type,
            encrypted_key,
            created_at: now,
            updated_at: now,
        }
    }
}
