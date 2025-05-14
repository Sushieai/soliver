use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole;
use wormhole_anchor_sdk::wormhole::Finality;

declare_id!("AYt9eRP7iTowpVC8FM4UWkod67zRm9LMTPMZF8LWLpJg");


#[account]
pub struct UserVault {
    pub loan_amount: u64,
    pub is_active: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("No active loan")]
    NoActiveLoan,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init_if_needed, payer = user, space = 8 + 8 + 1, seeds = [b"vault", user.key().as_ref()], bump)]
    pub user_vault: Account<'info, UserVault>,

    /// CHECK:
    pub wormhole_program: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_config: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_message: AccountInfo<'info>,
    #[account(mut)]
    pub wormhole_emitter: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_sequence: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_fee_collector: AccountInfo<'info>,
    /// CHECK:
    pub clock: AccountInfo<'info>,
    /// CHECK:
    pub rent: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Borrow<'info> {
    fn wormhole_ctx(&self) -> CpiContext<'_, '_, '_, 'info, wormhole::PostMessage<'info>> {
        let accounts = wormhole::PostMessage {
            config: self.wormhole_config.clone(),
            message: self.wormhole_message.clone(),
            emitter: self.wormhole_emitter.to_account_info(),
            sequence: self.wormhole_sequence.clone(),
            payer: self.user.to_account_info(),
            fee_collector: self.wormhole_fee_collector.clone(),
            clock: self.clock.clone(),
            rent: self.rent.clone(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(self.wormhole_program.clone(), accounts)
    }
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub user_vault: Account<'info, UserVault>,

    /// CHECK:
    pub wormhole_program: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_config: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_message: AccountInfo<'info>,
    #[account(mut)]
    pub wormhole_emitter: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_sequence: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_fee_collector: AccountInfo<'info>,
    /// CHECK:
    pub clock: AccountInfo<'info>,
    /// CHECK:
    pub rent: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub user_vault: Account<'info, UserVault>,

    /// CHECK:
    pub wormhole_program: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_config: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_message: AccountInfo<'info>,
    #[account(mut)]
    pub wormhole_emitter: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_sequence: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub wormhole_fee_collector: AccountInfo<'info>,
    /// CHECK:
    pub clock: AccountInfo<'info>,
    /// CHECK:
    pub rent: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Liquidate<'info> {
    fn wormhole_ctx(&self) -> CpiContext<'_, '_, '_, 'info, wormhole::PostMessage<'info>> {
        let accounts = wormhole::PostMessage {
            config: self.wormhole_config.clone(),
            message: self.wormhole_message.clone(),
            emitter: self.wormhole_emitter.to_account_info(),
            sequence: self.wormhole_sequence.clone(),
            payer: self.wormhole_emitter.to_account_info(),
            fee_collector: self.wormhole_fee_collector.clone(),
            clock: self.clock.clone(),
            rent: self.rent.clone(),
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new(self.wormhole_program.clone(), accounts)
    }
}


#[program]
pub mod soliver {
    use super::*;

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.user_vault;
        vault.loan_amount = amount;
        vault.is_active = true;

        let payload = format!("borrow|{}|{}", ctx.accounts.user.key(), amount).into_bytes();
        wormhole::post_message(
            ctx.accounts.wormhole_ctx(),
            0, // nonce
            payload,
            Finality::Finalized,
        )?;

        Ok(())
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.user_vault;
        require!(vault.is_active, ErrorCode::NoActiveLoan);

        vault.loan_amount = vault.loan_amount.saturating_sub(amount);
        if vault.loan_amount == 0 {
            vault.is_active = false;

            let payload = format!("repay|{}", ctx.accounts.user.key()).into_bytes();
            wormhole::post_message(
                ctx.accounts.wormhole_ctx(),
                0,
                payload,
                Finality::Finalized,
            )?;
        }

        Ok(())
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        let vault = &mut ctx.accounts.user_vault;
        require!(vault.is_active, ErrorCode::NoActiveLoan);

        vault.is_active = false;
        vault.loan_amount = 0;

        let payload = format!("liquidate|{}", ctx.accounts.user_vault.key()).into_bytes(); // or track vault owner in account

        wormhole::post_message(
            ctx.accounts.wormhole_ctx(),
            0,
            payload,
            Finality::Finalized,
        )?;

        Ok(())
    }
}
