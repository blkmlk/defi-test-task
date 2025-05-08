use anyhow::Context;
use futures::future::join_all;
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

    let tasks = config.wallets.iter().map(|addr| {
        let addr = addr.clone();
        async {
            let pubkey = addr.parse::<Pubkey>().expect("Invalid pubkey");
            match client.get_balance(&pubkey).await {
                Ok(balance) => Some((addr, balance)),
                Err(e) => {
                    eprintln!("Error fetching balance for {}: {}", addr, e);
                    None
                }
            }
        }
    });

    let results = join_all(tasks).await;

    for result in results.into_iter().flatten() {
        println!("Wallet: {}, Balance: {} lamports", result.0, result.1);
    }

    Ok(())
}
