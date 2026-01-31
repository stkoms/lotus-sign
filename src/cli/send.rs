//! 发送 FIL 代币的命令

use crate::config::Config;
use crate::db::Store;
use crate::service::Executor;
use anyhow::Result;
use clap::Args;

/// 发送 FIL 代币的命令参数
#[derive(Args)]
pub struct SendCmd {
    /// 目标地址（f1/f3 格式）
    pub to: String,
    /// 发送金额（单位：FIL，如 "0.1"）
    pub amount: String,
    /// 发送地址（钱包中必须有对应私钥）
    #[arg(long)]
    pub from: String,
    /// Gas 优先费（默认：0，自动估算）
    #[arg(long, default_value = "0")]
    pub gas_premium: String,
    /// Gas 费用上限（默认：0，自动估算）
    #[arg(long, default_value = "0")]
    pub gas_feecap: String,
    /// Gas 限制（默认：0，自动估算）
    #[arg(long, default_value = "0")]
    pub gas_limit: i64,
    /// 方法号（默认：0 = 转账）
    #[arg(long, default_value = "0")]
    pub method: u64,
    /// Nonce 覆盖（默认：从链上获取）
    #[arg(long)]
    pub nonce: Option<u64>,
}

/// 执行发送命令：签名并广播转账消息
pub async fn run(cmd: SendCmd, cfg: &Config, store: &Store) -> Result<()> {
    let executor = Executor::new(cfg, store);
    let cid = executor.transfer_with_options(
        &cmd.from,
        &cmd.to,
        &cmd.amount,
        &cmd.gas_premium,
        &cmd.gas_feecap,
        cmd.gas_limit,
        cmd.method,
        cmd.nonce,
    ).await?;
    println!("Message CID: {}", cid.root);
    Ok(())
}
