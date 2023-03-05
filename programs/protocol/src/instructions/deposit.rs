use crate::{
    core_lib::{decimal::Quantity, Token},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, transfer, TokenAccount, Transfer};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, seeds = [b"statement".as_ref(), signer.key.as_ref()], bump=statement.load()?.bump, 
    constraint = statement.load()?.owner == signer.key())]
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

impl Deposit<'_> {
    pub fn handler(
        &mut self,
        vault: u8,
        strategy: u8,
        quantity: u64,
        base: bool,
    ) -> anchor_lang::Result<()> {
        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(vault)?;
        let statement = &mut self.statement.load_mut()?;
        let user_statement = &mut statement.statement;
 
        let other_quantity = vault.deposit(
            user_statement,
            if base { Token::Base } else { Token::Quote },
            Quantity::new(quantity),
            strategy,
            Clock::get()?.unix_timestamp as u32,
        )?;

        msg!("{}", user_statement.positions.head);

        let (base_amount, quote_amount) = if base {
            (quantity, other_quantity.get())
        } else {
            (other_quantity.get(), quantity)
        };

        let take_base_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.account_base.to_account_info(),
                to: self.reserve_base.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        let take_quote_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.account_quote.to_account_info(),
                to: self.reserve_quote.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        msg!("{:?}", statement.statement.positions.head);

        transfer(take_base_ctx, base_amount)?;
        transfer(take_quote_ctx, quote_amount)?;

        Ok(())
    }
}
