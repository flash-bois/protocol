use crate::{
    core_lib::{decimal::Quantity, structs::Side},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct OpenPosition<'info> {
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

impl<'info> OpenPosition<'info> {
    pub fn handler(&mut self, vault: u8, amount: u64, long: bool) -> anchor_lang::Result<()> {
        let user_statement = &mut self.statement.load_mut()?.statement;
        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(vault)?;

        let current_timestamp = Clock::get()?.unix_timestamp as u32;
        let quantity = Quantity::new(amount);
        let side = if long { Side::Long } else { Side::Short };

        vault.open_position(user_statement, quantity, side, current_timestamp)?;

        Ok(())
    }
}
