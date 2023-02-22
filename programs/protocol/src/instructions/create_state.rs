use crate::structs::{State, Vaults};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

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
    /// CHECK:
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

impl<'info> CreateState<'info> {
    pub fn handler(&self, nonce: u8, bump: u8) -> Result<()> {
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
