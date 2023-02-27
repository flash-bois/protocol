use crate::{
    core_lib::errors::LibErrors,
    core_lib::{services::Services, strategy::Strategies, vault::Vault},
    structs::{State, VaultKeys, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,

    #[account(init,
        token::mint = base,
        token::authority = state.to_account_info(),
        payer = admin,
    )]
    pub reserve_base: Account<'info, TokenAccount>,
    #[account(init,
        token::mint = quote,
        token::authority = state.to_account_info(),
        payer = admin,
    )]
    pub reserve_quote: Account<'info, TokenAccount>,
    pub base: Account<'info, Mint>,
    pub quote: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl InitVault<'_> {
    pub fn handler(&mut self) -> anchor_lang::Result<()> {
        msg!("DotWave: Initializing vault");

        let keys = VaultKeys {
            base_token: self.base.key(),
            quote_token: self.quote.key(),
            base_reserve: self.reserve_base.key(),
            quote_reserve: self.reserve_quote.key(),
            base_oracle: None,
            quote_oracle: None,
        };

        let vaults = &mut self.vaults.load_mut()?;
        let created_vault = Vault {
            services: Services::default(),
            strategies: Strategies::default(),
            oracle: None,
            quote_oracle: None,
            id: vaults.arr.head,
        };

        vaults
            .arr
            .add(created_vault)
            .map_err(|_| LibErrors::AddVault)?;
        vaults.keys.add(keys).map_err(|_| LibErrors::AddKeys)?;

        Ok(())
    }
}
