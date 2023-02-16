use super::{
    utils::{CollateralValues, TradeResult},
    *,
};
use crate::structs::FixedSizeVector;
use checked_decimal_macro::Decimal;

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

    fn liabilities_value(&self, vaults: &[Vault]) -> Value {
        if let Some(iter) = self.positions.iter() {
            iter.filter(|&pos| pos.is_liability())
                .fold(Value::new(0), |sum, curr| {
                    sum + curr.liability_value(vaults)
                })
        } else {
            Value::new(0)
        }
    }

    /// Vault's oracles should be refreshed before using this function
    fn collaterals_values(&self, vaults: &[Vault]) -> CollateralValues {
        if let Some(iter) = self.positions.iter() {
            iter.filter(|&pos| pos.is_collateral())
                .fold(CollateralValues::default(), |sum, curr| {
                    sum + curr.collateral_values(vaults)
                })
        } else {
            CollateralValues::default()
        }
    }

    fn trades_values(&self, vaults: &[Vault]) -> TradeResult {
        if let Some(iter) = self.positions.iter() {
            iter.filter(|&pos| pos.is_trade())
                .fold(TradeResult::Profitable(Value::new(0)), |sum, curr| {
                    sum + curr.position_profit(vaults)
                })
        } else {
            TradeResult::None
        }
    }
}
