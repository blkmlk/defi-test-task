use anchor_lang::prelude::*;

declare_id!("HUfP36vmMYFkvV1e7EQd6cebP5PWikLSzzwv4Z78Wc6Z");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.owner,
            ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );

        vault.balance = vault
            .balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let mut vault = &mut ctx.accounts.vault;

        require_keys_eq!(
            vault.owner,
            ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );

        require!(vault.balance >= amount, ErrorCode::InsufficientFunds);

        vault.balance -= amount;

        **ctx
            .accounts
            .vault
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(init, seeds = [b"vault", user.key().as_ref()], bump, payer = user, space = 8 + 32 + 8)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Not enough funds")]
    InsufficientFunds,
    #[msg("Overflow occurred")]
    Overflow,
}
