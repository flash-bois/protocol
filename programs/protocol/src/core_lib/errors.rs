#[cfg(feature = "anchor")]
mod anchor_one {
    use anchor_lang::error_code;

    #[error_code]
    #[derive(PartialEq)]
    pub enum LibErrors {
        #[msg("Too large data")]
        DataTooLarge,
        #[msg("To be defined")]
        ToBeDefined,
        #[msg("Not enough available quote quantity")]
        NotEnoughQuoteQuantity,
        #[msg("Not enough available base quantity")]
        NotEnoughBaseQuantity,
        #[msg("Borrow value is higher than user max allowed borrow")]
        UserAllowedBorrowExceeded,
        #[msg("Services does not have lend")]
        LendServiceNone,
        #[msg("Services does not have swap")]
        SwapServiceNone,
        #[msg("Vault does not contain base oracle")]
        OracleNone,
        #[msg("Vault does not contain quote oracle")]
        QuoteOracleNone,
        #[msg("Given oracle was enabled before")]
        OracleAlreadyEnabled,
        #[msg("Price confidence is higher than spread limit")]
        ConfidenceTooHigh,
        #[msg("Strategy does not provide to lend")]
        StrategyNoLend,
        #[msg("Strategy does not provide to swap")]
        StrategyNoSwap,
        #[msg("Strategy does not provide to trade")]
        StrategyNoTrade,
        #[msg("There is no strategy on given index in strategies array")]
        StrategyMissing,
        #[msg("Cannot borrow due to high utilization or max borrow limit")]
        CannotBorrow,
        #[msg("Given repay amount is lower than accrued fee")]
        RepayLowerThanFee,
        #[msg("Cannot add strategy (array limit exceeded)")]
        CannotAddStrategy,
        #[msg("Cannot add user position (array limit exceeded)")]
        CannotAddPosition,
        #[msg("There is no defined vault on provided index")]
        NoVaultOnIndex,
        #[msg("Parsing between types was not successful")]
        ParseError,
        #[msg("Bump for given name not found in BSTree")]
        BumpNotFound,
        #[msg("Given decimal places are not expected")]
        InvalidDecimalPlaces,
        #[msg("Failed to add vault to vaults array")]
        AddVault,
        #[msg("Failed to add vault keys to vaults keys array")]
        AddKeys,
        #[msg("Amount out did not reached passed minimum")]
        NoMinAmountOut,
        #[msg("Provided index is out of bounds")]
        IndexOutOfBounds,
        #[msg("There is no defined strategy on provided index")]
        NoStrategyOnIndex,
        #[msg("Service is not valid")]
        InvalidService,
        #[msg("Parse price account error")]
        PythAccountParse,
        #[msg("Cannot get price within DEFAULT_MAX_ORACLE_AGE")]
        PythPriceGet,
        #[msg("Cannot find desired oracle account in remaining account infos")]
        OracleAccountNotFound,
        #[msg("Array is empty")]
        ArrayEmpty,
        #[msg("Cannot get time")]
        TimeGet,
        #[msg("pubkey should be defined")]
        PubkeyMissing,
        #[msg("Given position was not found")]
        PositionNotFound,
    }
}

#[cfg(feature = "anchor")]
pub use anchor_one::*;

#[cfg(not(feature = "anchor"))]
mod anchor_none {
    use thiserror::Error;
    use wasm_bindgen::JsValue;

    #[derive(Debug, Error, PartialEq)]
    pub enum LibErrors {
        #[error("Too large data")]
        DataTooLarge,
        #[error("To be defined")]
        ToBeDefined,
        #[error("Not enough available quote quantity")]
        NotEnoughQuoteQuantity,
        #[error("Not enough available base quantity")]
        NotEnoughBaseQuantity,
        #[error("Borrow value is higher than user max allowed borrow")]
        UserAllowedBorrowExceeded,
        #[error("Services does not have lend")]
        LendServiceNone,
        #[error("Services does not have swap")]
        SwapServiceNone,
        #[error("Vault does not contain base oracle")]
        OracleNone,
        #[error("Vault does not contain quote oracle")]
        QuoteOracleNone,
        #[error("Given oracle was enabled before")]
        OracleAlreadyEnabled,
        #[error("Price confidence is higher than spread limit")]
        ConfidenceTooHigh,
        #[error("Strategy does not provide to lend")]
        StrategyNoLend,
        #[error("Strategy does not provide to swap")]
        StrategyNoSwap,
        #[error("Strategy does not provide to trade")]
        StrategyNoTrade,
        #[error("There is no strategy on given index in strategies array")]
        StrategyMissing,
        #[error("Cannot borrow due to high utilization or max borrow limit")]
        CannotBorrow,
        #[error("Given repay amount is lower than accrued fee")]
        RepayLowerThanFee,
        #[error("Cannot add strategy (array limit exceeded)")]
        CannotAddStrategy,
        #[error("Cannot add user position (array limit exceeded)")]
        CannotAddPosition,
        #[error("There is no defined vault on provided index")]
        NoVaultOnIndex,
        #[error("Parsing between types was not successful")]
        ParseError,
        #[error("Bump for given name not found in BSTree")]
        BumpNotFound,
        #[error("Given decimal places are not expected")]
        InvalidDecimalPlaces,
        #[error("Failed to add vault to vaults array")]
        AddVault,
        #[error("Failed to add vault keys to vaults keys array")]
        AddKeys,
        #[error("Amount out did not reached passed minimum")]
        NoMinAmountOut,
        #[error("Provided index is out of bounds")]
        IndexOutOfBounds,
        #[error("There is no defined strategy on provided index")]
        NoStrategyOnIndex,
        #[error("Service is not valid")]
        InvalidService,
        #[error("Parse price account error")]
        PythAccountParse,
        #[error("Cannot get price within DEFAULT_MAX_ORACLE_AGE")]
        PythPriceGet,
        #[error("Array is empty")]
        ArrayEmpty,
        #[error("Cannot get time")]
        TimeGet,
        #[error("pubkey should be defined")]
        PubkeyMissing,
        #[error("Given position was not found")]
        PositionNotFound,
    }

    impl From<LibErrors> for JsValue {
        fn from(value: LibErrors) -> Self {
            let val = format!("{}:{}", value.to_string(), (value as u32).to_string());
            JsValue::from(val)
        }
    }
}

#[cfg(not(feature = "anchor"))]
pub use anchor_none::*;
