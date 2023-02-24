use crate::{
    core_lib::{
        services::Services,
        strategy::Strategies,
        structs::Oracle,
        vault::{self, Vault},
    },
    structs::{State, VaultKeys, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,

    #[account(mut,
        constraint = reserve_base.mint == base.key(),
        constraint = reserve_base.owner == state.key(),
    )]
    pub reserve_base: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = reserve_quote.mint == quote.key(),
        constraint = reserve_quote.owner == state.key(),
    )]
    pub reserve_quote: Account<'info, TokenAccount>,
    pub base: Account<'info, Mint>,
    pub quote: Account<'info, Mint>,
}

impl InitVault<'_> {
    pub fn handler(self) -> Result<()> {
        msg!("DotWave: Initializing vault");
        let keys = VaultKeys {
            base_token: self.base.key(),
            quote_token: self.quote.key(),
            base_reserve: self.reserve_base.key(),
            quote_reserve: self.reserve_quote.key(),
        };

        let vaults = &mut self.vaults.load_mut()?;
        let created_vault = Vault {
            services: Services::default(),
            strategies: Strategies::default(),
            oracle: Oracle::default(),
            quote_oracle: Oracle::default(),
            id: vaults.arr.head,
        };
        vaults.arr.add(created_vault);
        vaults.keys.add(keys);

        Ok(())
    }
}
