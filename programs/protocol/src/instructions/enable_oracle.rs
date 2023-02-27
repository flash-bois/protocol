use crate::core_lib::decimal::{DecimalPlaces, Price};
use crate::core_lib::Token;
use crate::errors::NoLibErrors;
use crate::structs::{State, Vaults};
use anchor_lang::prelude::*;
use checked_decimal_macro::Factories;

#[derive(Accounts)]
pub struct EnableOracle<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
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
    ) -> anchor_lang::Result<()> {
        msg!(
            "DotWave: Enabling {} oracle",
            if base { "base" } else { "quote" }
        );

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults
            .arr
            .get_mut(index as usize)
            .ok_or(NoLibErrors::NoVaultOnIndex)?;

        let decimal_places = match decimals {
            6 => DecimalPlaces::Six,
            9 => DecimalPlaces::Nine,
            _ => return Err(NoLibErrors::InvalidDecimalPlaces.into()),
        };

        if skip_init {
            vault.enable_oracle(
                decimal_places,
                Price::from_integer(0),
                Price::from_integer(0),
                Price::from_scale(2, 2),
                Clock::get()?
                    .unix_timestamp
                    .try_into()
                    .map_err(|_| NoLibErrors::ParseError)?,
                if base { Token::Base } else { Token::Quote },
            )?
        } else {
            // TODO parse price on init
            unimplemented!();
            // let price_feed: PriceFeed = load_price_feed_from_account_info(&self.price_feed).unwrap();
            // let current_timestamp = Clock::get()?.unix_timestamp;
            // let current_price = price_feed
            //     .get_price_no_older_than(current_timestamp, DEFAULT_MAX_ORACLE_AGE.into())
            //     .unwrap();
        }

        let keys = vaults
            .keys
            .get_mut(index as usize)
            .ok_or(NoLibErrors::NoVaultOnIndex)?;

        if base {
            keys.base_oracle = Some(self.price_feed.key());
        } else {
            keys.quote_oracle = Some(self.price_feed.key());
        }

        Ok(())
    }
}
