use super::LotusClient;
use crate::chain::{BigInt, Message, SignedMessage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct LotusApi {
    client: LotusClient,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MinerInfo {
    #[serde(rename = "Owner")]
    pub owner: String,
    #[serde(rename = "Worker")]
    pub worker: String,
    #[serde(rename = "ControlAddresses")]
    pub control_addresses: Option<Vec<String>>,
    #[serde(rename = "PeerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "SectorSize")]
    pub sector_size: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MarketBalance {
    #[serde(rename = "Escrow")]
    pub escrow: BigInt,
    #[serde(rename = "Locked")]
    pub locked: BigInt,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MsgLookup {
    #[serde(rename = "Height")]
    pub height: i64,
    #[serde(rename = "Receipt")]
    pub receipt: MsgReceipt,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MsgReceipt {
    #[serde(rename = "ExitCode")]
    pub exit_code: i64,
    #[serde(rename = "Return")]
    pub return_data: Option<String>,
    #[serde(rename = "GasUsed")]
    pub gas_used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cid {
    #[serde(rename = "/")]
    pub root: String,
}

impl LotusApi {
    pub fn new(url: &str, token: Option<String>) -> Self {
        Self {
            client: LotusClient::new(url, token),
        }
    }

    pub async fn wallet_balance(&self, address: &str) -> Result<BigInt> {
        self.client.call("WalletBalance", json!([address])).await
    }

    pub async fn mpool_get_nonce(&self, address: &str) -> Result<u64> {
        self.client.call("MpoolGetNonce", json!([address])).await
    }

    pub async fn mpool_push(&self, msg: &SignedMessage) -> Result<Cid> {
        self.client.call("MpoolPush", json!([msg])).await
    }

    pub async fn gas_estimate(&self, msg: &Message) -> Result<Message> {
        self.client
            .call("GasEstimateMessageGas", json!([msg, {}, null]))
            .await
    }

    pub async fn state_miner_info(&self, miner: &str) -> Result<MinerInfo> {
        self.client
            .call("StateMinerInfo", json!([miner, null]))
            .await
    }

    pub async fn state_miner_available_balance(&self, miner: &str) -> Result<BigInt> {
        self.client
            .call("StateMinerAvailableBalance", json!([miner, null]))
            .await
    }

    #[allow(dead_code)]
    pub async fn state_market_balance(&self, address: &str) -> Result<MarketBalance> {
        self.client
            .call("StateMarketBalance", json!([address, null]))
            .await
    }

    #[allow(dead_code)]
    pub async fn state_wait_msg(&self, cid: &Cid, confidence: u64) -> Result<MsgLookup> {
        self.client
            .call("StateWaitMsg", json!([cid, confidence]))
            .await
    }

    #[allow(dead_code)]
    pub async fn state_lookup_id(&self, address: &str) -> Result<String> {
        self.client
            .call("StateLookupID", json!([address, null]))
            .await
    }

    #[allow(dead_code)]
    pub async fn state_account_key(&self, address: &str) -> Result<String> {
        self.client
            .call("StateAccountKey", json!([address, null]))
            .await
    }

    #[allow(dead_code)]
    pub async fn chain_head(&self) -> Result<Value> {
        self.client.call("ChainHead", json!([])).await
    }
}
