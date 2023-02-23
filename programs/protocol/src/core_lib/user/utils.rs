use super::*;
use std::ops::Add;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Debug, Default)]
    #[repr(packed)]
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
    #[repr(packed)]
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
pub use mon_zero::*;

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

impl Add for TradeResult {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match self {
            Self::None => other,
            Self::Profitable(val) => match other {
                Self::Profitable(val_add) => Self::Profitable(val + val_add),
                Self::Loss(val_sub) if val_sub <= val => Self::Profitable(val - val_sub),
                Self::Loss(val_sub) if val_sub > val => Self::Loss(val_sub - val),
                _ => unreachable!(),
            },
            Self::Loss(val) => match other {
                Self::Loss(val_add) => Self::Loss(val + val_add),
                Self::Profitable(val_sub) if val_sub <= val => Self::Loss(val - val_sub),
                Self::Profitable(val_sub) if val_sub > val => Self::Profitable(val_sub - val),
                _ => unreachable!(),
            },
        }
    }
}
