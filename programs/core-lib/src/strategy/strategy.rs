use crate::decimal::{Fraction, Quantity, Shares};
use crate::services::{ServiceType, Services};

/// Balances of both base and quote tokens
#[derive(Debug, Clone, Default)]
pub struct Balances {
    /// Token characteristic for vault
    base: Quantity,
    /// Stable token
    quote: Quantity,
}

/// Strategy is where liquidity providers can deposit their tokens
#[derive(Debug, Clone, Default)]
pub struct Strategy {
    /// Quantity of tokens used in lending (borrowed)
    lent: Option<Quantity>,
    /// Quantity of tokens used in swapping (swapped for other tokens)
    sold: Option<Balances>,
    /// Quantity of tokens used in trading (currently locked in a position)
    traded: Option<Balances>,

    /// Quantity of tokens available for use (not used)
    available: Balances,
    /// Sum of all locked tokens for each of the
    locked: Balances,

    /// Total amount of shares in a strategy
    total_shares: Shares,
    // fee accrued from services
    accrued_fee: Quantity,

    /// Ratio at which shares in this strategy can be used as a collateral
    collateral_ratio: Fraction,
    /// Ratio at which value of shares is calculated during liquidation
    liquidation_threshold: Fraction,
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

    fn sold_checked_mut(&mut self) -> Result<&mut Balances, ()> {
        self.sold.as_mut().ok_or(())
    }

    fn traded_checked_mut(&mut self) -> Result<&mut Balances, ()> {
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

    fn locked_in(&mut self, sub: ServiceType) -> Result<&mut Quantity, ()> {
        Ok(match sub {
            ServiceType::Lend => self.lent.as_mut().ok_or(())?,
            ServiceType::Swap => &mut self.sold.as_mut().ok_or(())?.base,
            ServiceType::Trade => &mut self.traded.as_mut().ok_or(())?.base,
        })
    }

    fn locked_in_quote(&mut self, sub: ServiceType) -> Result<&mut Quantity, ()> {
        Ok(match sub {
            ServiceType::Lend => {
                unreachable!("Lending of quote tokens is separate")
            }
            ServiceType::Swap => &mut self.sold_checked_mut()?.quote,
            ServiceType::Trade => &mut self.traded_checked_mut()?.quote,
        })
    }

    /// Add locked tokens to a specific substrategy
    pub fn accrue_fee(&mut self, quantity: Quantity, sub: ServiceType) -> Result<(), ()> {
        *self.locked_in(sub)? += quantity;
        self.accrued_fee += quantity;
        self.locked.base += quantity;

        Ok(())
    }

    /// Lock tokens in a specific substrategy
    pub fn lock(&mut self, quantity: Quantity, sub: ServiceType, services: &mut Services) {
        *self.locked_in(sub).unwrap() += quantity;

        // if self.can_lend() {
        //     services.lend_service().unwrap().remove_available(quantity);
        // }
        // if self.can_swap() {
        //     services.swap_service().unwrap().remove_available(quantity);
        // }
        // if self.can_trade() {
        //     services.trade_service().unwrap().remove_available(quantity);
        // }

        self.locked.base += quantity;
        self.available.base -= quantity;
    }

    /// Lock tokens in a specific substrategy
    pub fn lock_quote(
        &mut self,
        quantity: Quantity,
        sub: ServiceType,
        // services: &mut Services,
    ) {
        *self.locked_in_quote(sub).unwrap() += quantity;

        // if self.can_swap() {
        //     services.swap_service().unwrap().remove_available(quantity);
        // }
        // if self.can_trade() {
        //     services.trade_service().unwrap().remove_available(quantity);
        // }

        self.locked.quote += quantity;
        self.available.quote -= quantity;
    }

    pub fn unlock(&mut self, quantity: Quantity, sub: ServiceType, services: &mut Services) {
        *self.locked_in(sub).unwrap() -= quantity;

        // if self.can_lend() {
        //     services.lend_service().unwrap().remove_available(quantity);
        // }
        // if self.can_swap() {
        //     services.swap_service().unwrap().remove_available(quantity);
        // }
        // if self.can_trade() {
        //     services.trade_service().unwrap().remove_available(quantity);
        // }

        self.locked.base -= quantity;
        self.available.base += quantity;
    }

    pub fn unlock_quote(&mut self, quantity: Quantity, sub: ServiceType, services: &mut Services) {
        *self.locked_in_quote(sub).unwrap() += quantity;

        // if self.can_lend() {
        //     services.lend_service().unwrap().remove_available(quantity);
        // }
        // if self.can_swap() {
        //     services.swap_service().unwrap().remove_available(quantity);
        // }
        // if self.can_trade() {
        //     services.trade_service().unwrap().remove_available(quantity);
        // }

        self.locked.quote -= quantity;
        self.available.quote += quantity;
    }
}
