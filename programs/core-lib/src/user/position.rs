use super::*;

#[derive(Debug)]
#[repr(u8)]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Default)]
pub enum Position {
    #[default]
    Empty,
    LiquidityProvide {
        vault_index: u8,
        strategy_index: u8,
        shares: Shares,
        amount: Quantity,
    },
    Borrow {
        vault_index: u8,
        shares: Shares,
        amount: Quantity,
    },
    Trading {
        vault_index: u8,
        side: Side,
        quantity: Quantity,
        quote_quantity: Option<Quantity>,
        open_price: Price,
        entry_funding: FundingRate,
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
            (Self::Empty, Self::Empty) => true,
            _ => false,
        }
    }
}

impl Position {
    pub fn is_liability(self) -> bool {
        match self {
            Position::Borrow { .. } => true,
            Position::LiquidityProvide { .. } => false,
            Position::Trading { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    pub fn is_collateral(self) -> bool {
        match self {
            Position::LiquidityProvide { .. } => true,
            Position::Borrow { .. } => false,
            Position::Trading { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    pub fn is_trade(self) -> bool {
        match self {
            Position::Trading { .. } => true,
            Position::Borrow { .. } => false,
            Position::LiquidityProvide { .. } => false,
            Position::Empty => unreachable!(),
        }
    }

    pub fn shares(&mut self) -> &mut Shares {
        match self {
            Position::Borrow { shares, .. } => shares,
            Position::LiquidityProvide { shares, .. } => shares,
            Position::Trading { .. } => panic!("trading does not have shares"),
            Position::Empty => unreachable!(),
        }
    }

    pub fn amount(&mut self) -> &mut Quantity {
        match self {
            Position::Borrow { amount, .. } => amount,
            Position::LiquidityProvide { amount, .. } => amount,
            Position::Trading { quantity, .. } => quantity,
            Position::Empty => unreachable!(),
        }
    }

    pub fn increase_amount(&mut self, amount: Quantity) {
        *self.amount() += amount
    }

    pub fn increase_shares(&mut self, shares: Shares) {
        *self.shares() += shares
    }

    pub fn decrease_amount(&mut self, amount: Quantity) {
        *self.amount() -= amount
    }

    pub fn decrease_shares(&mut self, shares: Shares) {
        *self.shares() -= shares
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
            amount: Quantity(0),
        };
        let provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity(0),
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
            amount: Quantity(0),
        };

        let non_matching_borrow = Position::Borrow {
            vault_index: 1,
            shares: Shares::new(0),
            amount: Quantity(0),
        };

        let matching_borrow = Position::Borrow {
            vault_index: 0,
            shares: Shares::new(1),
            amount: Quantity(1),
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
            amount: Quantity(0),
        };

        let non_matching_provide = Position::LiquidityProvide {
            vault_index: 1,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity(0),
        };

        let matching_provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 1,
            shares: Shares::new(1),
            amount: Quantity(1),
        };

        let reverse_non_matching_provide = Position::LiquidityProvide {
            vault_index: 0,
            strategy_index: 0,
            shares: Shares::new(0),
            amount: Quantity(0),
        };

        assert_ne!(provide, non_matching_provide);
        assert_ne!(provide, reverse_non_matching_provide);
        assert_eq!(provide, matching_provide);
    }
}
