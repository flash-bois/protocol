use super::*;

use crate::structs::FixedSizeVector;
use std::ops::Add;

#[derive(Clone, Debug, Default)]
struct CollateralValues {
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
enum Trades {
    #[default]
    None,
    Profitable(Value),
    Loss(Value),
}

impl Add for Trades {
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

#[derive(Default)]
struct UserTemporaryValues {
    pub liabilities: Value,
    pub collateral: CollateralValues,
    // pub trades: Trades,
}

#[derive(Default)]
pub struct UserStatement {
    positions: FixedSizeVector<Position, 64>,
    values: UserTemporaryValues,
}

impl UserStatement {
    pub fn add_position(&mut self, position: Position) -> Result<(), ()> {
        self.positions.add(position)
    }

    pub fn search_mut(&mut self, position_search: &Position) -> Option<&mut Position> {
        self.positions.find_mut(position_search)
    }

    pub fn search_mut_id(&mut self, position_search: &Position) -> Option<(usize, &mut Position)> {
        self.positions.enumerate_find_mut(position_search)
    }

    pub fn delete_position(&mut self, id: usize) {
        self.positions.delete(id)
    }

    /// calculate value that user can borrow
    pub fn permitted_debt(&self) -> Value {
        self.values.collateral.with_collateral_ratio - self.values.liabilities
    }
}
