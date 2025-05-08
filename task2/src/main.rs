use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, transaction::Transaction,
};
use serde::Deserialize;
use std::{fs::File, path::Path};
use anyhow::{Result, anyhow, Context};
use clap::Parser;
use futures::future::join_all;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "config.yaml")]
    config_path: String,
}

#[derive(Deserialize)]
struct Wallet {
    private_key: Vec<u8>,
}

#[derive(Deserialize)]
struct Receiver {
    address: String,
}

#[derive(Deserialize)]
struct Config {
    senders: Vec<Wallet>,
    receivers: Vec<Receiver>,
    amount: f64,
}

fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let file = File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
}

fn wallet_from_bytes(bytes: &[u8]) -> Result<Keypair> {
    Keypair::from_bytes(bytes).map_err(|e| anyhow!("Invalid keypair: {e}"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config(&args.config_path)?;

    if config.senders.len() != config.receivers.len() {
        return Err(anyhow!("senders and receivers count mismatch"));
    }

    let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let lamports = solana_sdk::native_token::sol_to_lamports(config.amount);

    let mut tasks = Vec::with_capacity(config.senders.len());

    for (sender, receiver) in config.senders.iter().zip(config.receivers.iter()) {
        let task = async {
            let keypair = wallet_from_bytes(&sender.private_key).context("failed to parse sender private key")?;
            let from_pub = keypair.pubkey();
            let to_pub: Pubkey = receiver.address.parse().context("failed to parse receiver public key")?;

            let blockhash = client.get_latest_blockhash().context("failed to get latest blockhash")?;
            let tx = Transaction::new_signed_with_payer(
                &[system_instruction::transfer(&from_pub, &to_pub, lamports)],
                Some(&from_pub),
                &[&keypair],
                blockhash,
            );

            let sig = client.send_and_confirm_transaction(&tx).context("failed to send a tx")?;
            Ok::<_, anyhow::Error>((sig.to_string(), from_pub.to_string(), to_pub.to_string()))
        };

        tasks.push(task)
    }

    let mut successes = 0;
    let mut txs = Vec::with_capacity(config.senders.len());

    let results = join_all(tasks).await;

    for result in results {
        match result {
            Ok((sig, from, to)) => {
                println!("Success: {} -> {} | tx = {}", from, to, sig);
                successes += 1;
                txs.push(sig);
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    println!("\n=== Transfer Summary ===");
    println!("Total: {}", config.senders.len());
    println!("Success: {}", successes);

    Ok(())
}