use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// Replace with your real program ID after `anchor build`
declare_id!("LEND1111111111111111111111111111111111111");

#[program]
pub mod lending {
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.ltv = 8000;                    // 80% Loan-to-Value
        market.liquidation_threshold = 8500;  // 85% liquidation threshold
        market.interest_rate = 500;           // 5% APR (simplified)
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        token::transfer(ctx.accounts.into_transfer_context(), amount)?;

        let position = &mut ctx.accounts.user_position;
        position.deposited = position.deposited.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let position = &mut ctx.accounts.user_position;
        require!(position.deposited >= amount, ErrorCode::InsufficientCollateral);

        // Check if withdrawal would undercollateralize existing borrows
        if position.borrowed > 0 {
            let remaining_collateral = position.deposited.checked_sub(amount).unwrap();
            let max_borrow = remaining_collateral.checked_mul(8000).unwrap() / 10000; // 80% LTV
            require!(position.borrowed <= max_borrow, ErrorCode::Undercollateralized);
        }

        token::transfer(ctx.accounts.into_withdraw_context(), amount)?;
        position.deposited = position.deposited.checked_sub(amount).unwrap();

        Ok(())
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        let position = &mut ctx.accounts.user_position;
        let market = &ctx.accounts.market;

        // Calculate max borrowable (80% of deposited)
        let max_borrow = position.deposited.checked_mul(market.ltv as u64).unwrap() / 10000;
        let new_borrow_total = position.borrowed.checked_add(amount).unwrap();

        require!(new_borrow_total <= max_borrow, ErrorCode::InsufficientCollateral);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // In production: transfer from vault to user
        // token::transfer(ctx.accounts.into_borrow_context(), amount)?;

        position.borrowed = new_borrow_total;

        Ok(())
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        let position = &mut ctx.accounts.user_position;

        require!(position.borrowed >= amount, ErrorCode::InvalidAmount);

        // In production: transfer from user to vault
        // token::transfer(ctx.accounts.into_repay_context(), amount)?;

        position.borrowed = position.borrowed.checked_sub(amount).unwrap();

        Ok(())
    }

    // Future: liquidate function + oracle price feed integration
}

#[account]
pub struct LendingMarket {
    pub ltv: u64,
    pub liquidation_threshold: u64,
    pub interest_rate: u64, // basis points
}

#[account]
pub struct UserPosition {
    pub deposited: u64,
    pub borrowed: u64,
    // In production: last_updated timestamp for interest
}

// ==================== CONTEXTS ====================

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(init, payer = authority, space = 8 + 24)]
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
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 16,
        seeds = [b"position", user.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token.to_account_info(),
                to: self.vault.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"position", user.key().as_ref()], bump)]
    pub user_position: Account<'info, UserPosition>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn into_withdraw_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.user_token.to_account_info(),
                authority: self.vault.to_account_info(), // In prod: use PDA authority
            },
        )
    }
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub market: Account<'info, LendingMarket>,
    #[account(mut, seeds = [b"position", user.key().as_ref()], bump)]
    pub user_position: Account<'info, UserPosition>,
    pub user: Signer<'info>,
    // Add vault account for real token transfer
}

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut, seeds = [b"position", user.key().as_ref()], bump)]
    pub user_position: Account<'info, UserPosition>,
    pub user: Signer<'info>,
    // Add user token and vault for real transfer
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient collateral for this action")]
    InsufficientCollateral,
    #[msg("Position would become undercollateralized")]
    Undercollateralized,
    #[msg("Invalid amount")]
    InvalidAmount,
}
