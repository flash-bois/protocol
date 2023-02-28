use super::{
    utils::{CollateralValues, TradeResult},
    *,
};

use checked_decimal_macro::num_traits::ToPrimitive;
use checked_decimal_macro::Decimal;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};
use vec_macro::SafeArray;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(SafeArray, Debug)]
    pub struct Positions {
        pub head: u8,
        pub elements: [Position; 64],
    }

    #[zero_copy]
    #[derive(Default, Debug)]
    #[repr(C)]
    pub struct UserTemporaryValues {
        pub liabilities: Value,
        pub collateral: CollateralValues,
        // pub trades: Trades,
    }

    #[zero_copy]
    #[derive(Default, Debug)]
    #[repr(C)]
    pub struct UserStatement {
        pub positions: Positions,
        pub values: UserTemporaryValues,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(SafeArray, Clone, Copy, Debug)]
    pub struct Positions {
        pub head: u8,
        pub elements: [Position; 64],
    }

    #[derive(Default, Clone, Copy, Debug)]
    #[repr(C)]
    pub struct UserTemporaryValues {
        pub liabilities: Value,
        pub collateral: CollateralValues,
        // pub trades: Trades,
    }

    #[derive(Default, Clone, Copy, Debug)]
    #[repr(C)]
    pub struct UserStatement {
        pub positions: Positions,
        pub values: UserTemporaryValues,
    }
}

#[cfg(feature = "anchor")]
pub use zero::*;

#[cfg(not(feature = "anchor"))]
pub use non_zero::*;

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

    fn _trades_values(&self, vaults: &[Vault]) -> TradeResult {
        if let Some(iter) = self.positions.iter() {
            iter.filter(|&pos| pos.is_trade())
                .fold(TradeResult::Profitable(Value::new(0)), |sum, curr| {
                    sum + curr.position_profit(vaults)
                })
        } else {
            TradeResult::None
        }
    }

    /// calculates user temporary values for collateral and liabilities positions
    pub fn refresh(&mut self, vaults: &[Vault]) {
        self.values.liabilities = self.liabilities_value(vaults);
        self.values.collateral = self.collaterals_values(vaults);

        // TODO: handle trades
    }
}

#[cfg(test)]
mod position_management {
    use checked_decimal_macro::Factories;

    use crate::core_lib::{
        decimal::{DecimalPlaces, Fraction, Price, Utilization},
        structs::FeeCurve,
        vault::Token,
    };

    use super::*;

    #[test]
    fn default_positions() {
        let mut user_statement = UserStatement::default();

        assert!(user_statement.positions.iter().is_none(), "should be empty");

        let def_pos = Position::default();

        // add 5 positions with default values
        for _ in 0..5 {
            user_statement.add_position(def_pos.clone()).unwrap();
        }

        let elements = user_statement.positions.iter().unwrap();

        assert_eq!(elements.len(), 5);

        for pos in elements {
            assert_eq!(*pos, Position::Empty)
        }

        while let Some(pos) = user_statement.positions.remove() {
            assert_eq!(*pos, Position::Empty)
        }

        assert!(user_statement.positions.iter().is_none(), "should be empty");
    }

    #[test]
    fn add_position() {
        let mut user_statement = UserStatement::default();

        let mut first_vault = Vault::default();
        let mut second_vault = Vault::default();

        first_vault.id = 0;
        second_vault.id = 1;

        first_vault
            .enable_oracle(
                DecimalPlaces::Six,
                Price::from_integer(1),
                Price::from_scale(1, 5),
                Price::from_scale(5, 3),
                0,
                Token::Base,
            )
            .unwrap();

        first_vault
            .enable_oracle(
                DecimalPlaces::Six,
                Price::from_integer(1),
                Price::from_scale(1, 5),
                Price::from_scale(5, 3),
                0,
                Token::Quote,
            )
            .unwrap();

        first_vault
            .enable_lending(
                FeeCurve::default(),
                Utilization::from_scale(8, 1),
                Quantity::new(u64::MAX),
                0,
            )
            .unwrap();

        first_vault
            .add_strategy(
                true,
                false,
                false,
                Fraction::from_integer(1),
                Fraction::from_integer(1),
            )
            .unwrap();

        second_vault
            .enable_oracle(
                DecimalPlaces::Six,
                Price::from_integer(2),
                Price::from_scale(1, 5),
                Price::from_scale(5, 3),
                0,
                Token::Base,
            )
            .unwrap();

        second_vault
            .enable_oracle(
                DecimalPlaces::Six,
                Price::from_integer(1),
                Price::from_scale(1, 5),
                Price::from_scale(5, 3),
                0,
                Token::Quote,
            )
            .unwrap();

        second_vault
            .enable_lending(
                FeeCurve::default(),
                Utilization::from_scale(8, 1),
                Quantity::new(u64::MAX),
                0,
            )
            .unwrap();

        second_vault
            .add_strategy(
                true,
                false,
                false,
                Fraction::from_integer(1),
                Fraction::from_integer(1),
            )
            .unwrap();

        let mut vaults = [first_vault, second_vault];

        vaults[0]
            .deposit(
                &mut user_statement,
                Token::Base,
                Quantity::new(10000000),
                0,
                0,
            )
            .unwrap();

        vaults[1]
            .deposit(
                &mut user_statement,
                Token::Base,
                Quantity::new(5000000),
                0,
                0,
            )
            .unwrap();

        user_statement
            .add_position(Position::LiquidityProvide {
                vault_index: 0,
                strategy_index: 0,
                shares: Shares::new(5000000),
                amount: Quantity::new(5000000),
                quote_amount: Quantity::new(5000000),
            })
            .unwrap();

        user_statement.refresh(&mut vaults);

        vaults[0]
            .borrow(&mut user_statement, Quantity::new(5000000))
            .unwrap();

        user_statement.refresh(&mut vaults);

        vaults[1]
            .borrow(&mut user_statement, Quantity::new(4000000))
            .unwrap();

        assert_eq!(user_statement.positions.iter().unwrap().len(), 5);

        user_statement.refresh(&mut vaults);
        assert_eq!(
            user_statement.collaterals_values(&vaults).exact,
            Value::new(50000000000)
        );

        assert_eq!(
            user_statement.liabilities_value(&vaults),
            Value::new(13000000000)
        )
    }

    #[test]
    fn delete_position_in_the_middle() {
        let mut user_statement = UserStatement::default();

        let mut new_position = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1516),
            amount: Quantity::new(1718),
            quote_amount: Quantity::new(0),
        };

        user_statement
            .add_position(Position::LiquidityProvide {
                vault_index: 0,
                strategy_index: 0,
                shares: Shares::new(1234),
                amount: Quantity::new(5678),
                quote_amount: Quantity::new(0),
            })
            .unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(91011),
                amount: Quantity::new(121314),
            })
            .unwrap();

        user_statement.add_position(new_position.clone()).unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(1920),
                amount: Quantity::new(2122),
            })
            .unwrap();

        assert_eq!(user_statement.positions.iter().unwrap().len(), 4);

        user_statement.delete_position(1);

        assert_eq!(user_statement.positions.iter().unwrap().len(), 3);

        assert!(user_statement.positions.get_mut_checked(3).is_none());

        let new_on_index_1 = user_statement.positions.get_mut_checked(1).unwrap();

        assert_eq!(*new_on_index_1.shares(), *new_position.shares());
        assert_eq!(*new_on_index_1.amount(), *new_position.amount());
    }

    #[test]
    fn delete_position_in_the_end() {
        let mut user_statement = UserStatement::default();

        let mut new_position = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1516),
            amount: Quantity::new(1718),
            quote_amount: Quantity::new(0),
        };

        user_statement
            .add_position(Position::LiquidityProvide {
                vault_index: 0,
                strategy_index: 0,
                shares: Shares::new(1234),
                amount: Quantity::new(5678),
                quote_amount: Quantity::new(0),
            })
            .unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(91011),
                amount: Quantity::new(121314),
            })
            .unwrap();

        user_statement.add_position(new_position.clone()).unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(1920),
                amount: Quantity::new(2122),
            })
            .unwrap();

        assert_eq!(user_statement.positions.iter().unwrap().len(), 4);

        user_statement.delete_position(3);

        assert_eq!(user_statement.positions.iter().unwrap().len(), 3);

        assert!(user_statement.positions.get_mut_checked(3).is_none());

        let new_on_index_1 = user_statement.positions.get_mut_checked(2).unwrap();

        assert_eq!(*new_on_index_1.shares(), *new_position.shares());
        assert_eq!(*new_on_index_1.amount(), *new_position.amount());
    }

    #[test]
    fn delete_position_in_the_beginning() {
        let mut user_statement = UserStatement::default();

        let mut new_position = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1516),
            amount: Quantity::new(1718),
            quote_amount: Quantity::new(0),
        };

        user_statement
            .add_position(Position::LiquidityProvide {
                vault_index: 0,
                strategy_index: 0,
                shares: Shares::new(1234),
                amount: Quantity::new(5678),
                quote_amount: Quantity::new(0),
            })
            .unwrap();

        user_statement.add_position(new_position.clone()).unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(91011),
                amount: Quantity::new(121314),
            })
            .unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(1920),
                amount: Quantity::new(2122),
            })
            .unwrap();

        assert_eq!(user_statement.positions.iter().unwrap().len(), 4);

        user_statement.delete_position(0);

        assert_eq!(user_statement.positions.iter().unwrap().len(), 3);

        assert!(user_statement.positions.get_mut_checked(3).is_none());

        let new_on_index_1 = user_statement.positions.get_mut_checked(0).unwrap();

        assert_eq!(*new_on_index_1.shares(), *new_position.shares());
        assert_eq!(*new_on_index_1.amount(), *new_position.amount());
    }

    #[test]
    fn finding_position() {
        let mut user_statement = UserStatement::default();

        let mut search_position = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1516),
            amount: Quantity::new(1718),
            quote_amount: Quantity::new(0),
        };

        user_statement
            .add_position(Position::LiquidityProvide {
                vault_index: 0,
                strategy_index: 0,
                shares: Shares::new(1234),
                amount: Quantity::new(5678),
                quote_amount: Quantity::new(0),
            })
            .unwrap();

        user_statement
            .add_position(search_position.clone())
            .unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(91011),
                amount: Quantity::new(121314),
            })
            .unwrap();

        user_statement
            .add_position(Position::Borrow {
                vault_index: 0,
                shares: Shares::new(1920),
                amount: Quantity::new(2122),
            })
            .unwrap();

        let modified_search_position = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        let found_position = user_statement
            .search_mut(&modified_search_position)
            .unwrap();

        assert_eq!(*found_position.shares(), *search_position.shares());
        assert_eq!(*found_position.amount(), *search_position.amount());

        let modified_non_matching_search_position = Position::LiquidityProvide {
            vault_index: 1,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        assert!(user_statement
            .search_mut(&modified_non_matching_search_position)
            .is_none());

        assert_eq!(
            user_statement
                .search_mut_id(&modified_search_position)
                .unwrap()
                .0,
            1
        );

        assert!(user_statement
            .search_mut_id(&modified_non_matching_search_position)
            .is_none());
    }
}
