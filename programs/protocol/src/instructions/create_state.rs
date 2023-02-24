use crate::structs::{State, Vaults};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateState<'info> {
    #[account(init, seeds = [b"state".as_ref()], bump, payer = admin, space = 8 + 65)]
    pub state: AccountLoader<'info, State>,
    #[account(zero)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateState>) -> Result<()> {
    let state = &mut ctx.accounts.state.load_init()?;

    msg!("Initializing state");

    **state = State {
        admin: ctx.accounts.admin.key(),
        vaults_acc: ctx.accounts.vaults.key(),
        bump: *ctx.bumps.get("state").unwrap(),
    };

    ctx.accounts.vaults.load_init()?;

    Ok(())
}
