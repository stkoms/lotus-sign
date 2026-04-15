use crate::chain::{
    cbor, Address, BigInt, Message, SignedMessage,
    METHOD_WITHDRAW_BALANCE, METHOD_CHANGE_OWNER,
    METHOD_CHANGE_WORKER, METHOD_CONFIRM_CHANGE_WORKER,
    METHOD_MARKET_WITHDRAW, STORAGE_MARKET_ACTOR,
};
use crate::config::Config;
use crate::db::Store;
use crate::rpc::{LotusApi, Cid};
use crate::wallet::Wallet;
use anyhow::Result;

pub struct Executor<'a> {
    pub api: LotusApi,
    pub wallet: Wallet<'a>,
}

impl<'a> Executor<'a> {
    pub fn new(cfg: &Config, store: &'a Store) -> Self {
        let api = LotusApi::new(&cfg.lotus.host, cfg.lotus.token.clone());
        let password = cfg.get_password();
        let wallet = Wallet::new(store, &password);
        Self { api, wallet }
    }

    #[allow(dead_code)]
    pub async fn transfer(&self, from: &str, to: &str, amount: &str) -> Result<Cid> {
        let msg = self.build_message(from, to, 0, amount, vec![]).await?;
        self.sign_and_push(msg, from).await
    }

    pub async fn transfer_with_options(
        &self,
        from: &str,
        to: &str,
        amount: &str,
        gas_premium: &str,
        gas_feecap: &str,
        gas_limit: i64,
        method: u64,
        nonce: Option<u64>,
    ) -> Result<Cid> {
        let actual_nonce = match nonce {
            Some(n) if n > 0 => n,
            _ => self.api.mpool_get_nonce(from).await?,
        };
        let value = BigInt::parse_token_amount(amount)?;
        let gas_fee_cap = BigInt::parse_token_amount(gas_feecap)?;
        let gas_premium = BigInt::parse_token_amount(gas_premium)?;

        let mut msg = Message {
            version: 0,
            to: Address::from_string(to)?,
            from: Address::from_string(from)?,
            nonce: actual_nonce,
            value,
            gas_limit,
            gas_fee_cap,
            gas_premium,
            method,
            params: vec![],
        };

        if gas_limit == 0 {
            msg = self.api.gas_estimate(&msg).await?;
        }

        self.sign_and_push(msg, from).await
    }

    pub async fn miner_withdraw(&self, miner: &str, from: &str, amount: &str) -> Result<Cid> {
        let amount = BigInt::parse_token_amount(amount)?;
        let params_bytes = cbor::serialize_withdraw_balance_params(&amount)?;

        let msg = self.build_message(from, miner, METHOD_WITHDRAW_BALANCE, "0", params_bytes).await?;
        self.sign_and_push(msg, from).await
    }

    pub async fn market_withdraw(&self, address: &str, from: &str, amount: &str) -> Result<Cid> {
        let provider_or_client = Address::from_string(address)?;
        let amount = BigInt::parse_token_amount(amount)?;
        let params_bytes = cbor::serialize_market_withdraw_params(&provider_or_client, &amount)?;

        let msg = self.build_message(from, STORAGE_MARKET_ACTOR, METHOD_MARKET_WITHDRAW, "0", params_bytes).await?;
        self.sign_and_push(msg, from).await
    }

    pub async fn change_owner(&self, miner: &str, new_owner: &str, from: &str) -> Result<Cid> {
        let new_owner = Address::from_string(new_owner)?;
        let params_bytes = cbor::serialize_change_owner_params(&new_owner)?;

        let msg = self.build_message(from, miner, METHOD_CHANGE_OWNER, "0", params_bytes).await?;
        self.sign_and_push(msg, from).await
    }

    pub async fn propose_change_worker(&self, miner: &str, new_worker: &str, from: &str) -> Result<Cid> {
        let new_worker = Address::from_string(new_worker)?;
        let params_bytes = cbor::serialize_change_worker_params(&new_worker, &[])?;

        let msg = self.build_message(from, miner, METHOD_CHANGE_WORKER, "0", params_bytes).await?;
        self.sign_and_push(msg, from).await
    }

    pub async fn confirm_change_worker(&self, miner: &str, from: &str) -> Result<Cid> {
        let msg = self.build_message(from, miner, METHOD_CONFIRM_CHANGE_WORKER, "0", vec![]).await?;
        self.sign_and_push(msg, from).await
    }

    async fn build_message(&self, from: &str, to: &str, method: u64, value: &str, params: Vec<u8>) -> Result<Message> {
        let nonce = self.api.mpool_get_nonce(from).await?;
        let value = BigInt::parse_token_amount(value)?;

        let msg = Message {
            version: 0,
            to: Address::from_string(to)?,
            from: Address::from_string(from)?,
            nonce,
            value,
            gas_limit: 0,
            gas_fee_cap: BigInt::zero(),
            gas_premium: BigInt::zero(),
            method,
            params,
        };

        self.api.gas_estimate(&msg).await
    }

    async fn sign_and_push(&self, msg: Message, from: &str) -> Result<Cid> {
        let sig = self.wallet.sign(&msg, from)?;
        let signed = SignedMessage { message: msg, signature: sig };
        self.api.mpool_push(&signed).await
    }
}
