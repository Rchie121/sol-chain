use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("LEND1111111111111111111111111111111111111");

#[program]
pub mod lending {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.ltv = 8000; // 80%
        market.liquidation_threshold = 8500;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into_transfer_context(), amount)?;
        let position = &mut ctx.accounts.user_position;
        position.deposited += amount;
        Ok(())
    }

    // Add borrow, repay, etc.
}

#[account]
pub struct LendingMarket {
    pub ltv: u64,
    pub liquidation_threshold: u64,
}

#[account]
pub struct UserPosition {
    pub deposited: u64,
    pub borrowed: u64,
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = authority, space = 8 + 16)]
    pub market: Account<'info, LendingMarket>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, space = 8 + 16, seeds = [b"position", user.key().as_ref()], bump)]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
