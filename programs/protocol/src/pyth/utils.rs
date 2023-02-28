use crate::core_lib::{decimal::Price, errors::LibErrors, structs::oracle::DEFAULT_MAX_ORACLE_AGE};
use anchor_lang::prelude::*;
use checked_decimal_macro::Decimal;
use pyth_sdk_solana::load_price_feed_from_account_info;

pub struct OracleUpdate {
    pub price: Price,
    pub confidence: Price,
}

pub fn get_oracle_update_data(
    acc_info: &AccountInfo,
    current_timestamp: i64,
) -> std::result::Result<OracleUpdate, LibErrors> {
    let price_feed =
        load_price_feed_from_account_info(acc_info).map_err(|_| LibErrors::PythAccountParse)?;

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

    Ok(OracleUpdate { price, confidence })
}
