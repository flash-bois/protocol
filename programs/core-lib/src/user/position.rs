use super::*;

#[repr(u8)]
pub enum Side {
    Long,
    Short,
}

#[derive(Default)]
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
}
