use super::*;
use std::ops::Add;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Debug, Default)]
    #[repr(C)]
    pub struct CollateralValues {
        /// value of collateral 1:1
        pub exact: Value,
        /// value of collateral with collateral ratio
        pub with_collateral_ratio: Value,
        /// value of collateral with liquidation threshold ratio
        pub unhealthy: Value,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, Default)]
    #[repr(C)]
    pub struct CollateralValues {
        /// value of collateral 1:1
        pub exact: Value,
        /// value of collateral with collateral ratio
        pub with_collateral_ratio: Value,
        /// value of collateral with liquidation threshold ratio
        pub unhealthy: Value,
    }
}

#[cfg(feature = "anchor")]
pub use zero::*;

#[cfg(not(feature = "anchor"))]
pub use non_zero::*;

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

#[derive(Default, PartialEq, Debug)]
#[repr(u8)]
pub enum ValueChange {
    #[default]
    None,
    Profitable(Value),
    Loss(Value),
}
