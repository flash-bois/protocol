use super::*;
use std::ops::Add;

#[derive(Clone, Debug, Default)]
pub struct CollateralValues {
    /// value of collateral 1:1
    pub exact: Value,
    /// value of collateral with collateral ratio
    pub with_collateral_ratio: Value,
    /// value of collateral with liquidation threshold ratio
    pub unhealthy: Value,
}

impl Add for CollateralValues {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            exact: self.exact + other.exact,
            with_collateral_ratio: self.with_collateral_ratio + other.with_collateral_ratio,
            unhealthy: self.unhealthy + other.unhealthy,
        }
    }
}

#[derive(Default)]
#[repr(u8)]
pub enum TradeResult {
    #[default]
    None,
    Profitable(Value),
    Loss(Value),
}
