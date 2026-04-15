use anyhow::Result;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub struct LotusClient {
    client: Client,
    url: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct RpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl LotusClient {
    pub fn new(url: &str, token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            url: url.to_string(),
            token,
        }
    }

    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            method: format!("Filecoin.{}", method),
            params,
            id: 1,
        };

        let mut builder = self.client.post(&self.url).json(&req);

        if let Some(ref token) = self.token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        let resp: RpcResponse<T> = builder.send().await?.json().await?;

        if let Some(err) = resp.error {
            anyhow::bail!("RPC error {}: {}", err.code, err.message);
        }

        resp.result.ok_or_else(|| anyhow::anyhow!("empty result"))
    }
}
