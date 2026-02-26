use anchor_lang::prelude::*;

declare_id!("Dm88AgRVd7ddjKn2N27a5oxod5kETjKvPjJq5KZ6BbBS");

#[program]
pub mod solana_savings_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
