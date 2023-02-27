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
        #[msg("Price confidence is higher than price")]
        ConfidenceHigherThanPrice,
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
    }
}

pub use anchor_one::*;

#[cfg(not(feature = "anchor"))]
mod anchor_none {
    #[derive(Debug, PartialEq)]
    pub enum LibErrors {
        ToBeDefined,
        NotEnoughQuoteQuantity,
        NotEnoughBaseQuantity,
        UserAllowedBorrowExceeded,
        LendServiceNone,
        SwapServiceNone,
        OracleNone,
        QuoteOracleNone,
        OracleAlreadyEnabled,
        ConfidenceHigherThanPrice,
        StrategyNoLend,
        StrategyNoSwap,
        StrategyNoTrade,
        StrategyMissing,
        CannotBorrow,
        RepayLowerThanFee,
        CannotAddStrategy,
        CannotAddPosition,
    }
}
