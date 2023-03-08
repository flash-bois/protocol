use crate::{
    core_lib::{decimal::BalanceChange, structs::Side},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, transfer, TokenAccount, Transfer};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct ClosePosition<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, seeds = [b"statement".as_ref(), signer.key.as_ref()], bump=statement.load()?.bump, constraint = statement.load()?.owner == signer.key())]
    pub statement: AccountLoader<'info, Statement>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,
    constraint = account_base.mint == reserve_base.mint,
    constraint = account_base.owner == signer.key(),
)]
    pub account_base: Account<'info, TokenAccount>,
    #[account(mut,
    constraint = account_quote.mint == reserve_quote.mint,
    constraint = account_quote.owner == signer.key(),
)]
    pub account_quote: Account<'info, TokenAccount>,
    #[account(mut,
    constraint = reserve_base.mint == vaults.load()?.keys.get(vault as usize).unwrap().base_token,
    constraint = reserve_base.key() == vaults.load()?.keys.get(vault as usize).unwrap().base_reserve,
    constraint = reserve_base.owner == state.key(),
)]
    pub reserve_base: Account<'info, TokenAccount>,
    #[account(mut,
    constraint = reserve_quote.mint == vaults.load()?.keys.get(vault as usize).unwrap().quote_token,
    constraint = reserve_quote.key() == vaults.load()?.keys.get(vault as usize).unwrap().quote_reserve,
    constraint = reserve_quote.owner == state.key(),
)]
    pub reserve_quote: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

impl<'info> ClosePosition<'info> {
    pub fn handler(ctx: Context<ClosePosition>, vault: u8, long: bool) -> anchor_lang::Result<()> {
        msg!("DotWave: Close position");
        let current_timestamp = Clock::get()?.unix_timestamp;
        let user_statement = &mut ctx.accounts.statement.load_mut()?.statement;
        let vaults = &mut ctx.accounts.vaults.load_mut()?;

        let mut vaults_indexes = vec![vault];
        if let Some(indexes_to_refresh) = user_statement.get_vaults_indexes() {
            vaults_indexes.extend(indexes_to_refresh.iter());
            vaults_indexes.dedup()
        }

        vaults.refresh(&vaults_indexes, ctx.remaining_accounts, current_timestamp)?;
        user_statement.refresh(&vaults.arr.elements)?;

        let vault = vaults.vault_checked_mut(vault)?;
        let current_timestamp = Clock::get()?.unix_timestamp as u32;
        let side = if long { Side::Long } else { Side::Short };
        let balance_change = vault.close_position(user_statement, side, current_timestamp)?;

        match balance_change {
            BalanceChange::Profit(profit_amount) => {
                let seeds = &[
                    b"state".as_ref(),
                    &[ctx.accounts.state.load().unwrap().bump],
                ];
                let signer = &[&seeds[..]];

                let send_ctx = match side {
                    Side::Long => ctx.accounts.send_base(),
                    Side::Short => ctx.accounts.send_quote(),
                };

                transfer(send_ctx.with_signer(signer), profit_amount.get())?;
            }
            BalanceChange::Loss(loss_amount) => {
                let take_ctx = match side {
                    Side::Long => ctx.accounts.take_base(),
                    Side::Short => ctx.accounts.take_quote(),
                };

                transfer(take_ctx, loss_amount.get())?;
            }
        }

        Ok(())
    }

    fn send_base(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_base.to_account_info(),
                to: self.account_base.to_account_info(),
                authority: self.state.to_account_info(),
            },
        )
    }

    fn send_quote(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_quote.to_account_info(),
                to: self.account_quote.to_account_info(),
                authority: self.state.to_account_info(),
            },
        )
    }

    fn take_base(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.account_base.to_account_info(),
                to: self.reserve_base.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    fn take_quote(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.account_quote.to_account_info(),
                to: self.reserve_quote.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }
}
