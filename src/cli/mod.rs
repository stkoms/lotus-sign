mod wallet;
mod send;
mod actor;
mod withdraw;
mod market;
mod push;

use crate::config::Config;
use crate::db::Store;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lotus-sign")]
#[command(about = "Filecoin wallet local signing tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Wallet(wallet::WalletCmd),
    Send(send::SendCmd),
    Actor(actor::ActorCmd),
    Withdraw(withdraw::WithdrawCmd),
    MarketWithdraw(market::MarketWithdrawCmd),
    MpoolPush(push::PushCmd),
}

pub async fn run(cli: Cli, cfg: Config, store: Store) -> Result<()> {
    match cli.command {
        Commands::Wallet(cmd) => wallet::run(cmd, &cfg, &store).await,
        Commands::Send(cmd) => send::run(cmd, &cfg, &store).await,
        Commands::Actor(cmd) => actor::run(cmd, &cfg, &store).await,
        Commands::Withdraw(cmd) => withdraw::run(cmd, &cfg, &store).await,
        Commands::MarketWithdraw(cmd) => market::run(cmd, &cfg, &store).await,
        Commands::MpoolPush(cmd) => push::run(cmd, &cfg, &store).await,
    }
}
