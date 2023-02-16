use crate::{
    decimal::Quantity,
    services::{ServiceType, Services},
    strategy::Strategy,
};

use super::Vault;

impl Vault {
    fn split(
        &mut self,
        quantity: Quantity,
        total: Quantity,
        service: ServiceType,
        part: fn(&Strategy) -> Quantity,
        action: fn(
            &mut Strategy,
            quantity: Quantity,
            service: ServiceType,
            services: &mut Services,
        ) -> Result<(), ()>,
    ) -> Result<(), ()> {
        let mut processed = Quantity(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self
                .strategies
                .get_mut_checked(i)
                .ok_or_else(|| unreachable!())?;

            if strategy.uses(service) {
                last_index = i;
                let to_lock = quantity.big_mul_div(part(&strategy), total);
                processed += to_lock;
                action(strategy, to_lock, service, &mut self.services)?;
            }
        }

        if processed < quantity {
            let strategy = self
                .strategies
                .get_mut_checked(last_index)
                .ok_or_else(|| unreachable!())?;
            action(strategy, quantity - processed, service, &mut self.services)?;
        }
        Ok(())
    }

    pub fn lock(
        &mut self,
        quantity: Quantity,
        total_available: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_available,
            service,
            Strategy::available,
            Strategy::lock,
        )
    }

    pub fn unlock(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked,
            Strategy::unlock,
        )
    }
}
