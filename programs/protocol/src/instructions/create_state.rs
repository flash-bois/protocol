use crate::structs::{State, Vaults};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction( nonce: u8 )]
pub struct CreateState<'info> {
    #[account(init, seeds = [b"state".as_ref()], bump, payer = admin, space = 8 + 97)]
    pub state: AccountLoader<'info, State>,
    #[account(seeds = [b"DotWave".as_ref()], bump = nonce)]
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(zero)]
    pub vaults: AccountLoader<'info, Vaults>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: SystemAccount<'info>,
}

impl<'info> CreateState<'info> {
    pub fn handler(&self, bump: u8, nonce: u8) -> Result<()> {
        let state = &mut self.state.load_init()?;
        let vaults = &mut self.vaults.load_init()?;

        msg!("Initializing state");

        **state = State {
            authority: *self.program_authority.key,
            admin: *self.admin.key,
            vaults: *self.vaults.to_account_info().key,
            nonce,
            bump,
        };

        msg!("Initializing vaults");

        **vaults = Vaults {
            ..Default::default()
        };

        Ok(())
    }
}
