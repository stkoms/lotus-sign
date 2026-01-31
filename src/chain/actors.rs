use super::{Address, BigInt};
use serde::Serialize;

// Miner Actor Method Numbers
pub const METHOD_WITHDRAW_BALANCE: u64 = 16;
pub const METHOD_CHANGE_OWNER: u64 = 23;
pub const METHOD_CHANGE_WORKER: u64 = 3;
pub const METHOD_CONFIRM_CHANGE_WORKER: u64 = 21;

// Market Actor Method Numbers
pub const METHOD_MARKET_WITHDRAW: u64 = 2;

// Storage Market Actor Address
pub const STORAGE_MARKET_ACTOR: &str = "f05";

#[derive(Debug, Clone, Serialize)]
pub struct WithdrawBalanceParams {
    pub amount: BigInt,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeOwnerParams {
    pub new_owner: Address,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeWorkerParams {
    pub new_worker: Address,
    pub new_control_addresses: Vec<Address>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketWithdrawParams {
    pub provider_or_client: Address,
    pub amount: BigInt,
}
