pub mod deposit;
pub mod general;
pub mod lend;
pub mod swap;
pub mod trade;

use crate::core_lib::{
    decimal::{DecimalPlaces, Fraction, Price, Quantity, Shares, Time, Utilization},
    services::{lending::Lend, swapping::Swap, ServiceType, ServiceUpdate, Services},
    strategy::{Strategies, Strategy},
    structs::{FeeCurve, Oracle},
};
use checked_decimal_macro::Decimal;

#[cfg(test)]
use checked_decimal_macro::Factories;

pub use self::deposit::Token;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq)]
    pub struct Vault {
        pub services: Services,
        pub strategies: Strategies,
        pub oracle: Option<Oracle>,
        pub quote_oracle: Option<Oracle>,
        pub id: u8,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    #[repr(C)]
    pub struct Vault {
        pub services: Services,
        pub strategies: Strategies,
        pub oracle: Option<Oracle>,
        pub quote_oracle: Option<Oracle>,
        pub id: u8,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Vault;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Vault;

use super::{errors::LibErrors, services::trading::Trade};

impl Vault {
    pub fn add_strategy(
        &mut self,
        has_lend: bool,
        has_swap: bool,
        has_trade: bool,
        collateral_ratio: Fraction,
        liquidation_threshold: Fraction,
    ) -> Result<(), LibErrors> {
        if has_lend && self.services.lend_mut().is_err() {
            return Err(LibErrors::LendServiceNone);
        }
        if has_swap && self.services.swap_mut().is_err() {
            return Err(LibErrors::SwapServiceNone);
        }

        // if has_trade && self.services.trade.is_none() {
        //     return Err(());
        // }

        self.strategies
            .add(Strategy::new(
                has_lend,
                has_swap,
                has_trade,
                collateral_ratio,
                liquidation_threshold,
            ))
            .map_err(|_| LibErrors::CannotAddStrategy)
    }

    pub fn enable_oracle(
        &mut self,
        decimal_places: DecimalPlaces,
        price: Price,
        confidence: Price,
        spread_limit: Price,
        time: Time,
        for_token: Token,
        max_update_interval: Time,
    ) -> Result<(), LibErrors> {
        if match for_token {
            Token::Base => self.oracle.is_some(),
            Token::Quote => self.quote_oracle.is_some(),
        } {
            return Err(LibErrors::OracleAlreadyEnabled);
        }

        let oracle = Some(Oracle::new(
            decimal_places,
            price,
            confidence,
            spread_limit,
            time,
            max_update_interval,
        ));

        match for_token {
            Token::Base => self.oracle = oracle,
            Token::Quote => self.quote_oracle = oracle,
        }

        Ok(())
    }
    pub fn quote_oracle(&self) -> Result<&Oracle, LibErrors> {
        self.quote_oracle.as_ref().ok_or(LibErrors::QuoteOracleNone)
    }

    pub fn oracle(&self) -> Result<&Oracle, LibErrors> {
        self.oracle.as_ref().ok_or(LibErrors::OracleNone)
    }

    pub fn quote_oracle_mut(&mut self) -> Result<&mut Oracle, LibErrors> {
        self.quote_oracle.as_mut().ok_or(LibErrors::QuoteOracleNone)
    }

    pub fn oracle_mut(&mut self) -> Result<&mut Oracle, LibErrors> {
        self.oracle.as_mut().ok_or(LibErrors::OracleNone)
    }

    pub fn enable_lending(
        &mut self,
        lending_fee: FeeCurve,
        max_utilization: Utilization,
        borrow_limit: Quantity,
        initial_fee_time: Time,
        last_fee_paid: Time,
    ) -> Result<(), LibErrors> {
        if self.oracle.is_none() {
            return Err(LibErrors::OracleNone);
        }

        if self.services.lend_mut().is_ok() {
            return Err(LibErrors::ServiceAlreadyExists);
        }

        self.services.lend = Some(Lend::new(
            lending_fee,
            max_utilization,
            borrow_limit,
            initial_fee_time,
            last_fee_paid,
        ));

        Ok(())
    }

    pub fn enable_trading(
        &mut self,
        open_fee: Fraction,
        max_leverage: Fraction,
        collateral_ratio: Fraction,
        liquidation_threshold: Fraction,
        start_time: Time,
    ) -> Result<(), LibErrors> {
        if self.oracle.is_none() {
            return Err(LibErrors::OracleNone);
        }

        if self.oracle.is_none() {
            return Err(LibErrors::QuoteOracleNone);
        }

        if self.services.trade_mut().is_ok() {
            return Err(LibErrors::ServiceAlreadyExists);
        }

        self.services.trade = Some(Trade::new(
            open_fee,
            max_leverage,
            start_time,
            collateral_ratio,
            liquidation_threshold,
        ));

        Ok(())
    }

    pub fn enable_swapping(
        &mut self,
        selling_fee: FeeCurve,
        buying_fee: FeeCurve,
        kept_fee: Fraction,
    ) -> Result<(), LibErrors> {
        if self.oracle.is_none() {
            return Err(LibErrors::OracleNone);
        }

        if self.oracle.is_none() {
            return Err(LibErrors::QuoteOracleNone);
        }

        if self.services.swap_mut().is_ok() {
            return Err(LibErrors::ServiceAlreadyExists);
        }

        self.services.swap = Some(Swap::new(selling_fee, buying_fee, kept_fee));

        Ok(())
    }

    pub fn trade_service(&mut self) -> Result<&mut Trade, LibErrors> {
        self.services.trade_mut()
    }

    pub fn swap_service(&mut self) -> Result<&mut Swap, LibErrors> {
        self.services.swap_mut()
    }

    pub fn lend_service(&mut self) -> Result<&mut Lend, LibErrors> {
        self.services.lend_mut()
    }

    pub fn trade_mut_and_oracles(&mut self) -> Result<(&mut Trade, &Oracle, &Oracle), LibErrors> {
        let Self { services, .. } = self;

        let oracle = self.oracle.as_ref().ok_or(LibErrors::OracleNone)?;
        let quote_oracle = self
            .quote_oracle
            .as_ref()
            .ok_or(LibErrors::QuoteOracleNone)?;

        Ok((services.trade_mut()?, oracle, quote_oracle))
    }

    pub fn lend_service_not_mut(&self) -> Result<&Lend, LibErrors> {
        self.services.lend()
    }

    pub fn swap_service_not_mut(&self) -> Result<&Swap, LibErrors> {
        self.services.swap()
    }

    pub fn trade_service_not_mut(&self) -> Result<&Trade, LibErrors> {
        self.services.trade()
    }

    pub fn refresh(&mut self, current_time: Time) -> Result<(), LibErrors> {
        if let Ok(lend) = self.lend_service() {
            lend.accrue_interest_rate(current_time);

            if lend.locked().base > Quantity::new(0) {
                // accrue_fee in lend also adds it to the borrowed
                let accrued_fees = lend.accrue_fee().base;

                let locked = lend.locked().base;

                //locked without accrued fees in previous call to accrue_fee
                let locked_without_current_fees = locked - accrued_fees;

                if accrued_fees.is_zero() {
                    return Ok(());
                }

                self.settle_lend_fees(
                    accrued_fees,
                    locked_without_current_fees,
                    ServiceType::Lend,
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
impl Vault {
    pub fn new_vault_for_tests() -> Result<Self, LibErrors> {
        let mut vault = Self::default();
        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(5, 3),
            Price::from_scale(1, 2),
            0,
            Token::Base,
            0,
        )?;
        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(2, 3),
            0,
            Token::Quote,
            0,
        )?;
        vault.enable_lending(
            FeeCurve::default(),
            Utilization::from_integer(1),
            Quantity::new(u64::MAX),
            0,
            0,
        )?;
        vault.enable_swapping(
            FeeCurve::default(),
            FeeCurve::default(),
            Fraction::from_scale(1, 1),
        )?;
        vault.add_strategy(
            true,
            true,
            false,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;
        Ok(vault)
    }
}
