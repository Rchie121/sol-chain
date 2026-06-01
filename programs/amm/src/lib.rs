use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo, Burn};

// Replace with your real program ID after `anchor build`
declare_id!("AMM1111111111111111111111111111111111111111");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, bump: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.token_a_reserve = 0;
        pool.token_b_reserve = 0;
        pool.lp_mint = ctx.accounts.lp_mint.key();
        pool.bump = bump;
        pool.fee_numerator = 3;      // 0.3% fee
        pool.fee_denominator = 1000;
        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // Transfer token A
        token::transfer(ctx.accounts.into_transfer_a_context(), amount_a)?;
        // Transfer token B
        token::transfer(ctx.accounts.into_transfer_b_context(), amount_b)?;

        // Update reserves
        pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_a).unwrap();
        pool.token_b_reserve = pool.token_b_reserve.checked_add(amount_b).unwrap();

        // Calculate LP tokens to mint (using geometric mean for shares)
        let total_supply = ctx.accounts.lp_mint.supply;
        let liquidity = if total_supply == 0 {
            // First liquidity provider - use geometric mean
            (amount_a as u128).checked_mul(amount_b as u128).unwrap().sqrt() as u64
        } else {
            // Subsequent providers
            let share_a = (amount_a as u128).checked_mul(total_supply as u128).unwrap() / pool.token_a_reserve as u128;
            let share_b = (amount_b as u128).checked_mul(total_supply as u128).unwrap() / pool.token_b_reserve as u128;
            std::cmp::min(share_a, share_b) as u64
        };

        require!(liquidity > 0, ErrorCode::InvalidLiquidity);

        // Mint LP tokens
        token::mint_to(ctx.accounts.into_mint_lp_context(), liquidity)?;

        Ok(())
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, liquidity: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        let total_supply = ctx.accounts.lp_mint.supply;
        require!(total_supply > 0, ErrorCode::NoLiquidity);

        // Calculate proportional amounts
        let amount_a = (liquidity as u128).checked_mul(pool.token_a_reserve as u128).unwrap() / total_supply as u128;
        let amount_b = (liquidity as u128).checked_mul(pool.token_b_reserve as u128).unwrap() / total_supply as u128;

        // Burn LP tokens
        token::burn(ctx.accounts.into_burn_lp_context(), liquidity)?;

        // Transfer tokens back to user
        token::transfer(ctx.accounts.into_transfer_a_out_context(), amount_a as u64)?;
        token::transfer(ctx.accounts.into_transfer_b_out_context(), amount_b as u64)?;

        // Update reserves
        pool.token_a_reserve = pool.token_a_reserve.checked_sub(amount_a as u64).unwrap();
        pool.token_b_reserve = pool.token_b_reserve.checked_sub(amount_b as u64).unwrap();

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_out: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let is_a_to_b = ctx.accounts.input_mint.key() == pool.token_a_mint;

        let (reserve_in, reserve_out) = if is_a_to_b {
            (pool.token_a_reserve, pool.token_b_reserve)
        } else {
            (pool.token_b_reserve, pool.token_a_reserve)
        };

        require!(reserve_in > 0 && reserve_out > 0, ErrorCode::InsufficientLiquidity);

        // Apply fee (0.3%)
        let fee = amount_in
            .checked_mul(pool.fee_numerator as u64)
            .unwrap()
            .checked_div(pool.fee_denominator as u64)
            .unwrap();
        let amount_in_after_fee = amount_in.checked_sub(fee).unwrap();

        // Constant product formula: x * y = k
        // amount_out = reserve_out - (k / (reserve_in + amount_in_after_fee))
        let k = (reserve_in as u128).checked_mul(reserve_out as u128).unwrap();
        let new_reserve_in = (reserve_in as u128).checked_add(amount_in_after_fee as u128).unwrap();
        let new_reserve_out = k.checked_div(new_reserve_in).unwrap();
        let amount_out = (reserve_out as u128).checked_sub(new_reserve_out).unwrap() as u64;

        require!(amount_out >= minimum_out, ErrorCode::SlippageExceeded);
        require!(amount_out > 0, ErrorCode::InvalidSwap);

        // Execute transfers
        token::transfer(ctx.accounts.into_transfer_in_context(), amount_in)?;
        token::transfer(ctx.accounts.into_transfer_out_context(), amount_out)?;

        // Update reserves
        if is_a_to_b {
            pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_in).unwrap();
            pool.token_b_reserve = pool.token_b_reserve.checked_sub(amount_out).unwrap();
        } else {
            pool.token_b_reserve = pool.token_b_reserve.checked_add(amount_in).unwrap();
            pool.token_a_reserve = pool.token_a_reserve.checked_sub(amount_out).unwrap();
        }

        Ok(())
    }
}

#[account]
pub struct Pool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub lp_mint: Pubkey,
    pub bump: u8,
    pub fee_numerator: u16,
    pub fee_denominator: u16,
}

// ==================== CONTEXTS ====================

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 * 3 + 8 * 2 + 1 + 4,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = pool
    )]
    pub lp_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_lp: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AddLiquidity<'info> {
    pub fn into_transfer_a_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token_a.to_account_info(),
                to: self.pool_token_a.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    pub fn into_transfer_b_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token_b.to_account_info(),
                to: self.pool_token_b.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    pub fn into_mint_lp_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.lp_mint.to_account_info(),
                to: self.user_lp.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_lp: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RemoveLiquidity<'info> {
    pub fn into_burn_lp_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.lp_mint.to_account_info(),
                from: self.user_lp.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    pub fn into_transfer_a_out_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_token_a.to_account_info(),
                to: self.user_token_a.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        )
    }

    pub fn into_transfer_b_out_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_token_b.to_account_info(),
                to: self.user_token_b.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub input_mint: Account<'info, Mint>,
    pub output_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_input: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_output: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_input: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_output: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Swap<'info> {
    pub fn into_transfer_in_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_input.to_account_info(),
                to: self.pool_input.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    pub fn into_transfer_out_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_output.to_account_info(),
                to: self.user_output.to_account_info(),
                authority: self.pool.to_account_info(),
            },
        )
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Invalid liquidity amount")]
    InvalidLiquidity,
    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,
    #[msg("Invalid swap parameters")]
    InvalidSwap,
    #[msg("No liquidity in pool")]
    NoLiquidity,
}
