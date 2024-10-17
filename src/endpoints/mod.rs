// use anyhow::{Context};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use serde_json::json;
use eyre::{Context, Result, eyre};

use crate::types::{
    type_utils::convert_hex_string_to_i64, BlockHeaderWithEmptyTransaction,
    BlockHeaderWithFullTransaction,
};

static CLIENT: Lazy<Client> = Lazy::new(Client::new);
static NODE_CONNECTION_STRING: Lazy<String> = Lazy::new(|| {
    dotenvy::var("NODE_CONNECTION_STRING").expect("NODE_CONNECTION_STRING must be set")
});

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub result: T,
}

#[derive(Serialize)]
struct RpcRequest<'a, T> {
    jsonrpc: &'a str,
    id: &'a str,
    method: &'a str,
    params: T,
}

pub async fn get_latest_finalized_blocknumber(timeout: Option<u64>) -> Result<i64> {
    let params = RpcRequest {
        jsonrpc: "2.0",
        id: "0",
        method: "eth_getBlockByNumber",
        params: vec!["finalized", "false"],
    };

    match make_rpc_call::<_, BlockHeaderWithEmptyTransaction>(&params, timeout)
        .await
        .context("Failed to get latest block number")
    {
        Ok(blockheader) => Ok(convert_hex_string_to_i64(&blockheader.number)),
        Err(e) => Err(e),
    }
}

pub async fn get_full_block_by_number(
    number: i64,
    timeout: Option<u64>,
) -> eyre::Result<BlockHeaderWithFullTransaction> {
    let params: Vec<serde_json::Value> = vec![
        json!(format!("0x{:x}", number)),  // Hex string for block number
        json!(true),                       // Boolean indicating full transaction data
    ];

    let request = RpcRequest {
        jsonrpc: "2.0",
        id: "0",
        method: "eth_getBlockByNumber",
        params,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(NODE_CONNECTION_STRING.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .timeout(std::time::Duration::from_secs(timeout.unwrap_or(300)))
        .send()
        .await
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?;

    let status = response.status();
    let headers = response.headers().clone();

    println!("Status: {}", status);
    println!("Headers: {:?}", headers);

    let response_text = response.text().await.map_err(|e| {
        eyre!("Failed to read response text: {}", e)
    })?;

    println!("Raw response for block {}: {}", number, response_text);

    let raw_response: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        eyre!(
            "Failed to decode response for block {}: {}\nResponse: {}",
            number,
            e,
            response_text
        )
    })?;

    let block_value = raw_response
        .get("result")
        .ok_or_else(|| eyre!("Missing 'result' field for block {}", number))?;

    let block: BlockHeaderWithFullTransaction = serde_json::from_value(block_value.clone()).map_err(
        |e| eyre!("Failed to decode block {}: {}", number, e),
    )?;

    println!("Successfully retrieved block: {:?}", block);
    Ok(block)
}


async fn make_rpc_call<T: Serialize, R: for<'de> Deserialize<'de>>(
    params: &T,
    timeout: Option<u64>,
) -> Result<R> {
    let raw_response = match timeout {
        Some(seconds) => {
            CLIENT
                .post(NODE_CONNECTION_STRING.as_str())
                .timeout(Duration::from_secs(seconds))
                .json(params)
                .send()
                .await
        }
        None => {
            CLIENT
                .post(NODE_CONNECTION_STRING.as_str())
                .json(params)
                .send()
                .await
        }
    };

    let response = raw_response?.json::<RpcResponse<R>>().await?;
    Ok(response.result)
}
