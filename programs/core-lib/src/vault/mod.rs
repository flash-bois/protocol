pub mod deposit;
pub mod general;
pub mod lend;
pub mod swap;

use crate::{
    decimal::{DecimalPlaces, Price, Quantity, Shares, Time, Utilization, Value},
    services::{lending::Lend, ServiceType, ServiceUpdate, Services},
    strategy::{Strategies, Strategy},
    structs::{FeeCurve, Oracle},
};

#[cfg(test)]
use checked_decimal_macro::Factories;

#[derive(Clone, Debug, Default)]
pub struct Vault {
    pub id: u8,

    pub oracle: Option<Oracle>,
    pub quote_oracle: Option<Oracle>,

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
        // if has_swap && self.services.swap.is_none() {
        //     return Err(());
        // }

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
    ) -> Result<(), ()> {
        if self.oracle.is_some() {
            return Err(());
        }

        self.oracle = Some(Oracle::new(
            decimal_places,
            price,
            confidence,
            spread_limit,
            time,
        )?);

        Ok(())
    }

    pub fn enable_quote_oracle(
        &mut self,
        decimal_places: DecimalPlaces,
        price: Price,
        confidence: Price,
        spread_limit: Price,
        time: Time,
    ) -> Result<(), ()> {
        if self.quote_oracle.is_some() {
            return Err(());
        }

        self.quote_oracle = Some(Oracle::new(
            decimal_places,
            price,
            confidence,
            spread_limit,
            time,
        )?);

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

    pub fn enable_swapping(&mut self, swapping_fee: FeeCurve) -> Result<(), ()> {
        if self.oracle.is_none() {
            return Err(());
        }
        // if self.services.swap.is_some() {
        //     return Err(());
        // }

        // TODO: create swap

        Ok(())
    }

    pub fn lend_service(&mut self) -> Result<&mut Lend, ()> {
        self.services.lend.as_mut().ok_or(())
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
            Price::from_scale(1, 2),
            Price::from_scale(5, 3),
            0,
        )?;
        vault.enable_lending(
            FeeCurve::default(),
            Utilization::from_integer(1),
            Quantity(u64::MAX),
            0,
        )?;
        vault.enable_swapping(FeeCurve::default())?;
        vault.add_strategy(true, true, false)?;
        Ok(vault)
    }
}
