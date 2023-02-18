use crate::{
    decimal::Quantity,
    services::{ServiceType, Services},
    strategy::Strategy,
};

use super::Vault;

type ActionFn = fn(
    &mut Strategy,
    quantity: Quantity,
    service: ServiceType,
    services: &mut Services,
) -> Result<(), ()>;

impl Vault {
    fn split(
        &mut self,
        quantity: Quantity,
        total: Quantity,
        service: ServiceType,
        part: fn(&Strategy) -> Quantity,
        action: ActionFn,
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

    fn double_split(
        &mut self,
        quantity_a: Quantity,
        quantity_b: Quantity,
        total: Quantity,
        service: ServiceType,
        part: fn(&Strategy) -> Quantity,
        action_a: ActionFn,
        action_b: ActionFn,
    ) -> Result<(), ()> {
        let mut processed_a = Quantity(0);
        let mut processed_b = Quantity(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self
                .strategies
                .get_mut_checked(i)
                .ok_or_else(|| unreachable!())?;

            if strategy.uses(service) {
                last_index = i;
                let to_lock_a = quantity_a.big_mul_div(part(&strategy), total);
                let to_lock_b = quantity_b.big_mul_div(part(&strategy), total);
                processed_a += to_lock_a;
                processed_b += to_lock_b;
                action_a(strategy, to_lock_a, service, &mut self.services)?;
                action_b(strategy, to_lock_b, service, &mut self.services)?;
            }
        }

        if processed_a < quantity_a {
            let strategy = self
                .strategies
                .get_mut_checked(last_index)
                .ok_or_else(|| unreachable!())?;
            action_a(
                strategy,
                quantity_a - processed_a,
                service,
                &mut self.services,
            )?;
        }

        if processed_b < quantity_b {
            let strategy = self
                .strategies
                .get_mut_checked(last_index)
                .ok_or_else(|| unreachable!())?;
            action_b(
                strategy,
                quantity_b - processed_b,
                service,
                &mut self.services,
            )?;
        }
        Ok(())
    }

    pub fn lock_base(
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
            Strategy::lock_base,
        )
    }

    pub fn lock_quote(
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
            Strategy::lock_quote,
        )
    }

    pub fn unlock_base(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_base,
            Strategy::unlock_base,
        )
    }

    pub fn unlock_quote(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_quote,
            Strategy::unlock_quote,
        )
    }

    pub fn exchange_to_quote(
        &mut self,
        sold: Quantity,
        bought: Quantity,
        total_available_base: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.double_split(
            sold,
            bought,
            total_available_base,
            service,
            Strategy::available,
            Strategy::decrease_balance_base,
            Strategy::increase_balance_quote,
        )
    }

    pub fn exchange_to_base(
        &mut self,
        sold: Quantity,
        bought: Quantity,
        total_available_quote: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.double_split(
            sold,
            bought,
            total_available_quote,
            service,
            Strategy::available,
            Strategy::decrease_balance_quote,
            Strategy::increase_balance_base,
        )
    }

    pub fn unlock_with_loss_base(
        &mut self,
        unlock: Quantity,
        loss: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_base,
            Strategy::unlock_base,
            Strategy::increase_balance_base,
        )
    }

    pub fn unlock_with_loss_quote(
        &mut self,
        unlock: Quantity,
        loss: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_base,
            Strategy::unlock_quote,
            Strategy::decrease_balance_quote,
        )
    }

    pub fn profit_base(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_base,
            Strategy::increase_balance_base,
        )
    }

    pub fn profit_quote(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), ()> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_quote,
            Strategy::increase_balance_quote,
        )
    }
}
