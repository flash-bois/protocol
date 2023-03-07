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

impl<'info> Deposit<'info> {
    pub fn handler(
        ctx: Context<Deposit>,
        vault: u8,
        strategy: u8,
        quantity: u64,
        base: bool,
    ) -> anchor_lang::Result<()> {
        let vaults = &mut ctx.accounts.vaults.load_mut()?;
        let statement = &mut ctx.accounts.statement.load_mut()?.statement;
        vaults.refresh(&[vault], ctx.remaining_accounts)?;

        let vault = vaults.vault_checked_mut(vault)?;

        let other_quantity = vault.deposit(
            statement,
            if base { Token::Base } else { Token::Quote },
            Quantity::new(quantity),
            strategy,
        )?;

        let (base_amount, quote_amount) = if base {
            (quantity, other_quantity.get())
        } else {
            (other_quantity.get(), quantity)
        };

        transfer(ctx.accounts.take_base(), base_amount)?;
        transfer(ctx.accounts.take_quote(), quote_amount)?;

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
}
