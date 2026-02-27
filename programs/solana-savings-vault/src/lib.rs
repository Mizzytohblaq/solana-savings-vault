use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("8QEXwvBH22arLe4XBYRHXCKjjqQ7YaJRPxq1Aj1ZDpzC");

#[program]
pub mod solana_savings_vault {
    use super::*;

    pub fn create_vault(
        ctx: Context<CreateVault>,
        amount: u64,
        lock_duration_days: u64,
        token_type: TokenType,
    ) -> Result<()> {
        require!(lock_duration_days >= 30, VaultError::LockTooShort);
        require!(lock_duration_days <= 1095, VaultError::LockTooLong);
        require!(amount > 0, VaultError::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        vault.owner = ctx.accounts.owner.key();
        vault.amount = amount;
        vault.token_type = token_type;
        vault.created_at = clock.unix_timestamp;
        vault.unlock_time = clock.unix_timestamp + (lock_duration_days as i64 * 86400);
        vault.lock_duration_days = lock_duration_days;
        vault.is_withdrawn = false;
        vault.bump = ctx.bumps.vault;

        // Transfer tokens from user to vault token account
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        msg!("Vault created! Locked for {} days", lock_duration_days);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        require!(!vault.is_withdrawn, VaultError::AlreadyWithdrawn);
        require!(
            clock.unix_timestamp >= vault.unlock_time,
            VaultError::StillLocked
        );

        let amount = vault.amount;
        vault.is_withdrawn = true;

        // Transfer tokens back to user
        let owner_key = vault.owner;
        let bump = vault.bump;
        let seeds = &[
            b"vault",
            owner_key.as_ref(),
            &[bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        msg!("Withdrawal successful! Amount: {}", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + VaultAccount::INIT_SPACE,
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        has_one = owner
    )]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct VaultAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub token_type: TokenType,
    pub created_at: i64,
    pub unlock_time: i64,
    pub lock_duration_days: u64,
    pub is_withdrawn: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum TokenType {
    SOL,
    USDC,
    USDT,
    JupUSD,
}

#[error_code]
pub enum VaultError {
    #[msg("Minimum lock period is 30 days")]
    LockTooShort,
    #[msg("Maximum lock period is 3 years (1095 days)")]
    LockTooLong,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Vault is still locked")]
    StillLocked,
    #[msg("Already withdrawn")]
    AlreadyWithdrawn,
}