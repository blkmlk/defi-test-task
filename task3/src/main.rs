mod config;

use config::Config;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use anyhow::{anyhow, Context};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use solana_sdk::signature::{Keypair, Signature};
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::geyser::{SubscribeRequest, SubscribeRequestPing};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::from_yaml(&args.config)?;
    let payer = read_keypair_file(&config.payer_keypair).map_err(|e| anyhow!("invalid keypair: {:?}", e))?;
    let receiver = Pubkey::from_str(&config.receiver_pubkey)?;

    let rpc_client = RpcClient::new(config.rpc_url.clone());

    let mut client = GeyserGrpcClient::build_from_shared(config.geyser_grpc_address.clone())?
        .x_token(Some(config.geyser_token.clone()))?
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .max_decoding_message_size(1024 * 1024 * 1024)
        .connect()
        .await
        .context("failed to build a geyser client")?;


    let req = SubscribeRequest::default();

    let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(req)).await?;

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => match msg.update_oneof {
                Some(UpdateOneof::Block(_)) => {
                    send_tx(&rpc_client, &payer, &receiver, config.transfer_amount_lamports).await?;
                }
                Some(UpdateOneof::Ping(_)) => {
                    subscribe_tx
                        .send(SubscribeRequest {
                            ping: Some(SubscribeRequestPing { id: 1 }),
                            ..Default::default()
                        })
                        .await?;
                }
                Some(UpdateOneof::Pong(_)) => {}
                None => {
                    // error!("update not found in the message");
                    break;
                }
                _ => {}
            },
            Err(error) => {
                // error!("error: {error:?}");
                break;
            }
        }
    }

    Ok(())
}

async fn send_tx(client: &RpcClient, payer: &Keypair, receiver: &Pubkey, amount: u64) -> anyhow::Result<Signature> {
    let ix = system_instruction::transfer(
        &payer.pubkey(),
        receiver,
        amount,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&tx).map_err(|e| anyhow!(e.to_string()))
}