pub mod deposit;
pub mod lend;

use crate::{
    decimal::{DecimalPlaces, Price, Quantity, Shares, Time, Utilization, Value},
    services::{lending::Lend, ServiceType, ServiceUpdate, Services},
    strategy::{Strategies, Strategy},
    structs::{FeeCurve, Oracle},
};

#[derive(Clone, Debug, Default)]
pub struct Vault {
    pub id: u8,

    pub oracle: Option<Oracle>,
    pub quote_oracle: Option<Oracle>,

    pub services: Services,
    pub strategies: Strategies,
}

#[cfg(test)]
use checked_decimal_macro::Factories;

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
            time,
            spread_limit,
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
            time,
            spread_limit,
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
                    let fees = service.accrue_fee(None);
                    (service, fees)
                }
                _ => unimplemented!(),
            };

        let locked = service_updatable.locked();
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

    fn lock(
        &mut self,
        quantity: Quantity,
        total_available: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        let mut locked_so_far = Quantity(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self.strategies.get_mut_checked(i).unwrap();
            if strategy.uses(service) {
                last_index = i;
                let to_lock = quantity.big_mul_div(strategy.available(), total_available);
                locked_so_far += to_lock;
                strategy.lock(to_lock, service, &mut self.services);
            }
        }

        if locked_so_far < quantity {
            let strategy = self.strategies.get_mut_checked(last_index).unwrap();
            strategy.lock(quantity - locked_so_far, service, &mut self.services);
        }
        Ok(())
    }

    fn unlock(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        if quantity == Quantity(0) {
            return Ok(());
        }

        let mut unlocked_so_far = Quantity(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self.strategies.get_mut_checked(i).unwrap();
            if strategy.uses(service) {
                last_index = i;
                let to_unlock = quantity.big_mul_div(strategy.locked(), total_locked);
                unlocked_so_far += to_unlock;
                strategy.unlock(to_unlock, service, &mut self.services);
            }
        }

        if unlocked_so_far < quantity {
            let strategy = self.strategies.get_mut_checked(last_index).unwrap();
            strategy.unlock(quantity - unlocked_so_far, service, &mut self.services);
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
