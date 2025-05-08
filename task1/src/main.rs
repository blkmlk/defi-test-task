use anyhow::Context;
use serde::Deserialize;
use serde_yaml;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::fs::File;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open("config.yaml").context("failed to open config.yaml")?;
    let config: Config = serde_yaml::from_reader(file).context("failed to parse config.yaml")?;

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    ));

    let mut set = JoinSet::new();

    for addr in config.wallets.into_iter() {
        let client = client.clone();
        set.spawn(async move {
            let pubkey = addr.parse::<Pubkey>().expect("Invalid pubkey");
            match client.get_balance(&pubkey).await {
                Ok(balance) => Some((addr, balance)),
                Err(e) => {
                    eprintln!("Error fetching balance for {}: {}", addr, e);
                    None
                }
            }
        });
    }

    while let Some(res) = set.join_next().await {
        if let Some((addr, balance)) = res? {
            println!("Wallet: {}, Balance: {} lamports", addr, balance);
        }
    }

    Ok(())
}
