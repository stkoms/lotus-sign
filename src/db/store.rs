use anyhow::Result;
use rusqlite::{Connection, params};
use super::WalletKey;
use chrono::Utc;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS wallet_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL UNIQUE,
                key_type TEXT NOT NULL,
                encrypted_key BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_key(&self, key: &WalletKey) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO wallet_keys (address, key_type, encrypted_key, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                key.address,
                key.key_type,
                key.encrypted_key,
                key.created_at.to_rfc3339(),
                key.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_key(&self, address: &str) -> Result<Option<WalletKey>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, address, key_type, encrypted_key, created_at, updated_at
             FROM wallet_keys WHERE address = ?1"
        )?;

        let mut rows = stmt.query(params![address])?;

        if let Some(row) = rows.next()? {
            Ok(Some(WalletKey {
                id: row.get(0)?,
                address: row.get(1)?,
                key_type: row.get(2)?,
                encrypted_key: row.get(3)?,
                created_at: row.get::<_, String>(4)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(5)?.parse().unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_keys(&self) -> Result<Vec<WalletKey>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, address, key_type, encrypted_key, created_at, updated_at
             FROM wallet_keys ORDER BY id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(WalletKey {
                id: row.get(0)?,
                address: row.get(1)?,
                key_type: row.get(2)?,
                encrypted_key: row.get(3)?,
                created_at: row.get::<_, String>(4)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(5)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        let mut keys = Vec::new();
        for key in rows {
            keys.push(key?);
        }
        Ok(keys)
    }

    #[allow(dead_code)]
    pub fn has_key(&self, address: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM wallet_keys WHERE address = ?1",
            params![address],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    #[allow(dead_code)]
    pub fn delete_key(&self, address: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM wallet_keys WHERE address = ?1",
            params![address],
        )?;
        Ok(())
    }
}
