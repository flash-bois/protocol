use crate::core_lib::{errors::LibErrors, structs::oracle::DEFAULT_MAX_ORACLE_AGE};
use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, Price};

pub struct OracleUpdate {
    pub price: i64,
    pub conf: u64,
    pub exp: i32,
}

pub fn get_oracle_update_from_acc(
    acc: &AccountInfo,
    current_timestamp: i64,
) -> std::result::Result<OracleUpdate, LibErrors> {
    let price_feed =
        load_price_feed_from_account_info(acc).map_err(|_| LibErrors::PythAccountParse)?;

    let Price {
        price,
        conf,
        expo: exp,
        ..
    } = price_feed
        .get_price_no_older_than(current_timestamp, DEFAULT_MAX_ORACLE_AGE.into())
        .ok_or(LibErrors::PythPriceGet)?;

    Ok(OracleUpdate { price, conf, exp })
}
