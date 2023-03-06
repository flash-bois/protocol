use crate::{core_lib::errors::LibErrors, structs::Statement};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateStatement<'info> {
    #[account(
      init,
      seeds = [b"statement", payer.key.as_ref()],
      bump,
      payer = payer,
      space = 8 + std::mem::size_of::<Statement>()
    )]
    pub statement: AccountLoader<'info, Statement>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateStatement>) -> Result<()> {
    msg!("DotWave: Initializing statement");

    let statement = &mut ctx.accounts.statement.load_init()?;

    statement.owner = *ctx.accounts.payer.key;
    statement.bump = *ctx.bumps.get("statement").ok_or(LibErrors::BumpNotFound)?;

    Ok(())
}
