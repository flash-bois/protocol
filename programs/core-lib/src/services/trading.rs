use checked_decimal_macro::{BetweenDecimals, BigOps, Decimal, Factories, Others};
use std::cmp::min;

use crate::decimal::{
    Balances, BigFraction, Fraction, FundingRate, Precise, Price, Quantity, Time, Value,
};
use crate::structs::oracle::{Oracle, OraclePriceType};
use crate::structs::{FeeCurve, Receipt, Side};

use super::ServiceUpdate;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trade {
    /// liquidity available to be locked
    available: Balances,
    /// total liquidity locked inside the vault
    locked: Balances,
    /// total value at the moment of opening a position
    long_open_value: Value,
    short_open_value: Value,

    /// struct for calculation of position fee
    borrow_fee: FeeCurve,
    /// fee paid on opening a position
    open_fee: Fraction,

    /// maximum leverage allowed
    max_open_leverage: Fraction,
    /// maximum leverage allowed
    max_leverage: Fraction,

    /// fees waiting to be distributed to liquidity providers
    accrued_fee: Balances,
}

pub enum BalanceChange {
    Profit(Quantity),
    Loss(Quantity),
}

impl ServiceUpdate for Trade {
    fn add_liquidity_base(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn add_liquidity_quote(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn remove_liquidity_base(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn remove_liquidity_quote(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }

    fn add_available_base(&mut self, quantity: Quantity) {
        self.available.base += quantity;
    }

    fn add_available_quote(&mut self, quantity: Quantity) {
        self.available.quote += quantity;
    }

    fn remove_available_base(&mut self, quantity: Quantity) {
        self.available.base -= quantity;
    }

    fn remove_available_quote(&mut self, quantity: Quantity) {
        self.available.quote -= quantity;
    }

    fn available(&self) -> Balances {
        self.available
    }

    fn locked(&self) -> Balances {
        self.locked
    }

    fn accrue_fee(&mut self) -> Balances {
        let fee = self.accrued_fee;
        self.accrued_fee = Balances::default();
        fee
    }
}

impl Trade {
    pub fn new(
        fee: FeeCurve,
        open_fee: Fraction,
        max_leverage: Fraction,
        start_time: Time,
    ) -> Self {
        Self {
            available: Balances::default(),
            locked: Balances::default(),
            long_open_value: Value::from_integer(0),
            short_open_value: Value::from_integer(0),
            borrow_fee: fee,
            open_fee,
            max_open_leverage: max_leverage,
            max_leverage,
            accrued_fee: Balances::default(),
        }
    }

    /// opens a long position
    pub fn open_long(
        &mut self,
        quantity: Quantity,
        collateral: Value,
        oracle: &Oracle,
        current_time: Time,
    ) -> Result<Receipt, ()> {
        let position_value = oracle.calculate_needed_value(quantity);
        let collateralization = Fraction::from_decimal_up(position_value.div_up(collateral));
        if collateralization > self.max_open_leverage {
            return Err(());
        }
        if quantity > self.available.base {
            return Err(());
        }

        self.locked.base += quantity;
        self.long_open_value += position_value;

        Ok(Receipt {
            side: Side::Long,
            size: quantity,
            locked: quantity,
            open_price: oracle.price(OraclePriceType::Buy),
            open_value: position_value,
        })
    }

    /// Opens a short position
    /// Returns the receipt of the position and value that needs to be locked
    pub fn open_short(
        &mut self,
        quantity: Quantity,
        quote_quantity: Quantity,
        collateral: Value,
        oracle: &Oracle,
        current_time: Time,
    ) -> Result<Receipt, ()> {
        let position_value = oracle.calculate_value(quantity);
        let collateralization = Fraction::from_decimal_up(position_value.div_up(collateral));

        if collateralization > self.max_open_leverage {
            return Err(());
        }

        self.locked.quote += quote_quantity;
        self.short_open_value += position_value;

        Ok(Receipt {
            side: Side::Short,
            size: quantity,
            locked: quote_quantity,
            open_price: oracle.price(OraclePriceType::Sell),
            open_value: position_value,
        })
    }
}
