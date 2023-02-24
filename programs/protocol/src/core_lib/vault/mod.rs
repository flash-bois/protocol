pub mod deposit;
pub mod general;
pub mod lend;
pub mod swap;
pub mod test;

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
        pub oracle: Oracle,
        pub quote_oracle: Oracle,
        pub id: u8,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    #[repr(C)]
    pub struct Vault {
        pub id: u8,

        pub oracle: Oracle,
        pub quote_oracle: Oracle,

        pub services: Services,
        pub strategies: Strategies,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Vault;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Vault;

impl Vault {
    pub fn add_strategy(
        &mut self,
        has_lend: bool,
        has_swap: bool,
        has_trade: bool,
    ) -> Result<(), ()> {
        if has_lend && self.services.lend_mut().is_ok() {
            return Err(());
        }
        if has_swap && self.services.swap_mut().is_ok() {
            return Err(());
        }

        // if has_trade && self.services.trade.is_none() {
        //     return Err(());
        // }

        self.strategies
            .add(Strategy::new(has_lend, has_swap, has_trade))
    }

    pub fn enable_oracle(
        &mut self,
        decimal_places: DecimalPlaces,
        price: Price,
        confidence: Price,
        spread_limit: Price,
        time: Time,
        for_token: Token,
    ) -> Result<(), ()> {
        // if match for_token {
        //     Token::Base => self.oracle.is_some(),
        //     Token::Quote => self.quote_oracle.is_some(),
        // } {
        //     return Err(());
        // }

        let oracle = Oracle::new(decimal_places, price, confidence, spread_limit, time)?;

        match for_token {
            Token::Base => self.oracle = oracle,
            Token::Quote => self.quote_oracle = oracle,
        }

        Ok(())
    }
    pub fn quote_oracle(&self) -> Result<&Oracle, ()> {
        // self.quote_oracle.as_ref().ok_or(())
        Ok(&self.quote_oracle)
    }

    pub fn oracle(&self) -> Result<&Oracle, ()> {
        // self.oracle.as_ref().ok_or(())
        Ok(&self.oracle)
    }

    pub fn quote_oracle_mut(&mut self) -> Result<&mut Oracle, ()> {
        // self.quote_oracle.as_mut().ok_or(())
        Ok(&mut self.quote_oracle)
    }

    pub fn oracle_mut(&mut self) -> Result<&mut Oracle, ()> {
        // self.oracle.as_mut().ok_or(())
        Ok(&mut self.oracle)
    }

    pub fn enable_lending(
        &mut self,
        lending_fee: FeeCurve,
        max_utilization: Utilization,
        borrow_limit: Quantity,
        initial_fee_time: Time,
    ) -> Result<(), ()> {
        if self.services.lend_mut().is_err() {
            return Err(());
        }

        // if self.oracle.is_none() {
        //     return Err(());
        // }

        self.services.lend =
            Lend::new(lending_fee, max_utilization, borrow_limit, initial_fee_time);

        Ok(())
    }

    pub fn enable_swapping(
        &mut self,
        selling_fee: FeeCurve,
        buying_fee: FeeCurve,
        kept_fee: Fraction,
    ) -> Result<(), ()> {
        // if self.oracle.is_none() {
        //     return Err(());
        // }
        if self.services.swap_mut().is_ok() {
            return Err(());
        }

        self.services.swap = Swap::new(selling_fee, buying_fee, kept_fee);

        Ok(())
    }

    pub fn lend_service(&mut self) -> Result<&mut Lend, ()> {
        Ok(&mut self.services.lend)
    }

    pub fn swap_service(&mut self) -> Result<&mut Swap, ()> {
        Ok(&mut self.services.swap)
    }

    fn settle_fees(&mut self, service: ServiceType) -> Result<(), ()> {
        let (service_updatable, service_accrued_fees): (&mut dyn ServiceUpdate, Quantity) =
            match service {
                ServiceType::Lend => {
                    let service = self.lend_service()?;
                    let fees = service.accrue_fee();
                    (service, fees.base)
                }
                _ => unimplemented!(),
            };

        let locked = service_updatable.locked().base;
        let service_locked_global = locked - service_accrued_fees;

        if service_accrued_fees.is_zero() {
            return Ok(());
        }

        let mut distributed_so_far = Quantity::new(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self.strategies.get_mut_checked(i).ok_or(())?;
            if strategy.uses(service) {
                last_index = i;

                let sub_strategy_locked = strategy.locked_by(service)?;

                let to_distribute =
                    service_accrued_fees.big_mul_div(sub_strategy_locked, service_locked_global);

                distributed_so_far += to_distribute;
                strategy.accrue_fee(to_distribute, service)?;
            }
        }

        if distributed_so_far < service_accrued_fees {
            let strategy = self.strategies.get_mut_checked(last_index).unwrap();
            strategy.accrue_fee(service_accrued_fees - distributed_so_far, service)?;
        }

        Ok(())
    }

    pub fn refresh(&mut self, current_time: Time) -> Result<(), ()> {
        // TODO check oracle if it is refreshed

        let service = &mut self.services.lend;

        service.accrue_interest_rate(current_time);
        self.settle_fees(ServiceType::Lend)?;

        // if self.services.swap.is_some() {
        //     self.settle_fees(ServiceType::Swap)?
        // }

        Ok(())
    }
}

#[cfg(test)]
impl Vault {
    pub fn new_vault_for_tests() -> Result<Self, ()> {
        let mut vault = Self::default();
        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(5, 3),
            Price::from_scale(1, 2),
            0,
            Token::Base,
        )?;
        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(2, 3),
            0,
            Token::Quote,
        )?;
        vault.enable_lending(
            FeeCurve::default(),
            Utilization::from_integer(1),
            Quantity::new(u64::MAX),
            0,
        )?;
        vault.enable_swapping(
            FeeCurve::default(),
            FeeCurve::default(),
            Fraction::from_scale(1, 1),
        )?;
        vault.add_strategy(true, true, false)?;
        Ok(vault)
    }
}
