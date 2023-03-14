use crate::core_lib::{
    decimal::Quantity,
    errors::LibErrors,
    services::{ServiceType, Services},
    strategy::Strategy,
};
use checked_decimal_macro::Decimal;

use super::Vault;

type ActionFn = fn(
    &mut Strategy,
    quantity: Quantity,
    service: ServiceType,
    services: &mut Services,
) -> Result<(), LibErrors>;

impl Vault {
    fn split(
        &mut self,
        quantity: Quantity,
        total: Quantity,
        service: ServiceType,
        part: fn(&Strategy, ServiceType) -> Quantity,
        action: ActionFn,
    ) -> Result<(), LibErrors> {
        let mut processed = Quantity::new(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self.strategies.get_strategy_mut(i as u8)?;

            if strategy.uses(service) {
                last_index = i;
                let to_lock = quantity.big_mul_div(part(&strategy, service), total);
                processed += to_lock;
                action(strategy, to_lock, service, &mut self.services)?;
            }
        }

        if processed < quantity {
            let strategy = self.strategies.get_strategy_mut(last_index as u8)?;
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
        part: fn(&Strategy, ServiceType) -> Quantity,
        action_a: ActionFn,
        action_b: ActionFn,
    ) -> Result<(), LibErrors> {
        let mut processed_a = Quantity::new(0);
        let mut processed_b = Quantity::new(0);
        let mut last_index = 0;

        for i in self.strategies.indexes() {
            let strategy = self.strategies.get_strategy_mut(i as u8)?;
            if strategy.uses(service) {
                last_index = i;
                let to_lock_a = quantity_a.big_mul_div(part(&strategy, service), total);
                let to_lock_b = quantity_b.big_mul_div(part(&strategy, service), total);

                processed_a += to_lock_a;
                processed_b += to_lock_b;
                action_a(strategy, to_lock_a, service, &mut self.services)?;
                action_b(strategy, to_lock_b, service, &mut self.services)?;
            }
        }

        if processed_a < quantity_a {
            let strategy = self.strategies.get_strategy_mut(last_index as u8)?;
            action_a(
                strategy,
                quantity_a - processed_a,
                service,
                &mut self.services,
            )?;
        }

        if processed_b < quantity_b {
            let strategy = self.strategies.get_strategy_mut(last_index as u8)?;
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
    ) -> Result<(), LibErrors> {
        self.split(
            quantity,
            total_available,
            service,
            Strategy::available_in,
            Strategy::lock_base,
        )
    }

    pub fn lock_quote(
        &mut self,
        quantity: Quantity,
        total_available: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.split(
            quantity,
            total_available,
            service,
            Strategy::available_in_quote,
            Strategy::lock_quote,
        )
    }

    pub fn settle_lend_fees(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_in,
            Strategy::accrue_lend_fee,
        )
    }

    pub fn unlock_base(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_in,
            Strategy::unlock_base,
        )
    }

    pub fn unlock_quote(
        &mut self,
        quantity: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.split(
            quantity,
            total_locked,
            service,
            Strategy::locked_in_quote,
            Strategy::unlock_quote,
        )
    }

    pub fn exchange_to_quote(
        &mut self,
        sold: Quantity,
        bought: Quantity,
        total_available_base: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.double_split(
            sold,
            bought,
            total_available_base,
            service,
            Strategy::available_in,
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
    ) -> Result<(), LibErrors> {
        self.double_split(
            sold,
            bought,
            total_available_quote,
            service,
            Strategy::available_in_quote,
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
    ) -> Result<(), LibErrors> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_in,
            Strategy::unlock_base,
            Strategy::decrease_balance_base,
        )
    }

    pub fn unlock_with_loss_quote(
        &mut self,
        unlock: Quantity,
        loss: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_in_quote,
            Strategy::unlock_quote,
            Strategy::decrease_balance_quote,
        )
    }

    pub fn unlock_with_profit_base(
        &mut self,
        unlock: Quantity,
        loss: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_in,
            Strategy::unlock_base,
            Strategy::increase_balance_base,
        )
    }

    pub fn unlock_with_profit_quote(
        &mut self,
        unlock: Quantity,
        loss: Quantity,
        total_locked: Quantity,
        service: ServiceType,
    ) -> Result<(), LibErrors> {
        self.double_split(
            unlock,
            loss,
            total_locked,
            service,
            Strategy::locked_in_quote,
            Strategy::unlock_quote,
            Strategy::increase_balance_quote,
        )
    }
}

#[cfg(test)]
mod tests_general {
    use super::*;
    use crate::core_lib::{
        decimal::{Balances, DecimalPlaces, Fraction, Price, Shares, Utilization},
        structs::FeeCurve,
        Token,
    };
    use checked_decimal_macro::{Decimal, Factories};

    fn test_vault() -> Result<Vault, LibErrors> {
        let mut vault = Vault::default();

        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(5, 3),
            Price::from_scale(2, 2),
            0,
            Token::Base,
            0,
        )?;

        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(2, 2),
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

        vault.enable_trading(
            Fraction::new(100),
            Fraction::from_integer(3),
            Fraction::from_integer(1),
            Fraction::from_integer(1),
            0,
        )?;

        vault.add_strategy(
            true,
            true,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        vault.add_strategy(
            true,
            false,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        vault.add_strategy(
            false,
            true,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        Ok(vault)
    }

    #[test]
    fn deposit() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };
        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        assert_eq!(lend.available, base);
        assert_eq!(trade.available, balances);
        assert_eq!(swap.available, balances);
        assert_eq!(swap.balances, balances);
        assert_eq!(strategy.total_shares, shares);
        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base, quote, shares, &mut vault.services);

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(1)?;

        assert_eq!(lend.available, base + base);
        assert_eq!(trade.available, balances + balances);
        assert_eq!(swap.available, balances);
        assert_eq!(swap.balances, balances);
        assert_eq!(strategy.total_shares, shares);
        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);

        Ok(())
    }

    #[test]
    fn withdraw() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };
        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };
        let zero_quantity = Quantity::new(0);

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base, quote, shares, &mut vault.services);

        vault
            .strategies
            .get_strategy_mut(0)?
            .withdraw(base, quote, shares, &mut vault.services);

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        assert_eq!(lend.available, base);
        assert_eq!(trade.available, balances);
        assert_eq!(swap.available, zero_balances);
        assert_eq!(swap.balances, zero_balances);
        assert_eq!(strategy.total_shares, Shares::new(0));
        assert_eq!(strategy.available, zero_balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        vault
            .strategies
            .get_strategy_mut(1)?
            .withdraw(base, quote, shares, &mut vault.services);

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(1)?;

        assert_eq!(lend.available, zero_quantity);
        assert_eq!(trade.available, zero_balances);
        assert_eq!(swap.available, zero_balances);
        assert_eq!(swap.balances, zero_balances);
        assert_eq!(strategy.total_shares, Shares::new(0));
        assert_eq!(strategy.available, zero_balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        Ok(())
    }

    #[test]
    fn lock_unlock_base() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };
        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };
        let zero_quantity = Quantity::new(0);

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base, quote, shares, &mut vault.services);

        vault
            .strategies
            .get_strategy_mut(2)?
            .deposit(base, quote, shares, &mut vault.services);

        let swap = vault.swap_service_not_mut()?;

        let lock_base = Quantity::new(3000000000);
        assert_eq!(swap.available.base, Quantity::new(4000000000));

        vault.lock_base(lock_base, swap.available.base, ServiceType::Swap)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        let locked = Balances {
            quote: zero_quantity,
            base: Quantity::new(1500000000),
        };

        assert_eq!(strategy.available, balances - locked);
        assert_eq!(strategy.locked, locked);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(2)?;

        assert_eq!(strategy.available, balances - locked);
        assert_eq!(strategy.locked, locked);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances - locked - locked);
        assert_eq!(swap.balances, balances + balances);
        assert_eq!(lend.available, base + base - locked.base);
        assert_eq!(
            trade.available,
            balances + balances + balances - locked - locked
        );

        vault.unlock_base(lock_base, lock_base, ServiceType::Swap)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances);
        assert_eq!(swap.balances, balances + balances);
        assert_eq!(lend.available, base + base);
        assert_eq!(trade.available, balances + balances + balances);

        Ok(())
    }

    #[test]
    fn lock_unlock_quote() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };

        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };
        let zero_quantity = Quantity::new(0);

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        let base2 = Quantity::new(3000000000);
        let quote2 = Quantity::new(40000000000);

        let balances2 = Balances {
            base: base2,
            quote: quote2,
        };

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base2, quote2, shares, &mut vault.services);

        let base3 = Quantity::new(4000000000);
        let quote3 = Quantity::new(50000000000);

        let balances3 = Balances {
            base: base3,
            quote: quote3,
        };

        vault
            .strategies
            .get_strategy_mut(2)?
            .deposit(base3, quote3, shares, &mut vault.services);

        let trade = vault.trade_service_not_mut()?;
        let lock_quote = Quantity::new(3000000000);

        assert_eq!(trade.available.quote, quote + quote2 + quote3);

        vault.lock_quote(lock_quote, trade.available.quote, ServiceType::Trade)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        let locked = Balances {
            quote: Quantity::new(750000000),
            base: zero_quantity,
        };

        assert_eq!(strategy.available, balances - locked);
        assert_eq!(strategy.locked, locked);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        let locked2 = Balances {
            quote: Quantity::new(1000000000),
            base: zero_quantity,
        };

        assert_eq!(strategy.available, balances2 - locked2);
        assert_eq!(strategy.locked, locked2);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(2)?;

        let locked3 = Balances {
            quote: Quantity::new(1250000000),
            base: zero_quantity,
        };

        assert_eq!(strategy.available, balances3 - locked3);
        assert_eq!(strategy.locked, locked3);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances3 - locked - locked3);
        assert_eq!(swap.balances, balances + balances3);
        assert_eq!(lend.available, base + base2 - locked.base - locked2.base);
        assert_eq!(
            trade.available,
            balances + balances2 + balances3 - locked - locked2 - locked3
        );

        vault.unlock_quote(lock_quote, lock_quote, ServiceType::Trade)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        assert_eq!(strategy.available, balances);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        assert_eq!(strategy.available, balances2);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(2)?;

        assert_eq!(strategy.available, balances3);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances3);
        assert_eq!(swap.balances, balances + balances3);
        assert_eq!(lend.available, base + base2);
        assert_eq!(trade.available, balances + balances2 + balances3);

        Ok(())
    }

    #[test]
    fn unlock_quote_with_loss() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };

        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };
        let zero_quantity = Quantity::new(0);

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        let base2 = Quantity::new(3000000000);
        let quote2 = Quantity::new(40000000000);

        let balances2 = Balances {
            base: base2,
            quote: quote2,
        };

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base2, quote2, shares, &mut vault.services);

        let base3 = Quantity::new(4000000000);
        let quote3 = Quantity::new(50000000000);

        let balances3 = Balances {
            base: base3,
            quote: quote3,
        };

        vault
            .strategies
            .get_strategy_mut(2)?
            .deposit(base3, quote3, shares, &mut vault.services);

        let trade = vault.trade_service_not_mut()?;
        let lock_quote = Quantity::new(3000000000);

        assert_eq!(trade.available.quote, quote + quote2 + quote3);

        vault.lock_quote(lock_quote, trade.available.quote, ServiceType::Trade)?;

        let loss = Quantity::new(100000000);
        vault.unlock_with_loss_quote(lock_quote, loss, lock_quote, ServiceType::Trade)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        let loss1 = Balances {
            base: zero_quantity,
            quote: Quantity::new(25000000),
        };

        assert_eq!(strategy.available, balances - loss1);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        let loss2 = Balances {
            base: zero_quantity,
            quote: Quantity::new(33333333),
        };

        assert_eq!(strategy.available, balances2 - loss2);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(2)?;

        let loss3 = Balances {
            base: zero_quantity,
            quote: Quantity::new(41666667),
        };

        assert_eq!(strategy.available, balances3 - loss3);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances3 - loss1 - loss3);
        assert_eq!(swap.balances, balances + balances3 - loss1 - loss3);
        assert_eq!(lend.available, base + base2);
        assert_eq!(
            trade.available,
            balances + balances2 + balances3 - loss1 - loss2 - loss3
        );

        Ok(())
    }

    #[test]
    fn unlock_quote_with_profit() -> Result<(), LibErrors> {
        let mut vault = test_vault()?;

        let base = Quantity::new(2000000000);
        let quote = Quantity::new(30000000000);
        let shares = Shares::new(1000);
        let balances = Balances { base, quote };

        let zero_balances = Balances {
            quote: Quantity::new(0),
            base: Quantity::new(0),
        };
        let zero_quantity = Quantity::new(0);

        vault
            .strategies
            .get_strategy_mut(0)?
            .deposit(base, quote, shares, &mut vault.services);

        let base2 = Quantity::new(3000000000);
        let quote2 = Quantity::new(40000000000);

        let balances2 = Balances {
            base: base2,
            quote: quote2,
        };

        vault
            .strategies
            .get_strategy_mut(1)?
            .deposit(base2, quote2, shares, &mut vault.services);

        let base3 = Quantity::new(4000000000);
        let quote3 = Quantity::new(50000000000);

        let balances3 = Balances {
            base: base3,
            quote: quote3,
        };

        vault
            .strategies
            .get_strategy_mut(2)?
            .deposit(base3, quote3, shares, &mut vault.services);

        let trade = vault.trade_service_not_mut()?;
        let lock_quote = Quantity::new(3000000000);

        assert_eq!(trade.available.quote, quote + quote2 + quote3);

        vault.lock_quote(lock_quote, trade.available.quote, ServiceType::Trade)?;

        let profit = Quantity::new(100000000);
        vault.unlock_with_profit_quote(lock_quote, profit, lock_quote, ServiceType::Trade)?;

        let lend = vault.lend_service_not_mut()?;
        let trade = vault.trade_service_not_mut()?;
        let swap = vault.swap_service_not_mut()?;
        let strategy = vault.strategies.get_strategy(0)?;

        let profit1 = Balances {
            base: zero_quantity,
            quote: Quantity::new(25000000),
        };

        assert_eq!(strategy.available, balances + profit1);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(1)?;

        let profit2 = Balances {
            base: zero_quantity,
            quote: Quantity::new(33333333),
        };

        assert_eq!(strategy.available, balances2 + profit2);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        let strategy = vault.strategies.get_strategy(2)?;

        let profit3 = Balances {
            base: zero_quantity,
            quote: Quantity::new(41666667),
        };

        assert_eq!(strategy.available, balances3 + profit3);
        assert_eq!(strategy.locked, zero_balances);
        assert_eq!(strategy.accrued_fee, Quantity::new(0));

        assert_eq!(swap.available, balances + balances3 + profit1 + profit3);
        assert_eq!(swap.balances, balances + balances3 + profit1 + profit3);
        assert_eq!(lend.available, base + base2);
        assert_eq!(
            trade.available,
            balances + balances2 + balances3 + profit1 + profit2 + profit3
        );

        Ok(())
    }
}

// vault.add_strategy(
//     true,
//     true,
//     true,
//     Fraction::from_integer(1),
//     Fraction::from_integer(1),
// )?;

// vault.add_strategy(
//     true,
//     false,
//     true,
//     Fraction::from_integer(1),
//     Fraction::from_integer(1),
// )?;

// vault.add_strategy(
//     false,
//     true,
//     true,
//     Fraction::from_integer(1),
//     Fraction::from_integer(1),
// )?;
