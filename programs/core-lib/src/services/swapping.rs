use checked_decimal_macro::{BetweenDecimals, Factories};

use crate::decimal::{Balances, Fraction, Quantity};
use crate::structs::{FeeCurve, Oracle};

use super::ServiceUpdate;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Swap {
    /// Liquidity available to be bought by a swapper.
    available: Balances,
    /// Current balance, greater or equal to available.
    balances: Balances,

    /// Total amount of tokens earned for liquidity providers.
    total_earned_fee: Balances,
    /// Total amount of tokens already paid for liquidity providers.
    total_paid_fee: Balances,
    /// Total amount of fee tokens kept as fee (insurance, PoL or burn).
    total_kept_fee: Balances,

    /// Struct for calculation of swapping fee.
    selling_fee: FeeCurve,
    buying_fee: FeeCurve,
    /// Fraction of paid fee to be kept.
    kept_fee: Fraction,
}

impl ServiceUpdate for Swap {
    fn add_liquidity_base(&mut self, quantity: Quantity) {
        self.available.base += quantity;
        self.balances.base += quantity;
    }

    fn add_liquidity_quote(&mut self, quantity: Quantity) {
        self.available.quote += quantity;
        self.balances.quote += quantity;
    }

    fn remove_liquidity_base(&mut self, quantity: Quantity) {
        self.available.base -= quantity;
        self.balances.base -= quantity;
    }

    fn remove_liquidity_quote(&mut self, quantity: Quantity) {
        self.available.quote -= quantity;
        self.balances.quote -= quantity;
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
        Balances::default()
    }

    fn accrue_fee(&mut self) -> Balances {
        let diff = self.total_earned_fee - self.total_paid_fee;
        self.total_paid_fee = self.total_earned_fee;
        diff
    }
}

impl Swap {
    pub fn new(selling_fee: FeeCurve, buying_fee: FeeCurve, kept_fee: Fraction) -> Swap {
        let swap = Self {
            available: Balances::default(),
            balances: Balances::default(),
            total_earned_fee: Balances::default(),
            total_paid_fee: Balances::default(),
            total_kept_fee: Balances::default(),
            selling_fee,
            buying_fee,
            kept_fee,
        };

        swap
    }

    fn get_proportion(&self, base_oracle: &Oracle, quote_oracle: &Oracle) -> Fraction {
        let base_value = base_oracle.calculate_value(self.balances.base);
        let quote_value = quote_oracle.calculate_value(self.balances.quote);
        let base_proportion = base_value / (base_value + quote_value);
        Fraction::from_decimal(base_proportion)
    }

    pub fn sell(
        &mut self,
        base_quantity: Quantity,
        base_oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> Result<Quantity, ()> {
        let proportion_before = self.get_proportion(base_oracle, quote_oracle);
        let swap_value = base_oracle.calculate_value(base_quantity);
        let quote_quantity = quote_oracle.calculate_quantity(swap_value);
        if quote_quantity > self.available.quote {
            return Err(());
        }

        self.balances.base += base_quantity;
        self.balances.quote -= quote_quantity;
        let proportion_after = self.get_proportion(base_oracle, quote_oracle);

        let fee_fraction = self
            .selling_fee
            .get_mean(proportion_before, proportion_after)?;

        let fee = quote_quantity * fee_fraction;
        let fee_to_keep = fee * self.kept_fee;
        self.total_kept_fee.base = fee_to_keep;
        self.total_earned_fee.base += fee - fee_to_keep;

        Ok(quote_quantity - fee)
    }

    pub fn buy(
        &mut self,
        quote_quantity: Quantity,
        base_oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> Result<Quantity, ()> {
        let proportion_before =
            Fraction::from_integer(1) - self.get_proportion(base_oracle, quote_oracle);
        let swap_value = quote_oracle.calculate_value(quote_quantity);
        let base_quantity = base_oracle.calculate_quantity(swap_value);
        if base_quantity > self.available.base {
            return Err(());
        }

        self.balances.quote += quote_quantity;
        self.balances.base -= base_quantity;
        let proportion_after =
            Fraction::from_integer(1) - self.get_proportion(base_oracle, quote_oracle);

        let fee_fraction = self
            .buying_fee
            .get_mean(proportion_before, proportion_after)?;

        let fee = base_quantity * fee_fraction;
        let fee_to_keep = fee * self.kept_fee;
        self.total_kept_fee.base = fee_to_keep;
        self.total_earned_fee.base += fee - fee_to_keep;

        Ok(base_quantity - fee)
    }

    pub fn fee_curve_sell(&mut self) -> &mut FeeCurve {
        &mut self.selling_fee
    }

    pub fn fee_curve_buy(&mut self) -> &mut FeeCurve {
        &mut self.buying_fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_with_fee() {
        let base_oracle = Oracle::new_for_test();
        let quote_oracle = Oracle::new_stable_for_test();

        let input = Quantity::from_integer(2_000000);

        // basic free swap
        {
            let mut swap = Swap::default();
            swap.fee_curve_buy()
                .add_constant_fee(Fraction::from_integer(0), Fraction::from_integer(1)); // 0% fee

            swap.add_liquidity_base(Quantity(10_000000));
            swap.add_liquidity_quote(Quantity(20_000000));

            let result = swap.buy(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(1_000000)));
        }

        // basic swap with constant fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(1_000000));
            swap.add_liquidity_quote(Quantity(5_000000));

            swap.fee_curve_buy()
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_integer(1)); // 1% fee

            let result = swap.buy(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(990000)));
        }

        // basic swap with constant fee from 0 to 0
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(1_000000));
            swap.add_liquidity_quote(Quantity(0));

            swap.fee_curve_buy()
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_integer(1)); // 1% fee

            let result = swap.buy(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(990000)));
        }

        // basic swap with changing fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(5_000000));
            swap.add_liquidity_quote(Quantity(10_000000));

            swap.fee_curve_buy()
                .add_constant_fee(Fraction::from_scale(3, 3), Fraction::from_scale(45, 2)) // 0.3% fee
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_scale(40, 1)); // 1% fee

            let result = swap.buy(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(990000)));
        }

        // swap with linear fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(5_000000));
            swap.add_liquidity_quote(Quantity(10_000000));

            swap.fee_curve_buy().add_linear_fee(
                Fraction::from_scale(3, 3),
                Fraction::from_scale(0, 3),
                Fraction::from_integer(1),
            ); // 0.3% * proportion + 0.1% fee

            let result = swap.buy(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(998350)));
        }
    }

    #[test]
    fn test_sell_with_fee() -> Result<(), ()> {
        let base_oracle = Oracle::new_for_test();
        let quote_oracle = Oracle::new_stable_for_test();

        let input = Quantity::from_integer(1_000000);

        // basic free swap
        {
            let mut swap = Swap::default();
            swap.fee_curve_sell()
                .add_constant_fee(Fraction::from_integer(0), Fraction::from_integer(1)); // 0% fee

            swap.add_liquidity_base(Quantity(10_000000));
            swap.add_liquidity_quote(Quantity(20_000000));

            let result = swap.sell(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(2_000000)));
        }

        // basic swap with constant fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(1_000000));
            swap.add_liquidity_quote(Quantity(5_000000));

            swap.fee_curve_sell()
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_integer(1)); // 1% fee

            let result = swap.sell(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(1_980000)));
        }

        // basic swap with constant fee from 0 to 0
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(0));
            swap.add_liquidity_quote(Quantity(2_000000));

            swap.fee_curve_sell()
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_integer(1)); // 1% fee

            let result = swap.sell(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(1_980000)));
        }

        // basic swap with changing fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(5_000000));
            swap.add_liquidity_quote(Quantity(10_000000));

            swap.fee_curve_sell()
                .add_constant_fee(Fraction::from_scale(3, 3), Fraction::from_scale(55, 2)) // 0.3% fee
                .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_scale(6, 1)); // 1% fee

            let result = swap.sell(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(1_987000)));
        }

        // swap with linear fee
        {
            let mut swap = Swap::default();
            swap.add_liquidity_base(Quantity(5_000000));
            swap.add_liquidity_quote(Quantity(10_000000));

            swap.fee_curve_sell().add_linear_fee(
                Fraction::from_scale(3, 3),
                Fraction::from_scale(0, 3),
                Fraction::from_integer(1),
            ); // 0.3% * proportion + 0.1% fee

            let result = swap.sell(input, &base_oracle, &quote_oracle);
            assert_eq!(result, Ok(Quantity::from_integer(1_996700)));
        }

        Ok(())
    }
}
