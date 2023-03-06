use checked_decimal_macro::{BetweenDecimals, Decimal, Factories, Others};
use std::cmp::min;

use crate::core_lib::{
    decimal::{BalanceChange, Balances, Both, Fraction, FundingRate, Quantity, Time, Value},
    errors::LibErrors,
    structs::{
        oracle::{Oracle, OraclePriceType},
        FeeCurve, Receipt, Side,
    },
    user::TradeResult,
};

use super::ServiceUpdate;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(C)]
    pub struct Trade {
        /// liquidity available to be locked
        pub available: Balances,
        /// total liquidity locked inside the vault
        pub locked: Balances,
        /// total value at the moment of opening a position
        pub open_value: Both<Value>,

        /// struct for calculation of position fee
        pub borrow_fee: Both<FeeCurve>,
        pub funding: Both<FundingRate>,
        pub last_fee: Time,
        pub funding_multiplier: Fraction,

        /// fee paid on opening a position
        pub open_fee: Fraction,

        /// maximum leverage allowed at the moment of opening a position
        pub max_open_leverage: Fraction,
        /// maximum leverage allowed
        pub max_leverage: Fraction,

        /// fees waiting to be distributed to liquidity providers
        pub accrued_fee: Balances,

        pub collateral_ratio: Fraction,
        pub liquidation_threshold: Fraction,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(C)]
    pub struct Trade {
        /// liquidity available to be locked
        pub available: Balances,
        /// total liquidity locked inside the vault
        pub locked: Balances,
        /// total value at the moment of opening a position
        pub open_value: Both<Value>,

        /// struct for calculation of position fee
        pub borrow_fee: Both<FeeCurve>,
        pub funding: Both<FundingRate>,
        pub last_fee: Time,
        pub funding_multiplier: Fraction,

        /// fee paid on opening a position
        pub open_fee: Fraction,

        /// maximum leverage allowed at the moment of opening a position
        pub max_open_leverage: Fraction,
        /// maximum leverage allowed
        pub max_leverage: Fraction,

        /// fees waiting to be distributed to liquidity providers
        pub accrued_fee: Balances,

        pub collateral_ratio: Fraction,
        pub liquidation_threshold: Fraction,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Trade;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Trade;

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
    pub fn collateral_ratio(&self) -> Fraction {
        self.collateral_ratio
    }

    pub fn liquidation_threshold(&self) -> Fraction {
        self.liquidation_threshold
    }

    pub fn new(
        open_fee: Fraction,
        max_leverage: Fraction,
        start_time: Time,
        collateral_ratio: Fraction,
        liquidation_threshold: Fraction,
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
            collateral_ratio,
            liquidation_threshold,
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
    ) -> Result<Receipt, LibErrors> {
        let position_value = oracle.calculate_needed_value(quantity);
        let collateralization = Fraction::from_decimal_up(position_value.div_up(collateral));

        if collateralization > self.max_open_leverage {
            return Err(LibErrors::CollateralizationTooLow);
        }
        if quantity > self.available.base {
            return Err(LibErrors::NotEnoughBaseQuantity);
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
    ) -> Result<Receipt, LibErrors> {
        let position_value = oracle.calculate_value(quantity);
        let quote_quantity = quote_oracle.calculate_needed_quantity(position_value);
        let collateralization = Fraction::from_decimal_up(position_value.div_up(collateral));

        if collateralization > self.max_open_leverage {
            return Err(LibErrors::CollateralizationTooLow);
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

    pub fn close_long(
        &mut self,
        receipt: Receipt,
        oracle: &Oracle,
    ) -> Result<(BalanceChange, Quantity), LibErrors> {
        let funding_fee = self.calculate_funding_fee(&receipt);

        let position_change = self.calculate_long_value(&receipt, oracle);
        let open_fee = receipt.size * self.open_fee;
        let change = position_change + funding_fee + BalanceChange::Loss(open_fee);

        self.open_value.base -= receipt.open_value;
        self.locked.base -= receipt.locked;

        Ok((change, receipt.locked))
    }

    pub fn close_short(
        &mut self,
        receipt: Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
        now: Time,
    ) -> Result<(BalanceChange, Quantity), LibErrors> {
        let funding_fee = self.calculate_funding_fee(&receipt);

        self.locked.quote -= receipt.locked;
        self.open_value.quote -= receipt.open_value;

        let position_change = self.calculate_short_change(&receipt, oracle, quote_oracle);

        let open_fee = receipt.size * self.open_fee;
        let change = position_change + funding_fee + BalanceChange::Loss(open_fee);

        Ok((change, receipt.locked))
    }

    pub fn calculate_value(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> TradeResult {
        let fee =
            self.calculate_funding_fee(receipt) + BalanceChange::Loss(receipt.size * self.open_fee);

        match receipt.side {
            Side::Long => match self.calculate_long_value(receipt, oracle) + fee {
                BalanceChange::Profit(profit) => {
                    TradeResult::Profitable(oracle.calculate_value(profit))
                }
                BalanceChange::Loss(loss) => TradeResult::Loss(oracle.calculate_needed_value(loss)),
            },
            Side::Short => match self.calculate_short_change(receipt, oracle, quote_oracle) + fee {
                BalanceChange::Profit(profit) => {
                    TradeResult::Profitable(quote_oracle.calculate_value(profit))
                }
                BalanceChange::Loss(loss) => {
                    TradeResult::Loss(quote_oracle.calculate_needed_value(loss))
                }
            },
        }
    }

    fn calculate_long_value(&self, receipt: &Receipt, oracle: &Oracle) -> BalanceChange {
        let Receipt {
            size, open_price, ..
        } = receipt;
        let close_price = oracle.price(OraclePriceType::Sell);

        match close_price > *open_price {
            true => {
                let profit_value =
                    oracle.calculate_value_difference_down(*size, close_price, *open_price);
                let profit = oracle.calculate_quantity(profit_value);

                BalanceChange::Profit(profit)
            }
            false => {
                let loss_value =
                    oracle.calculate_value_difference_up(*size, *open_price, close_price);
                let loss = oracle.calculate_needed_quantity(loss_value);

                BalanceChange::Loss(loss)
            }
        }
    }

    fn calculate_short_change(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> BalanceChange {
        let close_price = oracle.price(OraclePriceType::Buy);
        let Receipt {
            size,
            open_price,
            locked,
            ..
        } = receipt;

        match *open_price > close_price {
            true => {
                let profit_value =
                    oracle.calculate_value_difference_down(*size, *open_price, close_price);
                let profit = quote_oracle.calculate_quantity(profit_value);

                // maximum profit is limited by locked quote quantity (no change for constant price of quote)
                BalanceChange::Profit(min(*locked, profit))
            }
            false => {
                let loss_value =
                    oracle.calculate_value_difference_up(*size, close_price, *open_price);
                let loss = quote_oracle.calculate_needed_quantity(loss_value);
                BalanceChange::Loss(loss)
            }
        }
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
            (funding, Side::Short) => FundingRate::new(0) - FundingRate::from_decimal(funding),
        };

        self.funding.base += fee.base - funding;
        self.funding.quote += fee.quote + funding;

        FundingRate::from_decimal(fee.base);
    }

    fn calculate_funding_fee(&self, receipt: &Receipt) -> BalanceChange {
        let funding_change = match receipt.side {
            Side::Long => self.funding.base - receipt.initial_funding,
            Side::Short => self.funding.quote - receipt.initial_funding,
        };

        if funding_change > FundingRate::from_integer(0) {
            BalanceChange::Loss(receipt.size * funding_change)
        } else {
            BalanceChange::Profit(receipt.size * (FundingRate::from_integer(0) - funding_change))
        }
    }
}