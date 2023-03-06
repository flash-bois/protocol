use crate::{
    core_lib::decimal::Quantity,
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, transfer, TokenAccount, Transfer};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct Repay<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = statement.load()?.owner == signer.key())]
    pub statement: AccountLoader<'info, Statement>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,
      constraint = account_base.mint == reserve_base.mint,
      constraint = account_base.owner == signer.key(),
  )]
    account_base: Account<'info, TokenAccount>,
    #[account(mut,
      constraint = reserve_base.mint == vaults.load()?.keys.get(vault as usize).unwrap().base_token,
      constraint = reserve_base.key() == vaults.load()?.keys.get(vault as usize).unwrap().base_reserve,
      constraint = reserve_base.owner == state.key(),
  )]
    pub reserve_base: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

impl<'info> Repay<'info> {
    pub fn handler(ctx: Context<Repay>, vault: u8, amount: u64) -> anchor_lang::Result<()> {
        msg!("DotWave: Repay");

        let vaults = &mut ctx.accounts.vaults.load_mut()?;
        let user_statement = &mut ctx.accounts.statement.load_mut()?;
        let amount = Quantity::new(amount);

        //vaults.refresh_all(ctx.remaining_accounts)?;
        user_statement.statement.refresh(&vaults.arr.elements)?;

        let vault = vaults.vault_checked_mut(vault)?;
        let repay_amount = vault.repay(&mut user_statement.statement, amount)?;

        transfer(ctx.accounts.take_base(), repay_amount.get())?;

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
}
