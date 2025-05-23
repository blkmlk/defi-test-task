use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::{pubkey::Pubkey, signature::Signer, system_instruction, transaction::Transaction};
use std::sync::Arc;
use std::{fs::File, path::Path};
use tokio::task::JoinSet;
use tokio::time::Instant;

#[derive(Parser)]
struct Args {
    #[arg(default_value = "config.yaml")]
    config: String,
}

#[derive(Deserialize)]
struct Wallet {
    private_key: String,
}

#[derive(Deserialize)]
struct Receiver {
    address: String,
}

#[derive(Deserialize)]
struct Config {
    rpc_url: String,
    senders: Vec<Wallet>,
    receivers: Vec<Receiver>,
    amount: f64,
}

fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let file = File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config(&args.config)?;

    if config.senders.len() != config.receivers.len() {
        return Err(anyhow!("senders and receivers count mismatch"));
    }

    let client = Arc::new(RpcClient::new(config.rpc_url));
    let lamports = solana_sdk::native_token::sol_to_lamports(config.amount);
    let started_at = Instant::now();

    let total = config.senders.len();
    let mut set = JoinSet::new();

    for (sender, receiver) in config.senders.into_iter().zip(config.receivers.into_iter()) {
        let client = client.clone();
        set.spawn(async move {
            let keypair = read_keypair_file(&sender.private_key)
                .map_err(|e| anyhow!("invalid keypair: {:?}", e))?;
            let from_pub = keypair.pubkey();
            let to_pub: Pubkey = receiver
                .address
                .parse()
                .context("failed to parse receiver public key")?;

            let blockhash = client
                .get_latest_blockhash()
                .context("failed to get latest blockhash")?;
            let tx = Transaction::new_signed_with_payer(
                &[system_instruction::transfer(&from_pub, &to_pub, lamports)],
                Some(&from_pub),
                &[&keypair],
                blockhash,
            );

            let sig = client
                .send_and_confirm_transaction(&tx)
                .context("failed to send a tx")?;
            Ok::<_, anyhow::Error>((sig.to_string(), from_pub.to_string(), to_pub.to_string()))
        });
    }

    let mut successes = 0;

    while let Some(result) = set.join_next().await {
        match result? {
            Ok((sig, from, to)) => {
                println!("Success: {} -> {} | tx = {}", from, to, sig);
                successes += 1;
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    println!("\n=== Transfer Stats ===");
    println!("Elapsed: {:?}", started_at.elapsed());
    println!("Total: {}", total);
    println!("Success: {}", successes);

    Ok(())
}
