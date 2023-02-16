use super::*;
use crate::user::{Position, UserStatement};
use checked_decimal_macro::Factories;

#[derive(Clone, Copy)]
pub enum Token {
    Base,
    Quote,
}

impl Vault {
    pub fn opposite_quantity(
        &self,
        input_quantity: Quantity,
        input_token: Token,
        strategy: &Strategy,
    ) -> Quantity {
        let base_balance = strategy.balance();
        let quote_balance = strategy.balance_quote();

        if base_balance == Quantity(0) || quote_balance == Quantity(0) {
            return Quantity(0);
        }

        match input_token {
            Token::Quote => input_quantity.big_mul_div(base_balance, quote_balance),
            Token::Base => input_quantity.big_mul_div(quote_balance, base_balance),
        }
    }

    pub fn strategy_mut(&mut self, index: u8) -> Result<&mut Strategy, ()> {
        self.strategies.get_mut_checked(index as usize).ok_or(())
    }

    pub fn strategy(&self, index: u8) -> Result<&Strategy, ()> {
        self.strategies.get_checked(index as usize).ok_or(())
    }

    fn get_opposite_quantity_with_pos_value(
        &self,
        opposite_oracle: &Oracle,
        opposite_quantity: Quantity,
        known_oracle: &Oracle,
        known_amount: Quantity,
        reverse: bool,
    ) -> (Quantity, Quantity, Value) {
        let known_value = known_oracle.calculate_value(known_amount);

        let (opposite_quantity, position_value) = if opposite_quantity == Quantity(0) {
            (
                opposite_oracle.calculate_needed_quantity(known_value),
                known_value * Value::from_integer(2),
            )
        } else {
            (
                opposite_quantity,
                opposite_oracle.calculate_value(opposite_quantity) + known_value,
            )
        };

        if !reverse {
            (known_amount, opposite_quantity, position_value)
        } else {
            (opposite_quantity, known_amount, position_value)
        }
    }

    pub fn deposit(
        &mut self,
        user_statement: &mut UserStatement,
        deposit_token: Token,
        amount: Quantity,
        strategy_index: u8,
        current_time: Time,
    ) -> Result<(), ()> {
        self.refresh(current_time)?;
        let base_oracle = self.oracle.as_ref().ok_or(())?;
        let quote_oracle = self.quote_oracle.as_ref().ok_or(())?;

        let strategy = self.strategy(strategy_index)?;

        let strategy_value = base_oracle.calculate_needed_value(strategy.balance())
            + quote_oracle.calculate_needed_value(strategy.balance_quote());

        let opposite_quantity = self.opposite_quantity(amount, deposit_token, strategy);

        let (base_quantity, quote_quantity, position_value) = match deposit_token {
            Token::Base => self.get_opposite_quantity_with_pos_value(
                quote_oracle,
                opposite_quantity,
                base_oracle,
                amount,
                false,
            ),

            Token::Quote => self.get_opposite_quantity_with_pos_value(
                base_oracle,
                opposite_quantity,
                quote_oracle,
                amount,
                true,
            ),
        };

        let mut_strategy = self
            .strategies
            .get_mut_checked(strategy_index as usize)
            .ok_or(())?;

        let shares = mut_strategy.deposit(
            position_value,
            strategy_value,
            base_quantity,
            quote_quantity,
            &mut self.services,
        )?;

        let temp_position = Position::LiquidityProvide {
            vault_index: self.id,
            strategy_index,
            shares,
            amount,
        };

        match user_statement.search_mut(&temp_position) {
            Some(position) => {
                position.increase_amount(amount);
                position.increase_shares(shares);
            }
            None => {
                user_statement.add_position(temp_position)?;
            }
        }

        // TODO add it to user struct

        Ok(())
    }
}
