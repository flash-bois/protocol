use crate::core_lib::decimal::{DecimalPlaces, Price, Quantity, Time, Value};
use checked_decimal_macro::{BetweenDecimals, Decimal, Factories, Others};

pub enum OraclePriceType {
    Spot,
    Sell,
    Buy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(packed)]
/// Oracle is a struct that holds the price of an asset.
pub struct Oracle {
    /// The price of the asset.
    price: Price,
    /// The confidence of the price. It is a range around the price.
    confidence: Price,
    /// The time of the last update.
    last_update: Time,
    /// The maximum time interval between updates.
    max_update_interval: Time,
    /// If true, the oracle will force use the spread instead of the spot price.
    use_spread: bool,
    /// Limit of quotient above which the confidence is too great to use spot price.
    spread_limit: Price,
    /// The number of decimals of the asset.
    decimals: DecimalPlaces,
}

impl Oracle {
    /// Creates a new Oracle with the given price and confidence.
    pub fn new(
        decimals: DecimalPlaces,
        price: Price,
        confidence: Price,
        spread_limit: Price,
        time: Time,
    ) -> Result<Self, ()> {
        let mut oracle = Self {
            price: Price::from_integer(0),
            confidence: Price::from_integer(0),
            last_update: 0,
            max_update_interval: 0,
            use_spread: false,
            decimals,
            spread_limit,
        };
        oracle.update(price, confidence, time)?;
        Ok(oracle)
    }

    /// Updates the price and confidence of the oracle.
    pub fn update(&mut self, price: Price, confidence: Price, time: Time) -> Result<(), ()> {
        if confidence > price {
            return Err(());
        }
        self.price = price;
        self.confidence = confidence;
        self.last_update = time;
        Ok(())
    }

    /// Checks if the oracle has been updated in the last `max_update_interval` seconds.
    pub fn check_if_updated(&self, now: Time) -> Result<(), ()> {
        if now - self.last_update <= self.max_update_interval {
            return Ok(());
        }
        Err(())
    }

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
        self.use_spread || self.confidence / self.price > self.spread_limit
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
        quantity.div_up(price)
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
                .mul_up(self.price(OraclePriceType::Sell)),
        )
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
        .unwrap()
    }

    pub fn new_stable_for_test() -> Self {
        Self::new(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test_oracle {
    use checked_decimal_macro::{Decimal, Factories};

    use crate::core_lib::decimal::{DecimalPlaces, Price, Quantity, Value};

    use super::Oracle;

    #[test]
    fn test_calculate_value() {
        let mut oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap();

        assert_eq!(
            oracle.calculate_value(Quantity(100_000000)),
            Value::from_integer(200)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1),),
            Value::from_scale(2, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1_000000_000000),),
            Value::from_integer(2_000000)
        );

        oracle
            .update(Price::from_integer(50000), Price::from_scale(2, 3), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_value(Quantity(100_000000),),
            Value::from_integer(5000000)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1),),
            Value::from_scale(50000, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1_000000_000000),),
            Value::from_integer(50000000000u64)
        );

        oracle
            .update(Price::from_scale(2, 6), Price::from_scale(1, 9), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_value(Quantity(100_000000),),
            Value::from_scale(200, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1),),
            Value::from_scale(0, 6)
        );
        assert_eq!(
            oracle.calculate_value(Quantity(1_000000_000000),),
            Value::from_integer(2u64)
        );

        let oracle = Oracle::new(
            DecimalPlaces::Nine,
            Price::from_scale(2, 6),
            Price::from_scale(1, 9),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap();
        assert_eq!(
            oracle.calculate_value(Quantity(1_000000_000000000),),
            Value::from_integer(2u64)
        );
    }

    #[test]
    fn test_calculate_needed_value() {
        let mut oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_scale(5, 1),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap();

        assert_eq!(
            oracle.calculate_needed_value(Quantity(100_000000),),
            Value::from_integer(200)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1),),
            Value::from_scale(2, 6)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1_000000_000000)),
            Value::from_integer(2_000000)
        );

        oracle
            .update(Price::from_integer(50000), Price::from_scale(2, 3), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_needed_value(Quantity(100_000000),),
            Value::from_scale(2, 3)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1),),
            Value::from_scale(1, Value::scale())
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1_000000_000000)),
            Value::from_integer(20)
        );

        oracle
            .update(Price::from_scale(2, 6), Price::from_scale(1, 9), 0)
            .unwrap();

        assert_eq!(
            oracle.calculate_needed_value(Quantity(100_000000)),
            Value::from_integer(50000000)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1)),
            Value::from_scale(5, 1)
        );
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1_000000_000000)),
            Value::from_integer(500000_000000u64)
        );

        let oracle = Oracle::new(
            DecimalPlaces::Nine,
            Price::from_scale(2, 6),
            Price::from_scale(1, 9),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap();
        assert_eq!(
            oracle.calculate_needed_value(Quantity(1_000000_000000000)),
            Value::from_integer(500000_000000u64)
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
        )
        .unwrap();

        assert_eq!(
            oracle.calculate_quantity(Value::from_integer(200)),
            Quantity(100_000000)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_scale(2, 6)),
            Quantity(1)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_integer(2_000000)),
            Quantity(1_000000_000000)
        );
        assert_eq!(
            oracle.calculate_quantity(Value::from_scale(1, 6)),
            Quantity(0)
        );
    }

    #[test]
    fn test_calculate_needed_quantity() {
        let oracle = Oracle::new(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(1, 3),
            Price::from_scale(5, 3),
            0,
        )
        .unwrap();

        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_integer(100)),
            Quantity(200_000000)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_scale(1, 6)),
            Quantity(2)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_integer(1_000000)),
            Quantity(2_000000_000000)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_scale(1, 6)),
            Quantity(2)
        );
        assert_eq!(
            oracle.calculate_needed_quantity(Value::from_scale(1, 7)),
            Quantity(1)
        );
    }
}
