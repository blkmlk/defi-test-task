use borsh::BorshDeserialize;
use sha2::Digest;
use sha2::Sha256;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[derive(Debug, BorshDeserialize)]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

fn anchor_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", name));
    let hash = hasher.finalize();
    let mut output = [0u8; 8];
    output.copy_from_slice(&hash[..8]);
    output
}

pub fn encode_initialize_ix(program_id: Pubkey, vault: Pubkey, user: Pubkey) -> Instruction {
    let mut data = vec![];
    data.extend_from_slice(&anchor_discriminator("initialize"));

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

pub fn encode_deposit_ix(
    program_id: Pubkey,
    vault: Pubkey,
    user: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = vec![];
    data.extend_from_slice(&anchor_discriminator("deposit"));
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

pub fn encode_withdraw_ix(
    program_id: Pubkey,
    vault: Pubkey,
    user: Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = vec![];
    data.extend_from_slice(&anchor_discriminator("withdraw"));
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vault, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}
