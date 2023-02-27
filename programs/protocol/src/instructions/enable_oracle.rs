use crate::{
    core_lib::{
        decimal::{DecimalPlaces, Price},
        errors::LibErrors,
        structs::oracle::DEFAULT_MAX_ORACLE_AGE,
        Token,
    },
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use checked_decimal_macro::{Decimal, Factories};
use pyth_sdk_solana::load_price_feed_from_account_info;

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
        )?;

        if !skip_init {
            let price_feed = load_price_feed_from_account_info(&self.price_feed)
                .map_err(|_| LibErrors::PythAccountParse)?;

            let current_price = price_feed
                .get_price_no_older_than(current_timestamp, DEFAULT_MAX_ORACLE_AGE.into())
                .ok_or(LibErrors::PythPriceGet)?;

            let price = Price::new(
                current_price
                    .price
                    .try_into()
                    .map_err(|_| LibErrors::ParseError)?,
            );

            let confidence = Price::new(
                current_price
                    .conf
                    .try_into()
                    .map_err(|_| LibErrors::ParseError)?,
            );

            vault
                .oracle_mut()?
                .update(price, confidence, our_current_timestamp)?;
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
