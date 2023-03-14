use crate::core_lib::decimal::{Balances, Fraction, Quantity, Shares};
use crate::core_lib::errors::LibErrors;
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
    pub fn get_earned_double(&self, shares: &Shares) -> (Quantity, Quantity) {
        let base_quantity = self
            .total_shares()
            .calculate_earned(*shares, self.balance());

        let quote_quantity = self
            .total_shares()
            .calculate_earned(*shares, self.balance_quote());

        (base_quantity, quote_quantity)
    }

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

    pub fn uses(&self, service: ServiceType) -> bool {
        match service {
            ServiceType::Lend => self.lent.is_some(),
            ServiceType::Swap => self.sold.is_some(),
            ServiceType::Trade => self.traded.is_some(),
        }
    }

    pub fn new(
        lend: bool,
        swap: bool,
        trade: bool,
        collateral_ratio: Fraction,
        liquidation_threshold: Fraction,
    ) -> Self {
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

        strategy.collateral_ratio = collateral_ratio;
        strategy.liquidation_threshold = liquidation_threshold;

        strategy
    }

    fn locked_in_mut(&mut self, sub: ServiceType) -> &mut Quantity {
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

    pub fn locked_in(&self, sub: ServiceType) -> Quantity {
        let service = match sub {
            ServiceType::Lend => {
                return self
                    .lent
                    .ok_or(())
                    .expect("locked in requested for a service that is not enabled");
            }
            ServiceType::Swap => self.sold.ok_or(()),
            ServiceType::Trade => self.traded.ok_or(()),
        };

        let service = service.expect("locked in requested for a service that is not enabled");
        service.base
    }

    fn locked_in_quote_mut(&mut self, sub: ServiceType) -> &mut Quantity {
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

    pub fn locked_in_quote(&self, sub: ServiceType) -> Quantity {
        let service = match sub {
            ServiceType::Lend => {
                unreachable!("Lending of quote tokens is separate")
            }
            ServiceType::Swap => self.sold.ok_or(()),
            ServiceType::Trade => self.traded.ok_or(()),
        };
        let service = service.expect("locked in requested for a service that is not enabled");
        service.quote
    }

    pub fn withdraw(
        &mut self,
        quantity: Quantity,
        quote_quantity: Quantity,
        shares: Shares,
        services: &mut Services,
    ) {
        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.remove_liquidity_base(quantity);
            swap.remove_liquidity_quote(quote_quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.remove_available_base(quantity);
            trade.remove_available_quote(quantity);
        }

        self.available.base -= quantity;
        self.available.quote -= quote_quantity;
        self.total_shares -= shares;
    }

    pub fn deposit(
        &mut self,
        quantity: Quantity,
        quote_quantity: Quantity,
        shares: Shares,
        services: &mut Services,
    ) {
        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_base(quantity);
            swap.add_liquidity_quote(quote_quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.add_available_base(quantity);
            trade.add_available_quote(quote_quantity);
        }

        self.available.base += quantity;
        self.available.quote += quote_quantity;
        self.total_shares += shares;
    }

    /// Add locked tokens to a specific sub strategy
    pub fn accrue_lend_fee(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        *self.locked_in_mut(sub) += quantity;
        self.accrued_fee += quantity;
        self.locked.base += quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_base(quantity);
            swap.remove_available_base(quantity);
        }

        Ok(())
    }

    /// Lock tokens in a specific sub strategy
    pub fn lock_base(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        *self.locked_in_mut(sub) += quantity;
        self.locked.base += quantity;
        self.available.base -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.remove_available_base(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.remove_available_base(quantity);
        }

        Ok(())
    }

    /// Lock tokens in a specific substrategy
    pub fn lock_quote(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        *self.locked_in_quote_mut(sub) += quantity;
        self.locked.quote += quantity;
        self.available.quote -= quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.remove_available_quote(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.remove_available_quote(quantity);
        }

        Ok(())
    }

    pub fn unlock_base(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        *self.locked_in_mut(sub) -= quantity;
        self.locked.base -= quantity;
        self.available.base += quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.add_available_base(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.add_available_base(quantity);
        }

        Ok(())
    }

    pub fn unlock_quote(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        *self.locked_in_quote_mut(sub) -= quantity;
        self.locked.quote -= quantity;
        self.available.quote += quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.add_available_quote(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.add_available_quote(quantity);
        }

        Ok(())
    }

    pub fn decrease_balance_base(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        self.available.base -= quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.remove_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.remove_liquidity_base(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.remove_available_base(quantity);
        }

        Ok(())
    }

    pub fn decrease_balance_quote(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        self.available.quote -= quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.remove_liquidity_quote(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.remove_available_quote(quantity);
        }

        Ok(())
    }

    pub fn increase_balance_base(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        self.available.base += quantity;

        if let Ok(lend) = services.lend_mut() {
            lend.add_available_base(quantity);
        }

        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_base(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.add_available_base(quantity);
        }

        Ok(())
    }

    pub fn increase_balance_quote(
        &mut self,
        quantity: Quantity,
        _: ServiceType,
        services: &mut Services,
    ) -> Result<(), LibErrors> {
        self.available.quote += quantity;

        if let Ok(swap) = services.swap_mut() {
            swap.add_liquidity_quote(quantity);
        }

        if let Ok(trade) = services.trade_mut() {
            trade.add_available_quote(quantity);
        }

        Ok(())
    }

    pub fn available_in(&self, _service: ServiceType) -> Quantity {
        self.available()
    }

    pub fn available_in_quote(&self, _service: ServiceType) -> Quantity {
        self.available_quote()
    }
}
