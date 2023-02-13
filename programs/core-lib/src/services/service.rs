use super::lending::Lend;
use crate::decimal::*;
use crate::structs::Oracle;

pub trait ServiceUpdate {
    /// Increases available liquidity, and calculates new utilization.
    fn add_available(&mut self, quantity: Quantity);

    /// Decreases available liquidity, and calculates new utilization.
    fn remove_available(&mut self, quantity: Quantity);

    /// Returns amount of liquidity available in a service (sum of all strategies).
    fn available(&self) -> Quantity;

    /// Returns amount of used liquidity (like borrowed or sold).
    /// The fees are be included only if the `accrue_fee` method was called before
    fn locked(&self) -> Quantity;

    /// Returns fees that are to be distributed to strategies and resets it.
    fn accrue_fee(&mut self, oracle: Option<&Oracle>) -> Quantity;
}

#[derive(Clone, Debug, Default)]
pub struct Services {
    pub lend: Option<Lend>,
}

enum ServiceType {
    /// Lending allows borrowing of tokens
    Lend,
    /// Swapping allows swapping tokens between each other
    Swap,
    /// Trading allows creating leveraged LONG and SHORT positions
    Trade,
}

impl Services {
    pub fn lend_service(&mut self) -> Result<&mut Lend, ()> {
        self.lend.as_mut().ok_or(())
    }
}
