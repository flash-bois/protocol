use crate::{structs::Statement, core_lib::user::UserStatement};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateStatement<'info> {
    #[account(
      init, 
      seeds = [b"statement".as_ref(), payer.key.as_ref()], 
      bump, 
      payer = payer, 
      space = 8 + 2914
    )]
    pub statement: AccountLoader<'info, Statement>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}



pub fn handler(ctx: Context<CreateStatement>) -> Result<()> {
    let statement = &mut ctx.accounts.statement.load_init()?;

    msg!("Initializing statement");

    let bump = *ctx.bumps.get("statement").unwrap();

    msg!("key: {}", ctx.accounts.payer.key.to_string());

    **statement = Statement {
      owner: *ctx.accounts.payer.key,
      bump,
      statement: UserStatement::default()
    };

    // msg!("{}", std::mem::size_of_val(&stat));

    Ok(())
}
