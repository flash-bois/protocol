use crate::{
    core_lib::{
        decimal::{DecimalPlaces, Price},
        errors::LibErrors,
        Token,
    },
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use checked_decimal_macro::Factories;

#[derive(Accounts)]
pub struct EnableOracle<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,
    /// CHECK: deserialized in code for now
    pub price_feed: AccountInfo<'info>,
}

impl EnableOracle<'_> {
    pub fn handler(
        &mut self,
        index: u8,
        decimals: u8,
        base: bool,
        skip_init: bool,
        max_update_interval: u32,
    ) -> anchor_lang::Result<()> {
        msg!(
            "DotWave: Enabling {} oracle",
            if base { "base" } else { "quote" }
        );

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(index)?;

        let decimal_places = match decimals {
            6 => DecimalPlaces::Six,
            9 => DecimalPlaces::Nine,
            _ => return Err(LibErrors::InvalidDecimalPlaces.into()),
        };

        let current_timestamp = Clock::get()?.unix_timestamp;
        let our_current_timestamp = current_timestamp
            .try_into()
            .map_err(|_| LibErrors::ParseError)?;

        vault.enable_oracle(
            decimal_places,
            Price::from_integer(0),
            Price::from_integer(0),
            Price::from_scale(2, 2),
            our_current_timestamp,
            if base { Token::Base } else { Token::Quote },
            max_update_interval,
        )?;

        if !skip_init {
            let oracle = if base {
                vault.oracle_mut()?
            } else {
                vault.quote_oracle_mut()?
            };

            oracle.update_from_acc(&self.price_feed, current_timestamp)?;
        }

        let keys = vaults.keys_checked_mut(index)?;

        if base {
            keys.base_oracle = Some(self.price_feed.key());
        } else {
            keys.quote_oracle = Some(self.price_feed.key());
        }

        Ok(())
    }
}
