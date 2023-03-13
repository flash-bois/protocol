use crate::{
    core_lib::{decimal::Quantity, errors::LibErrors, Token},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, transfer, TokenAccount, Transfer};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct Withdraw<'info> {
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

impl<'info> Withdraw<'info> {
    pub fn handler(
        ctx: Context<Withdraw>,
        vault: u8,
        strategy: u8,
        quantity: u64,
        base: bool,
    ) -> anchor_lang::Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let vaults = &mut ctx.accounts.vaults.load_mut()?;
        let statement = &mut ctx.accounts.statement.load_mut()?.statement;

        let vaults_indexes = statement.get_vaults_indexes(&vault);
        vaults.refresh(&vaults_indexes, ctx.remaining_accounts, current_timestamp)?;

        let vault = vaults.vault_checked_mut(vault)?;

        let (base_amount, quote_amount) = vault.withdraw(
            statement,
            if base { Token::Base } else { Token::Quote },
            Quantity::new(quantity),
            strategy,
        )?;

        statement.refresh(&vaults.arr.elements)?;

        if !statement.collateralized() {
            return Err(LibErrors::UserNotCollateralized.into());
        }

        let seeds = &[b"state".as_ref(), &[ctx.accounts.state.load()?.bump]];
        let signer = &[&seeds[..]];

        transfer(
            ctx.accounts.send_base().with_signer(signer),
            base_amount.get(),
        )?;
        transfer(
            ctx.accounts.send_quote().with_signer(signer),
            quote_amount.get(),
        )?;

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
}
