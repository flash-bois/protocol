use crate::structs::State;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction( nonce: u8 )]
pub struct CreateState<'info> {
    // space = 8 + 14787
    #[account(zero)]
    pub state: AccountLoader<'info, State>,
    #[account(seeds = [b"DotWave".as_ref()], bump = nonce)]
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: SystemAccount<'info>,
}

pub fn handler(ctx: Context<CreateState>, nonce: u8) -> Result<()> {
    let state = &mut ctx.accounts.state.load_init()?;

    **state = State {
        authority: *ctx.accounts.program_authority.key,
        admin: *ctx.accounts.admin.key,
        nonce,
        ..Default::default()
    };

    Ok(())
}
