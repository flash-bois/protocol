use crate::core_lib::decimal::{Balances, Fraction, Quantity, Shares};
use crate::core_lib::services::{ServiceType, ServiceUpdate, Services};

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq)]
    pub struct Strategy {
        /// Quantity of tokens used in lending (borrowed)
        pub lent: Option<Quantity>,
        /// Quantity of tokens used in swapping (swapped for other tokens)
        pub sold: Option<Balances>,
        /// Quantity of tokens used in trading (currently locked in a position)
        pub traded: Option<Balances>,

        /// Quantity of tokens available for use (not used)
        pub available: Balances,
        /// Sum of all locked tokens for each of the
        pub locked: Balances,

        /// Total amount of shares in a strategy
        pub total_shares: Shares,
        // fee accrued from services
        pub accrued_fee: Quantity,

        /// Ratio at which shares in this strategy can be used as a collateral
        pub collateral_ratio: Fraction,
        /// Ratio at which value of shares is calculated during liquidation
        pub liquidation_threshold: Fraction,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Strategy {
        /// Quantity of tokens used in lending (borrowed)
        pub lent: Option<Quantity>,
        /// Quantity of tokens used in swapping (swapped for other tokens)
        pub sold: Option<Balances>,
        /// Quantity of tokens used in trading (currently locked in a position)
        pub traded: Option<Balances>,

        /// Quantity of tokens available for use (not used)
        pub available: Balances,
        /// Sum of all locked tokens for each of the
        pub locked: Balances,

        /// Total amount of shares in a strategy
        pub total_shares: Shares,
        // fee accrued from services
        pub accrued_fee: Quantity,

        /// Ratio at which shares in this strategy can be used as a collateral
        pub collateral_ratio: Fraction,
        /// Ratio at which value of shares is calculated during liquidation
        pub liquidation_threshold: Fraction,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Strategy;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Strategy;

/// Strategy is where liquidity providers can deposit their tokens

#[cfg(test)]
impl Strategy {
    pub fn set_collateral_ratio(&mut self, ratio: Fraction) {
        self.collateral_ratio = ratio
    }
}

impl Strategy {
    pub fn available(&self) -> Quantity {
        self.available.base
    }

    pub fn available_quote(&self) -> Quantity {
        self.available.quote
    }

    pub fn locked(&self) -> Quantity {
        self.locked.base
    }

    pub fn locked_quote(&self) -> Quantity {
        self.locked.quote
    }

    pub fn balance(&self) -> Quantity {
        self.available.base + self.locked.base
    }

    pub fn balance_quote(&self) -> Quantity {
        self.available.quote + self.locked.quote
    }

    pub fn total_shares(&self) -> Shares {
        self.total_shares
    }

    pub fn collateral_ratio(&self) -> Fraction {
        self.collateral_ratio
    }

    pub fn liquidation_threshold(&self) -> Fraction {
        self.liquidation_threshold
    }

    pub fn is_lending_enabled(&self) -> bool {
        self.lent.is_some()
    }

    pub fn is_swapping_enabled(&self) -> bool {
        self.sold.is_some()
    }

    pub fn is_trading_enabled(&self) -> bool {
        self.traded.is_some()
    }

    fn lent_checked(&self) -> Result<Quantity, ()> {
        self.lent.as_ref().ok_or(()).copied()
    }

    fn sold_checked(&self) -> Result<&Balances, ()> {
        self.sold.as_ref().ok_or(())
    }

    fn traded_checked(&self) -> Result<&Balances, ()> {
        self.traded.as_ref().ok_or(())
    }

    fn _sold_checked_mut(&mut self) -> Result<&mut Balances, ()> {
        self.sold.as_mut().ok_or(())
    }

    fn _traded_checked_mut(&mut self) -> Result<&mut Balances, ()> {
        self.traded.as_mut().ok_or(())
    }

    pub fn locked_by(&self, service: ServiceType) -> Result<Quantity, ()> {
        let quantity_locked = match service {
            ServiceType::Lend => self.lent_checked()?,
            ServiceType::Swap => self.sold_checked()?.base,
            ServiceType::Trade => self.traded_checked()?.base,
        };

        Ok(quantity_locked)
    }

    pub fn uses(&self, service: ServiceType) -> bool {
        match service {
            ServiceType::Lend => self.lent.is_some(),
            ServiceType::Swap => self.sold.is_some(),
            ServiceType::Trade => self.traded.is_some(),
        }
    }

    pub fn new(lend: bool, swap: bool, trade: bool) -> Self {
        let mut strategy = Self::default();

        if lend {
            strategy.lent = Some(Quantity::default());
        }
        if swap {
            strategy.sold = Some(Balances::default());
        }
        if trade {
            strategy.traded = Some(Balances::default());
        }

        strategy
    }

    fn locked_in(&mut self, sub: ServiceType) -> &mut Quantity {
        let service = match sub {
            ServiceType::Lend => {
                return self
                    .lent
                    .as_mut()
                    .ok_or(())
                    .expect("locked in requested for a service that is not enabled");
            }
            ServiceType::Swap => self.sold.as_mut().ok_or(()),
            ServiceType::Trade => self.traded.as_mut().ok_or(()),
        };

        let service = service.expect("locked in requested for a service that is not enabled");
        &mut service.base
    }

    fn locked_in_quote(&mut self, sub: ServiceType) -> &mut Quantity {
        let service = match sub {
            ServiceType::Lend => {
                unreachable!("Lending of quote tokens is separate")
            }
            ServiceType::Swap => self.sold.as_mut().ok_or(()),
            ServiceType::Trade => self.traded.as_mut().ok_or(()),
        };
        let service = service.expect("locked in requested for a service that is not enabled");
        &mut service.quote
    }

    pub fn deposit(
        &mut self,
        quantity: Quantity,
        quote_quantity: Quantity,
        input_quantity: Quantity,
        balance: Quantity,
        services: &mut Services,
    ) -> Result<Shares, ()> {
        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_base(quantity);
            swap.add_liquidity_quote(quote_quantity);
        }

        let shares = self.total_shares.get_change_down(input_quantity, balance);

        self.available.base += quantity;
        self.available.quote += quote_quantity;
        self.total_shares += shares;

        Ok(shares)
    }

    /// Add locked tokens to a specific substrategy
    pub fn accrue_fee(&mut self, quantity: Quantity, sub: ServiceType) -> Result<(), ()> {
        *self.locked_in(sub) += quantity;
        self.accrued_fee += quantity;
        self.locked.base += quantity;

        Ok(())
    }

    /// Lock tokens in a specific substrategy
    pub fn lock(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        *self.locked_in(sub) += quantity;
        self.locked.base += quantity;
        self.available.base -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_base(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.remove_available_base(quantity);
        }
        Ok(())
    }

    /// Lock tokens in a specific substrategy
    pub fn lock_quote(&mut self, quantity: Quantity, sub: ServiceType, services: &mut Services) {
        *self.locked_in_quote(sub) += quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_quote(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.remove_available_quote(quantity);
        }

        self.locked.quote += quantity;
        self.available.quote -= quantity;
    }

    pub fn unlock(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        *self.locked_in(sub) -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.add_available_base(quantity);
        }

        self.locked.base -= quantity;
        self.available.base += quantity;

        Ok(())
    }

    pub fn unlock_quote(&mut self, quantity: Quantity, sub: ServiceType, services: &mut Services) {
        *self.locked_in_quote(sub) -= quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.add_available_quote(quantity);
        }

        self.locked.quote -= quantity;
        self.available.quote += quantity;
    }

    pub fn decrease_balance_base(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        self.available.base -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_base(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.remove_liquidity_base(quantity);
        }

        Ok(())
    }

    pub fn decrease_balance_quote(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        self.available.quote -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_quote(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.remove_liquidity_quote(quantity);
        }

        Ok(())
    }

    pub fn increase_balance_base(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        self.available.base += quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_base(quantity);
        }

        Ok(())
    }

    pub fn increase_balance_quote(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), ()> {
        self.available.quote += quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.add_available_quote(quantity);
        }
        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_quote(quantity);
        }

        Ok(())
    }
}
