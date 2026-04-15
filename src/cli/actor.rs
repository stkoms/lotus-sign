use crate::config::Config;
use crate::db::Store;
use crate::rpc::LotusApi;
use crate::chain::format_fil;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ActorCmd {
    #[command(subcommand)]
    pub command: ActorSubCmd,
}

#[derive(Subcommand)]
pub enum ActorSubCmd {
    Info {
        miner: String,
    },
    Withdraw {
        #[arg(long)]
        miner: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from: String,
    },
    SetOwner {
        #[arg(long)]
        miner: String,
        #[arg(long)]
        new_owner: String,
        #[arg(long)]
        from: String,
        #[arg(long, default_value = "false")]
        really_do_it: bool,
    },
    ProposeChangeWorker {
        #[arg(long)]
        miner: String,
        #[arg(long)]
        new_worker: String,
        #[arg(long)]
        from: String,
        #[arg(long, default_value = "false")]
        really_do_it: bool,
    },
    ConfirmChangeWorker {
        #[arg(long)]
        miner: String,
        #[arg(long)]
        from: String,
        #[arg(long, default_value = "false")]
        really_do_it: bool,
    },
}

pub async fn run(cmd: ActorCmd, cfg: &Config, store: &Store) -> Result<()> {
    let api = LotusApi::new(&cfg.lotus.host, cfg.lotus.token.clone());

    match cmd.command {
        ActorSubCmd::Info { miner } => {
            let info = api.state_miner_info(&miner).await?;
            let balance = api.state_miner_available_balance(&miner).await?;
            let market = api.state_market_balance(&miner).await?;
            let market_available = &market.escrow.0 - &market.locked.0;

            println!("Miner: {}", miner);
            println!("Owner: {}", info.owner);
            println!("Worker: {}", info.worker);
            println!("Available Balance: {} attoFIL ({})", balance, format_fil(&balance.0));
            println!("Market Escrow: {} attoFIL ({})", market.escrow, format_fil(&market.escrow.0));
            println!("Market Locked: {} attoFIL ({})", market.locked, format_fil(&market.locked.0));
            println!("Market Available: {} attoFIL ({})", market_available, format_fil(&market_available));
        }
        ActorSubCmd::Withdraw { miner, amount, from } => {
            use crate::service::Executor;
            let executor = Executor::new(cfg, store);
            let cid = executor.miner_withdraw(&miner, &from, &amount).await?;
            println!("Message CID: {}", cid.root);
        }
        ActorSubCmd::SetOwner { miner, new_owner, from, really_do_it } => {
            if !really_do_it {
                println!("Pass --really-do-it to actually execute this action");
                return Ok(());
            }
            use crate::service::Executor;
            let executor = Executor::new(cfg, store);
            let cid = executor.change_owner(&miner, &new_owner, &from).await?;
            println!("Message CID: {}", cid.root);
        }
        ActorSubCmd::ProposeChangeWorker { miner, new_worker, from, really_do_it } => {
            if !really_do_it {
                println!("Pass --really-do-it to actually execute this action");
                return Ok(());
            }
            use crate::service::Executor;
            let executor = Executor::new(cfg, store);
            let cid = executor.propose_change_worker(&miner, &new_worker, &from).await?;
            println!("Message CID: {}", cid.root);
        }
        ActorSubCmd::ConfirmChangeWorker { miner, from, really_do_it } => {
            if !really_do_it {
                println!("Pass --really-do-it to actually execute this action");
                return Ok(());
            }
            use crate::service::Executor;
            let executor = Executor::new(cfg, store);
            let cid = executor.confirm_change_worker(&miner, &from).await?;
            println!("Message CID: {}", cid.root);
        }
    }
    Ok(())
}
