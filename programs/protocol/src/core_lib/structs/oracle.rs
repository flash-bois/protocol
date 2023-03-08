use crate::core_lib::{
    decimal::{DecimalPlaces, Price, Quantity, Time, Value},
    errors::LibErrors,
};
use checked_decimal_macro::{BetweenDecimals, BigOps, Decimal, Factories, Others};

#[repr(u8)]
pub enum OraclePriceType {
    Spot,
    Sell,
    Buy,
}

pub const DEFAULT_MAX_ORACLE_AGE: u32 = 100; // TODO refresh it in tests

#[cfg(feature = "anchor")]
mod zero {
    use crate::pyth::{get_oracle_update_from_acc, OracleUpdate};

    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq, Eq)]
    /// Oracle is a struct that holds the price of an asset.
    pub struct Oracle {
        /// The price of the asset.
        pub price: Price,
        /// The confidence of the price. It is a range around the price.
        pub confidence: Price,
        /// The time of the last update.
        pub last_update: u32,
        /// The maximum time interval between updates.
        pub max_update_interval: u32,
        /// If true, the oracle will force use the spread instead of the spot price.
        pub use_spread: u8,
        /// Limit of quotient above which the confidence is too great to use spot price.
        pub spread_limit: Price,
        /// The number of decimals of the asset.
        pub decimals: DecimalPlaces,
    }

    impl Oracle {
        pub fn update_from_acc(
            &mut self,
            acc: &AccountInfo,
            current_timestamp: i64,
        ) -> std::result::Result<(), LibErrors> {
            let OracleUpdate { price, conf, exp } =
                get_oracle_update_from_acc(acc, current_timestamp)?;

            let (price, confidence) = if exp < 0 {
                (
                    Price::from_scale(
                        price,
                        exp.abs().try_into().map_err(|_| LibErrors::ParseError)?,
                    ),
                    Price::from_scale(
                        conf,
                        exp.abs().try_into().map_err(|_| LibErrors::ParseError)?,
                    ),
                )
            } else {
                (
                    Price::from_integer(price).big_div(Price::from_scale(
                        1,
                        exp.try_into().map_err(|_| LibErrors::ParseError)?,
                    )),
                    Price::from_integer(conf).big_div(Price::from_scale(
                        1,
                        exp.try_into().map_err(|_| LibErrors::ParseError)?,
                    )),
                )
            };

            self.update(
                price,
                confidence,
                current_timestamp
                    .try_into()
                    .map_err(|_| LibErrors::ParseError)?,
            )
        }
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    #[repr(C)]
    /// Oracle is a struct that holds the price of an asset.
    pub struct Oracle {
        /// The price of the asset.
        pub price: Price,
        /// The confidence of the price. It is a range around the price.
        pub confidence: Price,
        /// The time of the last update.
        pub last_update: u32,
        /// The maximum time interval between updates.
        pub max_update_interval: u32,
        /// If true, the oracle will force use the spread instead of the spot price.
        pub use_spread: u8,
        /// Limit of quotient above which the confidence is too great to use spot price.
        pub spread_limit: Price,
        /// The number of decimals of the asset.
        pub decimals: DecimalPlaces,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Oracle;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Oracle;

impl Oracle {
    /// Creates a new Oracle with the given price and confidence.
    pub fn new(
        decimals: DecimalPlaces,
        price: Price,
        confidence: Price,
        spread_limit: Price,
        time: Time,
    ) -> Self {
        Self {
            price: price,
            confidence,
            last_update: time,
            max_update_interval: DEFAULT_MAX_ORACLE_AGE,
            use_spread: 0,
            decimals,
            spread_limit,
        }
    }

    /// Updates the price and confidence of the oracle.
    pub fn update(&mut self, price: Price, confidence: Price, time: Time) -> Result<(), LibErrors> {
        if confidence.big_div_up(price) > self.spread_limit {
            return Err(LibErrors::ConfidenceTooHigh);
        }
        self.price = price;
        self.confidence = confidence;
        self.last_update = time;
        Ok(())
    }

    // /// Checks if the oracle has been updated in the last `max_update_interval` seconds.
    // pub fn check_if_updated(&self, now: Time) -> Result<(), LibErrors> {
    //     if now - self.last_update <= self.max_update_interval {
    //         return Ok(());
    //     }
    //     Err(())
    // }

    /// Returns the price of the oracle. Depending on OraclePriceType, it can return the spot price,
    /// the sell price or the buy price.
    pub fn price(&self, which: OraclePriceType) -> Price {
        if !self.should_use_spread() {
            return self.price;
        }

        match which {
            OraclePriceType::Spot => self.price,
            OraclePriceType::Sell => self.price - self.confidence,
            OraclePriceType::Buy => self.price + self.confidence,
        }
    }

    /// Checks if either the confidence is too great to use spot price or the `use_spread` flag is
    /// set.
    pub fn should_use_spread(&self) -> bool {
        self.use_spread == 1 || self.confidence / self.price > self.spread_limit
    }

    /// Calculates the value for a given quantity of token.
    /// Can be understood as: How much value do I get for `quantity` of token?
    pub fn calculate_value(&self, quantity: Quantity) -> Value {
        let quantity = Value::from_scale(quantity.get(), self.decimals as u8);
        let price = self.price(OraclePriceType::Sell);
        quantity * price
    }

    /// Calculates the value that would be needed to get a given quantity of token.
    /// Can be understood as: How much value do I need to buy `quantity` of token?
    pub fn calculate_needed_value(&self, quantity: Quantity) -> Value {
        let quantity = Value::from_scale(quantity.get(), self.decimals as u8);
        let price = self.price(OraclePriceType::Buy);
        quantity.mul_up(price)
    }

    /// Calculates the quantity of token that can be bought for a given value.
    /// Can be understood as: How many tokens can I get for `value` of value?
    pub fn calculate_quantity(&self, value: Value) -> Quantity {
        Quantity::from_decimal(
            value
                / (Value::from_scale(1, self.decimals as u8))
                / (self.price(OraclePriceType::Buy)),
        )
        // CHECK ME: not sure if rounding errors wouldn't be a problem with precision
    }

    /// Calculates the value that would be needed to get a given quantity of token.
    /// Can be understood as: How much quantity of a token do I have to sell to get `value` of value?
    pub fn calculate_needed_quantity(&self, value: Value) -> Quantity {
        Quantity::from_decimal_up(
            value
                .div_up(Value::from_scale(1, self.decimals as u8))
                .div_up(self.price(OraclePriceType::Sell)),
        )
    }

    /// Calculates the value difference between two prices, rounding up.
    pub fn calculate_value_difference_up(
        &self,
        quantity: Quantity,
        greater: Price,
        smaller: Price,
    ) -> Value {
        let quantity = Value::from_scale_up(quantity.get(), self.decimals as u8);
        quantity.mul_up(greater - smaller)
    }

    /// Calculates the value difference between two prices, rounding down.
    pub fn calculate_value_difference_down(
        &self,
        quantity: Quantity,
        greater: Price,
        smaller: Price,
    ) -> Value {
        let quantity = Value::from_scale(quantity.get(), self.decimals as u8);
        quantity * (greater - smaller)
    }
}

#[cfg(test)]
impl Oracle {
    pub fn new_for_test() -> Self {
        Self::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
    }

    pub fn new_stable_for_test() -> Self {
        Self::new(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
    }
}

#[cfg(test)]
mod test_oracle {
    use checked_decimal_macro::{Decimal, Factories};

    use crate::core_lib::decimal::{DecimalPlaces, Price, Quantity, Value};

    use super::Oracle;

    #[test]
    fn test_update_oracle() {
        let mut oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(2, 2),
            0,
        );

        oracle
            .update(Price::new(5000000000), Price::new(25000000), 0)
            .unwrap();
    }

    #[test]
    fn test_calculate_value() {
        let mut oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        );

        assert_eq!(
            oracle.calculate_value(Quantity::new(100_000000)),
            Value::from_integer(200)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(100_000000)),
            Value::from_integer(200)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1),),
            Value::from_scale(2, 6)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1),),
            Value::from_scale(2, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1_000000_000000),),
            Value::from_integer(2_000000)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1_000000_000000),),
            Value::from_integer(2_000000)
        );

        oracle
            .update(Price::from_integer(50000), Price::from_scale(2, 3), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_value(Quantity::new(100_000000),),
            Value::from_integer(5000000)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(100_000000),),
            Value::from_integer(5000000)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1),),
            Value::from_scale(50000, 6)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1),),
            Value::from_scale(50000, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1_000000_000000),),
            Value::from_integer(50000000000u64)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1_000000_000000),),
            Value::from_integer(50000000000u64)
        );

        oracle
            .update(Price::from_scale(2, 6), Price::from_scale(1, 9), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_value(Quantity::new(100_000000),),
            Value::from_scale(200, 6)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(100_000000),),
            Value::from_scale(200, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1),),
            Value::from_scale(0, 6)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1),),
            Value::from_scale(1, 9)
        );
        assert_eq!(
            oracle.calculate_value(Quantity::new(1_000000_000000),),
            Value::from_integer(2u64)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1_000000_000000),),
            Value::from_integer(2u64)
        );

        let oracle = Oracle::new(
            DecimalPlaces::Nine,
            Price::from_scale(2, 6),
            Price::from_scale(1, 9),
            Price::from_scale(5, 3),
            0,
        );

        assert_eq!(
            oracle.calculate_value(Quantity::new(1_000000_000000000),),
            Value::from_integer(2u64)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity::new(1_000000_000000000),),
            Value::from_integer(2u64)
        );
    }

    #[test]
    fn test_calculate_quantity() {
        let oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        );

        assert_eq!(
            oracle.calculate_quantity(Value::from_integer(200)),
            Quantity::new(100_000000)
        );

        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_integer(200)),
            Quantity::new(100_000000)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_scale(2, 6)),
            Quantity::new(1)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_scale(2, 6)),
            Quantity::new(1)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_integer(2_000000)),
            Quantity::new(1_000000_000000)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_integer(2_000000)),
            Quantity::new(1_000000_000000)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_scale(1, 6)),
            Quantity::new(0)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_scale(1, 6)),
            Quantity::new(1)
        );
    }
}
