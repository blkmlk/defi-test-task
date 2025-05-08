use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub payer_keypair: String,
    pub receiver_pubkey: String,
    pub transfer_amount_lamports: u64,
    pub geyser_grpc_address: String,
    pub geyser_token: String,
}

impl Config {
    pub fn from_yaml(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}