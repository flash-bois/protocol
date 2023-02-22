use checked_decimal_macro::{BetweenDecimals, BigOps, Decimal, Factories, Others};
use std::cmp::min;
use std::default;

use crate::decimal::{
    Balances, BigFraction, Both, Fraction, FundingRate, Precise, Price, Quantity, Shares, Time,
    Value,
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
    open_value: Both<Value>,

    /// struct for calculation of position fee
    borrow_fee: Both<FeeCurve>,
    funding: Both<FundingRate>,
    last_fee: Time,
    funding_multiplier: Fraction,

    /// fee paid on opening a position
    open_fee: Fraction,

    /// maximum leverage allowed at the moment of opening a position
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
            open_value: Both::<Value>::default(),
            borrow_fee: Both::<FeeCurve>::default(),
            funding: Both::<FundingRate>::default(),
            funding_multiplier: Fraction::from_integer(1),
            last_fee: start_time,
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
        self.open_value.base += position_value;

        Ok(Receipt {
            side: Side::Long,
            size: quantity,
            locked: quantity,
            initial_funding: self.funding.base,
            open_price: oracle.price(OraclePriceType::Buy),
            open_value: position_value,
        })
    }

    /// Opens a short position
    /// Returns the receipt of the position and value that needs to be locked
    pub fn open_short(
        &mut self,
        quantity: Quantity,
        collateral: Value,
        oracle: &Oracle,
        quote_oracle: &Oracle,
        current_time: Time,
    ) -> Result<Receipt, ()> {
        let position_value = oracle.calculate_value(quantity);
        let quote_quantity = quote_oracle.calculate_needed_quantity(position_value);
        let collateralization = Fraction::from_decimal_up(position_value.div_up(collateral));

        if collateralization > self.max_open_leverage {
            return Err(());
        }

        self.locked.quote += quote_quantity;
        self.open_value.quote += position_value;

        Ok(Receipt {
            side: Side::Short,
            size: quantity,
            locked: quote_quantity,
            initial_funding: self.funding.quote,
            open_price: oracle.price(OraclePriceType::Sell),
            open_value: position_value,
        })
    }

    fn utilization(&self) -> Both<Fraction> {
        Both::<Fraction> {
            base: Fraction::from_decimal(self.locked.base)
                / Fraction::from_decimal(self.available.base + self.locked.base),
            quote: Fraction::from_decimal(self.locked.quote)
                / Fraction::from_decimal(self.available.quote + self.locked.quote),
        }
    }

    fn calculate_fee(&self, current_time: Time) -> Both<FundingRate> {
        if current_time > self.last_fee {
            let time_period = current_time - self.last_fee;

            let utilization = self.utilization();

            let base_fee = self
                .borrow_fee
                .base
                .compounded_fee(utilization.base, time_period);
            let quote_fee = self
                .borrow_fee
                .quote
                .compounded_fee(utilization.quote, time_period);

            Both {
                base: FundingRate::from_decimal(base_fee),
                quote: FundingRate::from_decimal(quote_fee),
            }
        } else {
            Both::default()
        }
    }

    fn calculate_funding(&self, oracle: &Oracle, quote_oracle: &Oracle) -> (Fraction, Side) {
        let long_value = oracle.calculate_value(self.locked.base);
        let short_value = quote_oracle.calculate_value(self.locked.quote);

        let total_value = long_value + short_value;

        if long_value >= short_value {
            let longs = (long_value / total_value) - Value::from_scale(5, 1);
            (
                Fraction::from_decimal(longs) * self.funding_multiplier,
                Side::Long,
            )
        } else {
            let shorts = (short_value / total_value) - Value::from_scale(5, 1);
            (
                Fraction::from_decimal(shorts) * self.funding_multiplier,
                Side::Short,
            )
        }
    }

    fn refresh(&mut self, oracle: &Oracle, quote_oracle: &Oracle, now: Time) {
        let fee = self.calculate_fee(now);
        let funding = match self.calculate_funding(oracle, quote_oracle) {
            (funding, Side::Long) => FundingRate::from_decimal(funding),
            (funding, Side::Short) => FundingRate(0) - FundingRate::from_decimal(funding),
        };

        self.funding.base += fee.base - funding;
        self.funding.quote += fee.quote + funding;

        FundingRate::from_decimal(fee.base);
    }
}