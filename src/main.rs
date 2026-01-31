mod cli;
mod chain;
mod config;
mod crypto;
mod db;
mod rpc;
mod service;
mod wallet;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = config::Config::load()?;
    let store = db::Store::open(&cfg.database.path)?;

    let args = cli::Cli::parse();
    cli::run(args, cfg, store).await
}
