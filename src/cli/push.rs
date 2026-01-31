use crate::config::Config;
use crate::db::Store;
use crate::chain::SignedMessage;
use crate::rpc::LotusApi;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct PushCmd {
    pub signed_message: String,
}

pub async fn run(cmd: PushCmd, cfg: &Config, _store: &Store) -> Result<()> {
    let api = LotusApi::new(&cfg.lotus.host, cfg.lotus.token.clone());
    let msg: SignedMessage = serde_json::from_str(&cmd.signed_message)?;
    let cid = api.mpool_push(&msg).await?;
    println!("Message CID: {}", cid.root);
    Ok(())
}
