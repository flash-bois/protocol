use checked_decimal_macro::{BetweenDecimals, BigOps, Decimal, Factories, Others};
use std::cmp::min;

use crate::core_lib::{
    decimal::{
        BalanceChange, Balances, BothFeeCurves, BothFractions, BothFundingRates, BothValues,
        Fraction, FundingRate, Quantity, Time, Utilization, Value,
    },
    errors::LibErrors,
    structs::{
        oracle::{Oracle, OraclePriceType},
        Receipt, Side,
    },
    user::ValueChange,
};

use super::ServiceUpdate;

#[cfg(feature = "anchor")]
mod zero {
    use crate::core_lib::decimal::{BothFeeCurves, BothFundingRates, BothValues};

    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Debug, Default, PartialEq, Eq)]
    #[repr(C)]
    pub struct Trade {
        /// liquidity available to be locked
        pub available: Balances,
        /// total liquidity locked inside the vault
        pub locked: Balances,
        /// total value at the moment of opening a position
        pub open_value: BothValues,

        /// struct for calculation of position fee
        pub borrow_fee: BothFeeCurves,
        pub funding: BothFundingRates,
        pub last_fee: u32,
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
    use crate::core_lib::decimal::{BothFeeCurves, BothFundingRates, BothValues};

    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(C)]
    pub struct Trade {
        /// liquidity available to be locked
        pub available: Balances,
        /// total liquidity locked inside the vault
        pub locked: Balances,
        /// total value at the moment of opening a position
        pub open_value: BothValues,

        /// struct for calculation of position fee
        pub borrow_fee: BothFeeCurves,
        pub funding: BothFundingRates,
        pub last_fee: u32,
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
            open_value: BothValues::default(),
            borrow_fee: BothFeeCurves::default(),
            funding: BothFundingRates::default(),
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
        receipt: &Receipt,
        oracle: &Oracle,
    ) -> Result<(BalanceChange, Quantity), LibErrors> {
        let funding_fee = self.calculate_funding_fee(&receipt);
        let open_fee = BalanceChange::Loss(receipt.locked * self.open_fee);

        let position_change = self.calculate_long_change(&receipt, oracle);
        let change = position_change + funding_fee + open_fee;

        self.open_value.base -= receipt.open_value;
        self.locked.base -= receipt.locked;

        Ok((change, receipt.locked))
    }

    pub fn close_short(
        &mut self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> Result<(BalanceChange, Quantity), LibErrors> {
        let funding_fee = self.calculate_quote_funding_fee(&receipt, oracle, quote_oracle);
        let open_fee = receipt.locked * self.open_fee;

        let position_change = self.calculate_short_change(&receipt, oracle, quote_oracle);
        let change = position_change + funding_fee + BalanceChange::Loss(open_fee);

        self.locked.quote -= receipt.locked;
        self.open_value.quote -= receipt.open_value;

        Ok((change, receipt.locked))
    }

    pub fn long_fees(&self, receipt: &Receipt) -> BalanceChange {
        self.calculate_funding_fee(receipt) + BalanceChange::Loss(receipt.locked * self.open_fee)
    }

    pub fn short_fees(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> BalanceChange {
        self.calculate_quote_funding_fee(receipt, oracle, quote_oracle)
            + BalanceChange::Loss(receipt.locked * self.open_fee)
    }

    fn get_value_change(&self, change: &BalanceChange, oracle: &Oracle) -> ValueChange {
        match change {
            BalanceChange::Profit(profit) => {
                ValueChange::Profitable(oracle.calculate_value(*profit))
            }

            BalanceChange::Loss(loss) => ValueChange::Loss(oracle.calculate_needed_value(*loss)),
        }
    }

    pub fn calculate_position(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
        minus_fees: bool,
    ) -> (BalanceChange, ValueChange) {
        match receipt.side {
            Side::Long => {
                let balance_change =
                    self.calculate_position_change(receipt, oracle, quote_oracle, minus_fees);
                let value_change = self.get_value_change(&balance_change, oracle);

                (balance_change, value_change)
            }
            Side::Short => {
                let balance_change =
                    self.calculate_position_change(receipt, oracle, quote_oracle, minus_fees);
                let value_change = self.get_value_change(&balance_change, quote_oracle);

                (balance_change, value_change)
            }
        }
    }

    pub fn calculate_position_change(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
        minus_fees: bool,
    ) -> BalanceChange {
        match receipt.side {
            Side::Long => {
                let change = self.calculate_long_change(receipt, oracle);

                if minus_fees {
                    change + self.long_fees(receipt)
                } else {
                    change
                }
            }

            Side::Short => {
                let change = self.calculate_short_change(receipt, oracle, quote_oracle);

                if minus_fees {
                    change + self.short_fees(receipt, oracle, quote_oracle)
                } else {
                    change
                }
            }
        }
    }

    fn calculate_long_change(&self, receipt: &Receipt, oracle: &Oracle) -> BalanceChange {
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

    fn utilization(&self) -> BothFractions {
        BothFractions {
            base: Fraction::from_decimal_up(Utilization::get_utilization(
                self.locked.base,
                self.available.base + self.locked.base,
            )),
            quote: Fraction::from_decimal_up(Utilization::get_utilization(
                self.locked.quote,
                self.available.quote + self.locked.quote,
            )),
        }
    }

    fn calculate_fee(&self, current_time: Time) -> BothFundingRates {
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

            BothFundingRates {
                base: FundingRate::from_decimal(base_fee),
                quote: FundingRate::from_decimal(quote_fee),
            }
        } else {
            BothFundingRates::default()
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
    }

    fn calculate_quote_funding_fee(
        &self,
        receipt: &Receipt,
        oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> BalanceChange {
        match self.calculate_funding_fee(receipt) {
            BalanceChange::Profit(profit) if profit == Quantity::new(0) => {
                BalanceChange::Profit(profit)
            }
            BalanceChange::Profit(profit) => {
                let value = oracle.calculate_value(profit);
                BalanceChange::Profit(quote_oracle.calculate_quantity(value))
            }
            BalanceChange::Loss(loss) => {
                let value = oracle.calculate_needed_value(loss);
                BalanceChange::Loss(quote_oracle.calculate_needed_quantity(value))
            }
        }
    }

    fn calculate_funding_fee(&self, receipt: &Receipt) -> BalanceChange {
        let funding_change = match receipt.side {
            Side::Long => self.funding.base - receipt.initial_funding,
            Side::Short => self.funding.quote - receipt.initial_funding,
        };

        if funding_change > FundingRate::from_integer(0) {
            BalanceChange::Loss(receipt.size * funding_change)
        } else {
            BalanceChange::Profit(
                receipt
                    .size
                    .big_mul(FundingRate::from_integer(0) - funding_change),
            )
        }
    }
}

impl Trade {
    pub fn max_open_leverage(&self) -> Fraction {
        self.max_open_leverage
    }

    pub fn open_fee(&self) -> Fraction {
        self.open_fee
    }

    pub fn borrow_fee(&self, side: Side) -> Result<Fraction, LibErrors> {
        match side {
            Side::Long => self.borrow_fee.base.get_point_fee(self.utilization().base),
            Side::Short => self
                .borrow_fee
                .quote
                .get_point_fee(self.utilization().quote),
        }
    }

    #[cfg(test)]
    pub fn new_test() -> Trade {
        Trade::new(
            Fraction::new(100),
            Fraction::from_integer(3),
            0,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )
    }

    #[cfg(test)]
    pub fn new_test_for_long() -> Trade {
        let mut trade = Trade::new(
            Fraction::new(100),
            Fraction::from_integer(3),
            0,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        );

        trade.add_available_base(Quantity::new(200000000));
        trade.add_available_quote(Quantity::new(400000000));

        trade
    }
}

#[cfg(test)]
mod global {
    use super::*;

    #[test]
    fn creates_trade_service() {
        let trade = Trade::new_test();

        assert_eq!(trade.collateral_ratio(), Fraction::from_integer(1));
        assert_eq!(trade.liquidation_threshold(), Fraction::from_integer(1));
        assert_eq!(trade.utilization(), BothFractions::default());
        assert_eq!(trade.open_fee(), Fraction::new(100));
        assert_eq!(trade.max_open_leverage(), Fraction::from_integer(3))
    }

    #[test]
    fn calculates_utilization() {
        let mut trade = Trade::new_test();

        trade.add_available_base(Quantity::new(20565400));
        trade.add_available_quote(Quantity::new(40000000));

        trade.locked.base += Quantity::new(15442000); // it is what happens in open long
        trade.locked.quote += Quantity::new(12735500); // it is what happens in open short

        trade.remove_available_base(Quantity::new(15442000)); // it is what happens after open_long
        trade.remove_available_quote(Quantity::new(12735500)); // it is what happens after open_short

        assert_eq!(
            trade.utilization(),
            BothFractions {
                base: Fraction::new(750873),
                quote: Fraction::new(318388)
            }
        );
    }
}

#[cfg(test)]
mod long_tests {
    use crate::core_lib::decimal::{DecimalPlaces, Price};
    use checked_decimal_macro::{Decimal, Factories};

    use super::*;

    #[test]
    fn opens_long() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_long(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
        )?;

        assert_eq!(res.side, Side::Long);
        assert_eq!(res.size, Quantity::new(2000000));
        assert_eq!(res.locked, Quantity::new(2000000));
        assert_eq!(res.open_price, Price::from_integer(2));
        assert_eq!(res.open_value, Value::from_integer(4));
        assert_eq!(
            trade.locked(),
            Balances {
                base: Quantity::new(2000000),
                quote: Quantity::new(0)
            }
        );

        trade.remove_available_base(Quantity::new(2000000));

        assert_eq!(
            trade.utilization(),
            BothFractions {
                base: Fraction::new(10000),
                quote: Fraction::new(0)
            }
        );

        assert_eq!(
            trade.long_fees(&res),
            BalanceChange::Loss(Quantity::new(200))
        );

        assert_eq!(
            trade.calculate_long_change(&res, &base_oracle),
            BalanceChange::Loss(Quantity::new(0))
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(200)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(400000)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), false);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(0)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(0)));

        Ok(())
    }

    #[test]
    fn long_price_change() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let mut base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_long(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
        )?;

        base_oracle.update(Price::new(2000000010), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_down(
                Quantity::new(2000000),
                Price::new(2000000010),
                Price::new(2000000000)
            ),
            Value::new(20)
        );

        assert_eq!(
            trade.calculate_long_change(&res, &base_oracle),
            BalanceChange::Profit(Quantity::new(0))
        );

        assert_eq!(
            base_oracle.calculate_needed_value(Quantity::new(200)),
            Value::new(400001)
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(200)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(400001)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), false);

        assert_eq!(balance_change, BalanceChange::Profit(Quantity::new(0)));
        assert_eq!(value_change, ValueChange::Profitable(Value::new(0)));

        base_oracle.update(Price::new(2100000000), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_down(
                Quantity::new(2000000),
                Price::new(2100000000),
                Price::new(2000000000)
            ),
            Value::new(200000000)
        );

        assert_eq!(
            trade.calculate_long_change(&res, &base_oracle),
            BalanceChange::Profit(Quantity::new(95238))
        );

        assert_eq!(
            base_oracle.calculate_needed_value(Quantity::new(200)),
            Value::new(420000)
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), true);

        assert_eq!(balance_change, BalanceChange::Profit(Quantity::new(95038))); // -200 fee
        assert_eq!(value_change, ValueChange::Profitable(Value::new(199579800)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), false);

        assert_eq!(balance_change, BalanceChange::Profit(Quantity::new(95238))); // without fee
        assert_eq!(value_change, ValueChange::Profitable(Value::new(199999800)));

        base_oracle.update(Price::new(1900000000), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_down(
                Quantity::new(2000000),
                Price::new(2000000000),
                Price::new(1900000000)
            ),
            Value::new(200000000)
        );

        assert_eq!(
            trade.calculate_long_change(&res, &base_oracle),
            BalanceChange::Loss(Quantity::new(105264))
        );

        assert_eq!(
            base_oracle.calculate_needed_value(Quantity::new(200)),
            Value::new(380000)
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(105464))); // -200 fee
        assert_eq!(value_change, ValueChange::Loss(Value::new(200381600)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), false);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(105264))); // without fee
        assert_eq!(value_change, ValueChange::Loss(Value::new(200001600)));

        Ok(())
    }

    #[test]
    fn close_long() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let mut base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_long(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
        )?;

        // SAME PRICE, returns only fees to pay

        let (pos_change, to_unlock) = trade.close_long(&res, &base_oracle)?;

        assert_eq!(pos_change, BalanceChange::Loss(Quantity::new(200)));
        assert_eq!(to_unlock, Quantity::new(2000000));

        let res = trade.open_long(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
        )?;

        // price +5%
        base_oracle.update(Price::new(2100000000), Price::new(21000000), 0)?;

        let (pos_change, to_unlock) = trade.close_long(&res, &base_oracle)?;

        assert_eq!(pos_change, BalanceChange::Profit(Quantity::new(95038)));
        assert_eq!(to_unlock, Quantity::new(2000000));

        base_oracle.update(Price::new(2000000000), Price::new(21000000), 0)?;

        let res = trade.open_long(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
        )?;

        // price -5%
        base_oracle.update(Price::new(1900000000), Price::new(2000000), 0)?;

        let (pos_change, to_unlock) = trade.close_long(&res, &base_oracle)?;

        assert_eq!(pos_change, BalanceChange::Loss(Quantity::new(105464)));
        assert_eq!(to_unlock, Quantity::new(2000000));

        Ok(())
    }

    #[test]
    fn collateralization_limits_trade() {
        let mut trade = Trade::new_test_for_long();

        let base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        assert!(trade
            .open_long(
                Quantity::new(15000001),
                Value::from_integer(10),
                &base_oracle,
            )
            .is_err());

        assert!(trade
            .open_long(
                Quantity::new(15000000),
                Value::from_integer(10),
                &base_oracle,
            )
            .is_ok())
    }

    #[test]
    fn opens_short() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let quote_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::new(1000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_short(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
            &quote_oracle,
        )?;

        assert_eq!(res.side, Side::Short);
        assert_eq!(res.size, Quantity::new(2000000));
        assert_eq!(res.locked, Quantity::new(4000000));
        assert_eq!(res.open_price, Price::from_integer(2));
        assert_eq!(res.open_value, Value::from_integer(4));
        assert_eq!(
            trade.locked(),
            Balances {
                base: Quantity::new(0),
                quote: Quantity::new(4000000)
            }
        );

        trade.remove_available_quote(Quantity::new(4000000));

        assert_eq!(
            trade.utilization(),
            BothFractions {
                base: Fraction::new(0),
                quote: Fraction::new(10000)
            }
        );

        assert_eq!(
            trade.short_fees(&res, &base_oracle, &quote_oracle),
            BalanceChange::Loss(Quantity::new(400))
        );

        assert_eq!(
            trade.calculate_short_change(&res, &base_oracle, &quote_oracle),
            BalanceChange::Loss(Quantity::new(0))
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(400)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(400000)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &base_oracle.clone(), false);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(0)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(0)));

        Ok(())
    }

    #[test]
    fn price_changes_short() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let mut base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let quote_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::new(1000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_short(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
            &quote_oracle,
        )?;

        base_oracle.update(Price::new(2000000010), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_up(
                Quantity::new(2000000),
                Price::new(2000000010),
                Price::new(2000000000)
            ),
            Value::new(20)
        );

        assert_eq!(
            quote_oracle.calculate_needed_quantity(Value::new(20)),
            Quantity::new(1)
        );

        assert_eq!(
            trade.calculate_short_change(&res, &base_oracle, &quote_oracle),
            BalanceChange::Loss(Quantity::new(1))
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(401)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(401000)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, false);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(1)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(1000)));

        base_oracle.update(Price::new(2100000000), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_up(
                Quantity::new(2000000),
                Price::new(2100000000),
                Price::new(2000000000)
            ),
            Value::new(200000000)
        );

        assert_eq!(
            quote_oracle.calculate_needed_quantity(Value::new(200000000)),
            Quantity::new(200000)
        );

        assert_eq!(
            trade.calculate_short_change(&res, &base_oracle, &quote_oracle),
            BalanceChange::Loss(Quantity::new(200000))
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, true);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(200400)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(200400000)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, false);

        assert_eq!(balance_change, BalanceChange::Loss(Quantity::new(200000)));
        assert_eq!(value_change, ValueChange::Loss(Value::new(200000000)));

        base_oracle.update(Price::new(1900000000), Price::new(21000000), 0)?;

        assert_eq!(
            base_oracle.calculate_value_difference_down(
                Quantity::new(2000000),
                Price::new(2000000000),
                Price::new(1900000000)
            ),
            Value::new(200000000)
        );

        assert_eq!(
            quote_oracle.calculate_quantity(Value::new(200000000)),
            Quantity::new(200000)
        );

        assert_eq!(
            trade.calculate_short_change(&res, &base_oracle, &quote_oracle),
            BalanceChange::Profit(Quantity::new(200000))
        );

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, true);

        assert_eq!(balance_change, BalanceChange::Profit(Quantity::new(199600)));
        assert_eq!(value_change, ValueChange::Profitable(Value::new(199600000)));

        let (balance_change, value_change) =
            trade.calculate_position(&res, &base_oracle, &quote_oracle, false);

        assert_eq!(balance_change, BalanceChange::Profit(Quantity::new(200000)));
        assert_eq!(value_change, ValueChange::Profitable(Value::new(200000000)));

        Ok(())
    }

    #[test]
    fn close_short() -> Result<(), LibErrors> {
        let mut trade = Trade::new_test_for_long();

        let mut base_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(2000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let quote_oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::new(1000000),
            Price::from_scale(2, 2),
            0,
            0,
        );

        let res = trade.open_short(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
            &quote_oracle,
        )?;

        // SAME PRICE, returns only fees to pay

        let (pos_change, to_unlock) = trade.close_short(&res, &base_oracle, &quote_oracle)?;

        assert_eq!(pos_change, BalanceChange::Loss(Quantity::new(400)));
        assert_eq!(to_unlock, Quantity::new(4000000));

        let res = trade.open_short(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
            &quote_oracle,
        )?;

        // price +5%
        base_oracle.update(Price::new(2100000000), Price::new(21000000), 0)?;

        let (pos_change, to_unlock) = trade.close_short(&res, &base_oracle, &quote_oracle)?;

        assert_eq!(pos_change, BalanceChange::Loss(Quantity::new(200400)));
        assert_eq!(to_unlock, Quantity::new(4000000));

        base_oracle.update(Price::new(2000000000), Price::new(21000000), 0)?;

        let res = trade.open_short(
            Quantity::new(2000000),
            Value::from_integer(10),
            &base_oracle,
            &quote_oracle,
        )?;

        // price -5%
        base_oracle.update(Price::new(1900000000), Price::new(2000000), 0)?;

        let (pos_change, to_unlock) = trade.close_short(&res, &base_oracle, &quote_oracle)?;

        assert_eq!(pos_change, BalanceChange::Profit(Quantity::new(199600)));
        assert_eq!(to_unlock, Quantity::new(4000000));

        Ok(())
    }
}
