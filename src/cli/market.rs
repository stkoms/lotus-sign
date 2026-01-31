use crate::config::Config;
use crate::db::Store;
use crate::service::Executor;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct MarketWithdrawCmd {
    #[arg(long)]
    pub address: String,
    #[arg(long)]
    pub amount: String,
    #[arg(long)]
    pub from: String,
}

pub async fn run(cmd: MarketWithdrawCmd, cfg: &Config, store: &Store) -> Result<()> {
    let executor = Executor::new(cfg, store);
    let cid = executor.market_withdraw(&cmd.address, &cmd.from, &cmd.amount).await?;
    println!("Market Withdraw CID: {}", cid.root);
    Ok(())
}
