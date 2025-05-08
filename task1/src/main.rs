use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use serde::Deserialize;
use std::fs::File;
use futures::future::join_all;
use serde_yaml;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("config.yaml")?;
    let config: Config = serde_yaml::from_reader(file)?;

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );

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