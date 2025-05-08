mod vault;

use anyhow::{anyhow, Context};
use borsh::BorshDeserialize;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::{fs, str::FromStr};

#[derive(Debug, Deserialize)]
struct Config {
    rpc_url: String,
    private_key: String,
    program_id: String,
}

#[derive(Parser)]
#[command(name = "vault-cli")]
#[command(about = "CLI to interact with Anchor deposit contract", long_about = None)]
struct Args {
    config: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Initialize,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
    Balance,
}

fn load_config(file_path: &str) -> anyhow::Result<Config> {
    let content = fs::read_to_string(file_path).context("Cannot read config.yaml")?;
    let cfg: Config = serde_yaml::from_str(&content).context("Invalid config")?;

    Ok(cfg)
}

fn derive_vault_pda(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", user.as_ref()], program_id)
}

fn send_tx(
    rpc: &RpcClient,
    payer: &Keypair,
    ix: Instruction,
    signers: &[&Keypair],
) -> anyhow::Result<Signature> {
    let blockhash = rpc
        .get_latest_blockhash()
        .context("failed to get latest blockhash")?;

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), signers, blockhash);

    rpc.send_and_confirm_transaction(&tx)
        .context("failed to send")
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cfg = load_config(&args.config).context("Failed to load config")?;
    let rpc = RpcClient::new_with_commitment(cfg.rpc_url, CommitmentConfig::default());

    let payer =
        read_keypair_file(&cfg.private_key).map_err(|e| anyhow!("invalid keypair: {:?}", e))?;
    let program_id = Pubkey::from_str(&cfg.program_id).context("invalid program id")?;

    let (vault_pda, _) = derive_vault_pda(&payer.pubkey(), &program_id);

    match args.command {
        Commands::Initialize => {
            println!("Initializing vault for user...");
            let ix = vault::encode_initialize_ix(program_id, vault_pda, payer.pubkey());
            let sig = send_tx(&rpc, &payer, ix, &[&payer]).context("failed to initialize")?;
            println!("Initialized. Signature: {}", sig);
        }

        Commands::Deposit { amount } => {
            println!("Depositing {} lamports...", amount);
            let ix = vault::encode_deposit_ix(program_id, vault_pda, payer.pubkey(), amount);
            let sig = send_tx(&rpc, &payer, ix, &[&payer]).context("failed to deposit")?;
            println!("Deposit complete: {}", sig);
        }

        Commands::Withdraw { amount } => {
            println!("Withdrawing {} lamports...", amount);
            let ix = vault::encode_withdraw_ix(program_id, vault_pda, payer.pubkey(), amount);
            let sig = send_tx(&rpc, &payer, ix, &[&payer]).context("failed to withdraw")?;
            println!("Withdraw complete: {}", sig);
        }

        Commands::Balance => match rpc.get_account(&vault_pda) {
            Ok(acc) => {
                let vault = vault::Vault::try_from_slice(&acc.data[8..])
                    .context("Invalid vault account")?;
                println!("Vault balance (in state): {} lamports", vault.balance);
            }
            Err(_) => println!("Vault not initialized"),
        },
    }

    Ok(())
}
