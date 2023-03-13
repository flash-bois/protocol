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
#[instruction(vault_in: u8, vault_out: u8)]
pub struct DoubleSwap<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,
        constraint = account_in.mint == reserve_in.mint,
        constraint = account_in.owner == signer.key(),
    )]
    account_in: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = account_out.mint == reserve_out.mint,
        constraint = account_out.owner == signer.key(),
    )]
    account_out: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = reserve_in.mint == vaults.load()?.keys.get(vault_in as usize).unwrap().base_token,
        constraint = reserve_in.key() == vaults.load()?.keys.get(vault_in as usize).unwrap().base_reserve,
        constraint = reserve_in.owner == state.key(),
    )]
    pub reserve_in: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        constraint = reserve_out.mint == vaults.load()?.keys.get(vault_out as usize).unwrap().base_token,
        constraint = reserve_out.key() == vaults.load()?.keys.get(vault_out as usize).unwrap().base_reserve,
        constraint = reserve_out.owner == state.key(),
    )]
    pub reserve_out: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        constraint = reserve_in_quote.mint == vaults.load()?.keys.get(vault_in as usize).unwrap().quote_token,
        constraint = reserve_in_quote.key() == vaults.load()?.keys.get(vault_in as usize).unwrap().quote_reserve,
        constraint = reserve_in_quote.owner == state.key(),
    )]
    pub reserve_in_quote: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        constraint = reserve_out_quote.mint == vaults.load()?.keys.get(vault_out as usize).unwrap().quote_token,
        constraint = reserve_out_quote.mint == vaults.load()?.keys.get(vault_in as usize).unwrap().quote_token,
        constraint = reserve_out_quote.key() == vaults.load()?.keys.get(vault_out as usize).unwrap().quote_reserve,
        constraint = reserve_out_quote.owner == state.key(),
    )]
    pub reserve_out_quote: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, token::Token>,
}

impl<'info> DoubleSwap<'info> {
    pub fn handler(
        ctx: Context<DoubleSwap<'info>>,
        vault_in: u8,
        vault_out: u8,
        amount: u64,
        min_expected: u64,
        by_amount_out: bool,
    ) -> anchor_lang::Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let vaults = &mut ctx.accounts.vaults.load_mut()?;
        let quantity = Quantity::new(amount);

        if by_amount_out {
            unimplemented!("swaps by amount out are not yet implemented")
        }

        let mut vaults_indexes = HashSet::new();
        vaults_indexes.insert(vault_in);
        vaults_indexes.insert(vault_out);
        vaults.refresh(&vaults_indexes, ctx.remaining_accounts, current_timestamp)?;

        let vault_in = vaults.vault_checked_mut(vault_in)?;
        let quote_quantity = vault_in.sell(quantity)?;
        msg!("quantity quote: {}", quote_quantity);

        let vault_out = vaults.vault_checked_mut(vault_out)?;
        let quantity_out = vault_out.buy(quote_quantity)?;
        msg!("quantity out: {}", quantity_out);

        if quantity_out < Quantity::new(min_expected) {
            return Err(LibErrors::NoMinAmountOut.into());
        }

        let seeds = &[b"state".as_ref(), &[ctx.accounts.state.load()?.bump]];
        let signer = &[&seeds[..]];

        transfer(ctx.accounts.take_in(), amount)?;
        transfer(
            ctx.accounts.move_quote().with_signer(signer),
            quote_quantity.get(),
        )?;
        transfer(
            ctx.accounts.send_out().with_signer(signer),
            quantity_out.get(),
        )?;

        Ok(())
    }

    fn take_in(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.account_in.to_account_info(),
                to: self.reserve_in.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    fn send_out(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_out.to_account_info(),
                to: self.account_out.to_account_info(),
                authority: self.state.to_account_info(),
            },
        )
    }

    fn move_quote(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve_in_quote.to_account_info(),
                to: self.reserve_out_quote.to_account_info(),
                authority: self.state.to_account_info(),
            },
        )
    }
}
