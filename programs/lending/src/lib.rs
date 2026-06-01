    pub fn liquidate(ctx: Context<Liquidate>, repay_amount: u64) -> Result<()> {
        let position = &mut ctx.accounts.user_position;
        let market = &ctx.accounts.market;

        // In production: fetch real price from oracle
        // let collateral_price = get_token_price(&ctx.accounts.price_oracle)?;
        // let health_factor = (position.deposited * collateral_price) / position.borrowed;

        // Simplified health check (assume 1:1 for demo)
        let health_factor = if position.borrowed > 0 {
            (position.deposited * 10000) / position.borrowed
        } else {
            10000
        };

        require!(health_factor < market.liquidation_threshold, ErrorCode::NotLiquidatable);

        // Repay part of the debt
        let to_repay = std::cmp::min(repay_amount, position.borrowed);
        position.borrowed = position.borrowed.checked_sub(to_repay).unwrap();

        // Seize collateral (simplified - in prod use bonus for liquidator)
        let collateral_to_seize = to_repay.checked_mul(110).unwrap() / 100; // 10% bonus
        position.deposited = position.deposited.saturating_sub(collateral_to_seize);

        // In production: transfer seized collateral to liquidator
        msg!("Liquidation executed. Repaid: {}, Seized: {}", to_repay, collateral_to_seize);

        Ok(())
    }

    // Add Liquidate context struct below
