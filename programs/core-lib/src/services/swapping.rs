use crate::decimal::{Balances, Fraction, Quantity};
use crate::structs::FeeCurve;

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
    /// Total amount of fee tokens kept as fee (insurance or PoL).
    total_kept_fee: Balances,

    /// Struct for calculation of swapping fee.
    fee_curve: FeeCurve,
    /// Fraction of paid fee to be kept.
    kept_fee: Fraction,
}

impl ServiceUpdate for Swap {
    fn add_liquidity(&mut self, balances: Balances) {
        self.balances += balances;
        self.available += balances;
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
