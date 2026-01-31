use crate::config::Config;
use crate::db::Store;
use crate::service::Executor;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct WithdrawCmd {
    #[arg(long)]
    pub miner: String,
    #[arg(long)]
    pub amount: String,
    #[arg(long)]
    pub from: String,
}

pub async fn run(cmd: WithdrawCmd, cfg: &Config, store: &Store) -> Result<()> {
    let executor = Executor::new(cfg, store);
    let cid = executor.miner_withdraw(&cmd.miner, &cmd.from, &cmd.amount).await?;
    println!("Withdraw Message CID: {}", cid.root);
    Ok(())
}
