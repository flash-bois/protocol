pub mod deposit;
pub mod general;
pub mod lend;
pub mod swap;
pub mod trade;

use crate::{
    decimal::{DecimalPlaces, Fraction, Price, Quantity, Shares, Time, Utilization, Value},
    services::{
        lending::Lend, swapping::Swap, trading::Trade, ServiceType, ServiceUpdate, Services,
    },
    strategy::{Strategies, Strategy},
    structs::{FeeCurve, Oracle},
};

#[cfg(test)]
use checked_decimal_macro::Factories;

pub use self::deposit::Token;

#[derive(Clone, Debug, Default)]
pub struct Vault {
    pub id: u8,

    oracle: Option<Oracle>,
    quote_oracle: Option<Oracle>,

    pub services: Services,
    pub strategies: Strategies,
}

impl Vault {
    pub fn add_strategy(
        &mut self,
        has_lend: bool,
        has_swap: bool,
        has_trade: bool,
    ) -> Result<(), ()> {
        if has_lend && self.services.lend.is_none() {
            return Err(());
        }
        if has_swap && self.services.swap.is_none() {
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
        if match for_token {
            Token::Base => self.oracle.is_some(),
            Token::Quote => self.quote_oracle.is_some(),
        } {
            return Err(());
        }

        let oracle = Some(Oracle::new(
            decimal_places,
            price,
            confidence,
            spread_limit,
            time,
        )?);

        match for_token {
            Token::Base => self.oracle = oracle,
            Token::Quote => self.quote_oracle = oracle,
        }

        Ok(())
    }

    pub fn enable_lending(
        &mut self,
        lending_fee: FeeCurve,
        max_utilization: Utilization,
        borrow_limit: Quantity,
        initial_fee_time: Time,
    ) -> Result<(), ()> {
        if self.services.lend.is_some() {
            return Err(());
        }

        if self.oracle.is_none() {
            return Err(());
        }

        self.services.lend = Some(Lend::new(
            lending_fee,
            max_utilization,
            borrow_limit,
            initial_fee_time,
        ));

        Ok(())
    }

    pub fn enable_swapping(
        &mut self,
        selling_fee: FeeCurve,
        buying_fee: FeeCurve,
        kept_fee: Fraction,
    ) -> Result<(), ()> {
        if self.oracle.is_none() {
            return Err(());
        }
        if self.services.swap.is_some() {
            return Err(());
        }

        self.services.swap = Some(Swap::new(selling_fee, buying_fee, kept_fee));

        Ok(())
    }

    pub fn lend_service(&mut self) -> Result<&mut Lend, ()> {
        self.services.lend_mut()
    }

    pub fn swap_service(&mut self) -> Result<&mut Swap, ()> {
        self.services.swap_mut()
    }

    pub fn trade_service(&mut self) -> Result<&mut Trade, ()> {
        self.services.trade_mut()
    }

    pub fn trade_and_oracles(&mut self) -> Result<(&mut Trade, &Oracle, &Oracle), ()> {
        let Self {
            services,
            oracle,
            quote_oracle,
            ..
        } = self;

        let oracle = self.oracle.as_ref().ok_or(())?;
        let quote_oracle = self.quote_oracle.as_ref().ok_or(())?;

        Ok((services.trade_mut()?, oracle, quote_oracle))
    }

    pub fn quote_oracle(&self) -> Result<&Oracle, ()> {
        self.quote_oracle.as_ref().ok_or(())
    }

    pub fn oracle(&self) -> Result<&Oracle, ()> {
        self.oracle.as_ref().ok_or(())
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

        let mut distributed_so_far = Quantity(0);
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

        if let Some(ref mut service) = self.services.lend {
            service.accrue_interest_rate(current_time);
            self.settle_fees(ServiceType::Lend)?
        }

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
            Quantity(u64::MAX),
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
