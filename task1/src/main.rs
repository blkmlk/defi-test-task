use anyhow::Context;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use serde::Deserialize;
use serde_yaml;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file = File::open("config.yaml").context("failed to open config.yaml")?;
    let config: Config = serde_yaml::from_reader(file).context("failed to parse config.yaml")?;

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let mut futures = FuturesUnordered::new();

    for addr in config.wallets.iter() {
        let client = &client;
        let f = async move {
            let pubkey = addr.parse::<Pubkey>().expect("Invalid pubkey");
            match client.get_balance(&pubkey).await {
                Ok(balance) => Some((addr, balance)),
                Err(e) => {
                    eprintln!("Error fetching balance for {}: {}", addr, e);
                    None
                }
            }
        };
        futures.push(f);
    }

    while let Some(res) = futures.next().await {
        if let Some((addr, balance)) = res {
            println!("Wallet: {}, Balance: {} lamports", addr, balance);
        }
    }

    Ok(())
}
