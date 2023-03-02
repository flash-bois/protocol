use super::*;
use crate::core_lib::user::{Position, UserStatement};
use checked_decimal_macro::Decimal;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

        if base_balance == Quantity::new(0) || quote_balance == Quantity::new(0) {
            return Quantity::new(0);
        }

        match input_token {
            Token::Quote => input_quantity.big_mul_div(base_balance, quote_balance),
            Token::Base => input_quantity.big_mul_div(quote_balance, base_balance),
        }
    }

    pub fn strategy_mut(&mut self, index: u8) -> Result<&mut Strategy, LibErrors> {
        self.strategies
            .get_mut_checked(index as usize)
            .ok_or(LibErrors::StrategyMissing)
    }

    pub fn strategy(&self, index: u8) -> Result<&Strategy, LibErrors> {
        self.strategies
            .get_checked(index as usize)
            .ok_or(LibErrors::StrategyMissing)
    }

    pub fn get_opposite_quantity(
        &self,
        opposite_oracle: &Oracle,
        opposite_quantity: Quantity,
        known_oracle: &Oracle,
        known_amount: Quantity,
    ) -> (Quantity, Quantity) {
        let known_value = known_oracle.calculate_value(known_amount);

        let opposite_quantity = if opposite_quantity == Quantity::new(0) {
            opposite_oracle.calculate_needed_quantity(known_value)
        } else {
            opposite_quantity
        };

        (known_amount, opposite_quantity)
    }

    pub fn deposit(
        &mut self,
        user_statement: &mut UserStatement,
        deposit_token: Token,
        amount: Quantity,
        strategy_index: u8,
        current_time: Time,
    ) -> Result<Quantity, LibErrors> {
        self.refresh(current_time)?;
        let base_oracle = self.oracle()?;
        let quote_oracle = self.quote_oracle()?;

        let strategy = self.strategy(strategy_index)?;
        let opposite_quantity = self.opposite_quantity(amount, deposit_token, strategy);

        let (base_quantity, quote_quantity, input_balance) = match deposit_token {
            Token::Base => {
                let (base, quote) = self.get_opposite_quantity(
                    quote_oracle,
                    opposite_quantity,
                    base_oracle,
                    amount,
                );

                (base, quote, strategy.balance())
            }
            Token::Quote => {
                let (quote, base) = self.get_opposite_quantity(
                    base_oracle,
                    opposite_quantity,
                    quote_oracle,
                    amount,
                );

                (base, quote, strategy.balance_quote())
            }
        };

        let mut_strategy = self
            .strategies
            .get_mut_checked(strategy_index as usize)
            .ok_or(LibErrors::StrategyMissing)?;

        let shares = mut_strategy.deposit(
            base_quantity,
            quote_quantity,
            amount,
            input_balance,
            &mut self.services,
        )?;

        let temp_position = Position::LiquidityProvide {
            vault_index: self.id,
            strategy_index,
            shares,
            amount,
            quote_amount: quote_quantity,
        };

        match user_statement.search_mut(&temp_position) {
            Some(position) => {
                position.increase_amount(amount);
                position.increase_shares(shares);
            }
            None => {
                user_statement
                    .add_position(temp_position)
                    .map_err(|_| LibErrors::CannotAddPosition)?;
            }
        }

        if deposit_token == Token::Base {
            Ok(quote_quantity)
        } else {
            Ok(base_quantity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_lib::Vault;

    #[test]
    fn test_deposit() {
        let mut vault = Vault::new_vault_for_tests().expect("couldn't create vault");
        let user_statement = &mut UserStatement::default();

        vault
            .deposit(user_statement, Token::Base, Quantity::new(100), 0, 1000)
            .expect("deposit failed");
    }
}
