use super::{
    utils::{CollateralValues, TradeResult},
    *,
};
use crate::core_lib::{errors::LibErrors, services::ServiceUpdate, structs::Receipt};
use checked_decimal_macro::Decimal;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, u8)]
pub enum Position {
    #[default]
    Empty,
    LiquidityProvide {
        vault_index: u8,
        strategy_index: u8,
        shares: Shares,
        amount: Quantity,
        quote_amount: Quantity,
    },
    Borrow {
        vault_index: u8,
        shares: Shares,
        amount: Quantity,
    },
    Trading {
        vault_index: u8,
        receipt: Receipt,
    },
}

// user to compare user positions in vector, it is quick compare, by enum field
// and some of its subfields:
// LiquidityProvide: strategy index and vault index
// Borrow: vault index
//
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::LiquidityProvide {
                    vault_index,
                    strategy_index,
                    ..
                },
                Self::LiquidityProvide {
                    vault_index: vault_index_cmp,
                    strategy_index: strategy_index_cmp,
                    ..
                },
            ) => vault_index == vault_index_cmp && strategy_index == strategy_index_cmp,
            (
                Self::Borrow { vault_index, .. },
                Self::Borrow {
                    vault_index: vault_index_cmp,
                    ..
                },
            ) => vault_index == vault_index_cmp,
            (
                Self::Trading {
                    vault_index,
                    receipt,
                    ..
                },
                Self::Trading {
                    vault_index: vault_index_cmp,
                    receipt: receipt_cmp,
                    ..
                },
            ) => vault_index == vault_index_cmp && receipt.side == receipt_cmp.side,
            (Self::Empty, Self::Empty) => true,
            _ => false,
        }
    }
}

impl Position {
    pub fn is_liability(&self) -> bool {
        match self {
            Position::Borrow { .. } => true,
            Position::LiquidityProvide { .. } => false,
            Position::Trading { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    pub fn is_collateral(&self) -> bool {
        match self {
            Position::LiquidityProvide { .. } => true,
            Position::Borrow { .. } => false,
            Position::Trading { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    pub fn is_trade(&self) -> bool {
        match self {
            Position::Trading { .. } => true,
            Position::Borrow { .. } => false,
            Position::LiquidityProvide { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    fn shares_mut(&mut self) -> &mut Shares {
        match self {
            Position::Borrow { shares, .. } => shares,
            Position::LiquidityProvide { shares, .. } => shares,
            Position::Trading { .. } => panic!("trading does not have shares"),
            Position::Empty => unreachable!(),
        }
    }

    fn amount_mut(&mut self) -> &mut Quantity {
        match self {
            Position::Borrow { amount, .. } => amount,
            Position::LiquidityProvide { amount, .. } => amount,
            Position::Trading { receipt, .. } => &mut receipt.size,
            Position::Empty => unreachable!(),
        }
    }

    fn amount_quote_mut(&mut self) -> &mut Quantity {
        match self {
            Position::Borrow { .. } => unreachable!(),
            Position::LiquidityProvide { quote_amount, .. } => quote_amount,
            Position::Trading { .. } => unimplemented!(),
            Position::Empty => unreachable!(),
        }
    }

    pub fn shares(&self) -> &Shares {
        match self {
            Position::Borrow { shares, .. } => shares,
            Position::LiquidityProvide { shares, .. } => shares,
            Position::Trading { .. } => panic!("trading does not have shares"),
            Position::Empty => unreachable!(),
        }
    }

    pub fn amount(&self) -> &Quantity {
        match self {
            Position::Borrow { amount, .. } => amount,
            Position::LiquidityProvide { amount, .. } => amount,
            _ => unreachable!(),
        }
    }

    pub fn receipt(&mut self) -> &mut Receipt {
        match self {
            Position::Trading { receipt, .. } => receipt,
            _ => unreachable!(),
        }
    }

    pub fn increase_amount(&mut self, amount: Quantity) {
        *self.amount_mut() += amount
    }

    pub fn increase_quote_amount(&mut self, amount: Quantity) {
        *self.amount_quote_mut() += amount
    }

    pub fn increase_shares(&mut self, shares: Shares) {
        *self.shares_mut() += shares
    }

    pub fn decrease_amount(&mut self, amount: Quantity) {
        *self.amount_mut() -= amount
    }

    pub fn decrease_shares(&mut self, shares: Shares) {
        *self.shares_mut() -= shares
    }

    pub fn get_owed_single(&self, shares: &Shares, vault: &Vault) -> Result<Quantity, LibErrors> {
        let service = vault.lend_service_not_mut()?;

        Ok(service
            .borrow_shares()
            .calculate_owed(*shares, service.locked().base))
    }

    pub fn get_owed_double(
        &self,
        strategy_index: u8,
        shares: &Shares,
        vault: &Vault,
    ) -> Result<(Quantity, Quantity), LibErrors> {
        let strategy = vault.strategies.get_strategy(strategy_index)?;

        let base_quantity = strategy
            .total_shares()
            .calculate_earned(*shares, strategy.balance());

        let quote_quantity = strategy
            .total_shares()
            .calculate_earned(*shares, strategy.balance_quote());

        Ok((base_quantity, quote_quantity))
    }

    pub fn liability_value(&self, vaults: &[Vault]) -> Result<Value, LibErrors> {
        match *self {
            Position::Borrow {
                vault_index,
                shares,
                ..
            } => {
                let vault = &vaults[vault_index as usize];
                let oracle = vault.oracle()?;
                let amount = self.get_owed_single(&shares, vault)?;
                Ok(oracle.calculate_value(amount))
            }
            _ => unreachable!("should be called on liability, oopsie"),
        }
    }

    pub fn collateral_values(&self, vaults: &[Vault]) -> Result<CollateralValues, LibErrors> {
        match self {
            Position::LiquidityProvide {
                vault_index,
                strategy_index,
                shares,
                ..
            } => {
                let vault = &vaults[*vault_index as usize];
                let oracle = vault.oracle()?;
                let quote_oracle = vault.quote_oracle()?;

                let strategy = vault.strategies.get_strategy(*strategy_index)?;

                let (base_quantity, quote_quantity) =
                    self.get_owed_double(*strategy_index, shares, vault)?;

                let value = oracle.calculate_value(base_quantity)
                    + quote_oracle.calculate_value(quote_quantity);
                let with_collateral_ratio = value * strategy.collateral_ratio();
                let unhealthy = value * strategy.liquidation_threshold();

                Ok(CollateralValues {
                    exact: value,
                    with_collateral_ratio,
                    unhealthy,
                })
            }
            _ => unreachable!("should be called on collateral, oopsie"),
        }
    }

    pub fn loss_n_profit(&self, vaults: &[Vault]) -> Result<(Value, CollateralValues), LibErrors> {
        match *self {
            Position::Trading {
                vault_index,
                receipt,
                ..
            } => {
                let vault = &vaults[vault_index as usize];
                let trade = vault.services.trade()?;
                let oracle = vault.oracle()?;
                let quote_oracle = vault.quote_oracle()?;
                let profit_or_loss = trade.calculate_value(&receipt, oracle, quote_oracle);

                let res = match profit_or_loss {
                    TradeResult::None => unreachable!(),
                    TradeResult::Profitable(val) => (
                        Value::new(0),
                        CollateralValues {
                            exact: val,
                            with_collateral_ratio: val * trade.collateral_ratio(),
                            unhealthy: val * trade.liquidation_threshold(),
                        },
                    ),
                    TradeResult::Loss(val) => (val, CollateralValues::default()),
                };

                Ok(res)
            }
            _ => unreachable!("should be called on trade, oopsie"),
        }
    }
}

#[cfg(test)]
mod position_equality {
    use super::*;
    use checked_decimal_macro::Decimal;

    #[test]
    fn empties() {
        let first_empty = Position::Empty;
        let second_empty = Position::Empty;
        let borrow = Position::Borrow {
            vault_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };
        let provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        assert_eq!(first_empty, second_empty);
        assert_ne!(first_empty, borrow);
        assert_ne!(first_empty, provide);
        assert_ne!(borrow, provide);
    }

    #[test]
    fn specific_borrow() {
        let borrow = Position::Borrow {
            vault_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };

        let non_matching_borrow = Position::Borrow {
            vault_index: 1,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };

        let matching_borrow = Position::Borrow {
            vault_index: 0,
            shares: Shares::new(1),
            amount: Quantity::new(1),
        };

        assert_ne!(borrow, non_matching_borrow);
        assert_eq!(borrow, matching_borrow);
    }

    #[test]
    fn specific_provide() {
        let provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        let non_matching_provide = Position::LiquidityProvide {
            vault_index: 1,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        let matching_provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1),
            amount: Quantity::new(1),
            quote_amount: Quantity::new(0),
        };

        let reverse_non_matching_provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        assert_ne!(provide, non_matching_provide);
        assert_ne!(provide, reverse_non_matching_provide);
        assert_eq!(provide, matching_provide);
    }
}
