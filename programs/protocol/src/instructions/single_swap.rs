use std::collections::HashSet;

use crate::{
    core_lib::decimal::Quantity,
    core_lib::errors::LibErrors,
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, transfer, TokenAccount, Transfer};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct SingleSwap<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,
        constraint = account_base.mint == reserve_base.mint,
        constraint = account_base.owner == signer.key(),
    )]
    account_base: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = account_quote.mint == reserve_quote.mint,
        constraint = account_quote.owner == signer.key(),
    )]
    account_quote: Account<'info, TokenAccount>,
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

impl<'info> SingleSwap<'info> {
    pub fn handler(
        ctx: Context<SingleSwap<'info>>,
        vault: u8,
        amount: u64,
        min_expected: u64,
        from_base: bool,
        by_amount_out: bool,
    ) -> anchor_lang::Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let vaults = &mut ctx.accounts.vaults.load_mut()?;

        let mut vaults_indexes = HashSet::new();
        vaults_indexes.insert(vault);
        vaults.refresh(&vaults_indexes, ctx.remaining_accounts, current_timestamp)?;

        let vault = vaults.vault_checked_mut(vault)?;
        let quantity = Quantity::new(amount);

        if by_amount_out {
            unimplemented!("swaps by amount out are not yet implemented")
        }

        let quantity_out = match from_base {
            true => vault.sell(quantity)?,
            false => vault.buy(quantity)?,
        };

        msg!("quantity out: {}", quantity_out);

        if quantity_out < Quantity::new(min_expected) {
            return Err(LibErrors::NoMinAmountOut.into());
        }

        let seeds = &[b"state".as_ref(), &[ctx.accounts.state.load()?.bump]];
        let signer = &[&seeds[..]];

        let (take, send) = if from_base {
            (ctx.accounts.take_base(), ctx.accounts.send_quote())
        } else {
            (ctx.accounts.take_quote(), ctx.accounts.send_base())
        };

        transfer(take, amount)?;
        transfer(send.with_signer(signer), quantity_out.get())?;

        Ok(())
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
}
