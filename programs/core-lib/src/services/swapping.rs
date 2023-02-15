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
    fn get_proportion(&self, base_oracle: &Oracle, quote_oracle: &Oracle) -> Fraction {
        let base_value = base_oracle.calculate_value(self.balances.base);
        let quote_value = quote_oracle.calculate_value(self.balances.quote);
        let base_proportion = base_value / (base_value + quote_value);
        Fraction::from_decimal(base_proportion)
    }

    pub fn sell(
        &mut self,
        quantity: Quantity,
        base_oracle: &Oracle,
        quote_oracle: &Oracle,
    ) -> Result<Quantity, ()> {
        let proportion_before = self.get_proportion(base_oracle, quote_oracle);
        self.balances.base += quantity;
        self.balances.quote -= quantity;
        let proportion_after = self.get_proportion(base_oracle, quote_oracle);

        let fee_fraction = self
            .selling_fee
            .get_mean(proportion_before, proportion_after)?;

        let fee = quantity * fee_fraction;
        let after_fee = quantity - fee;

        let fee_to_keep = fee * self.kept_fee;
        self.total_kept_fee.base = fee_to_keep;
        self.total_earned_fee.base += fee - fee_to_keep;

        let swap_value = base_oracle.calculate_value(after_fee);
        let quote_quantity = quote_oracle.calculate_quantity(swap_value);
        if quantity > self.available.quote {
            return Err(());
        }

        Ok(quote_quantity)
    }

    pub fn fee_curve_sell(&mut self) -> &mut FeeCurve {
        &mut self.selling_fee
    }

    pub fn fee_curve_buy(&mut self) -> &mut FeeCurve {
        &mut self.buying_fee
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
        assert_eq!(result, Ok(Quantity::from_integer(1_980000) - Quantity(2)));
    }

    // basic swap with constant fee from 0 to 0
    {
        let mut swap = Swap::default();
        swap.add_liquidity_base(Quantity(0));
        swap.add_liquidity_quote(Quantity(2_000000));

        swap.fee_curve_sell()
            .add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_integer(1)); // 1% fee

        let result = swap.sell(input, &base_oracle, &quote_oracle);
        assert_eq!(result, Ok(Quantity::from_integer(1_980000) - Quantity(2)));
    }

    Ok(())
}
