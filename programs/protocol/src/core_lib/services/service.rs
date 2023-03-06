use crate::core_lib::decimal::*;

pub trait ServiceUpdate {
    /// Increases both balance and available liquidity.
    fn add_liquidity_base(&mut self, quantity: Quantity);
    fn add_liquidity_quote(&mut self, quantity: Quantity);
    fn remove_liquidity_base(&mut self, quantity: Quantity);
    fn remove_liquidity_quote(&mut self, quantity: Quantity);

    /// Increases available liquidity due to deposits, unlocks or profit
    fn add_available_base(&mut self, quantity: Quantity);
    fn add_available_quote(&mut self, quantity: Quantity);

    /// Decreases available liquidity due to withdrawals, locks or loss
    fn remove_available_base(&mut self, quantity: Quantity);
    fn remove_available_quote(&mut self, quantity: Quantity);

    /// Returns amount of liquidity available in a service (sum of all strategies).
    fn available(&self) -> Balances;

    /// Returns amount of used liquidity (like borrowed or sold).
    /// The fees are be included only if the `accrue_fee` method was called before
    fn locked(&self) -> Balances;

    /// Returns fees that are to be distributed to strategies and resets it.
    fn accrue_fee(&mut self) -> Balances;
}
