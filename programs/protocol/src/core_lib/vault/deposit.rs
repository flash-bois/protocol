use super::*;
use crate::core_lib::user::{Position, UserStatement};
use checked_decimal_macro::Decimal;
use std::cmp::min;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Token {
    Base,
    Quote,
}

impl Vault {
    pub fn other_quantity_in_ratio(
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

    pub fn withdraw(
        &mut self,
        user_statement: &mut UserStatement,
        withdraw_token: Token,
        amount: Quantity,
        strategy_index: u8,
    ) -> Result<(Quantity, Quantity), LibErrors> {
        let position_temp = Position::LiquidityProvide {
            vault_index: self.id,
            strategy_index,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        let (id, position) = user_statement.search_mut_id(&position_temp)?;

        let strategy = self.strategy(strategy_index)?;
        let balance = match withdraw_token {
            Token::Base => strategy.balance(),
            Token::Quote => strategy.balance_quote(),
        };
        let base_available = strategy.available();
        let quote_available = strategy.available_quote();

        let shares = min(
            strategy.total_shares().get_change_up(amount, balance),
            *position.shares(),
        );

        let (base_quantity, quote_quantity) = strategy.get_earned_double(&shares);

        if base_quantity > base_available {
            return Err(LibErrors::NotEnoughBaseQuantity);
        }
        if quote_quantity > quote_available {
            return Err(LibErrors::NotEnoughQuoteQuantity);
        }

        let strategy = self.strategies.get_strategy_mut(strategy_index)?;
        strategy.withdraw(base_quantity, quote_quantity, shares, &mut self.services);

        if shares.lt(position.shares()) {
            position.decrease_amount(min(*position.amount(), base_quantity));
            position.decrease_quote_amount(min(*position.quote_amount(), quote_quantity));
            position.increase_shares(shares);
        } else {
            user_statement.delete_position(id)
        }

        Ok((base_quantity, quote_quantity))
    }

    pub fn deposit(
        &mut self,
        user_statement: &mut UserStatement,
        deposit_token: Token,
        amount: Quantity,
        strategy_index: u8,
    ) -> Result<Quantity, LibErrors> {
        let base_oracle = self.oracle()?;
        let quote_oracle = self.quote_oracle()?;

        let strategy = self.strategy(strategy_index)?;
        // get other quantity in ratio based on base token and quote token balances and given input,
        // if neither of them are greaten than zero returns 0 as marker for further calculations
        let opposite_quantity = self.other_quantity_in_ratio(amount, deposit_token, strategy);

        // returns quantities, if previously mentioned zero amount case happened,
        // then calculate worth of known input and get others in 1:1 value to value ratio
        // from oracles
        let (base_quantity, quote_quantity, balance) = match deposit_token {
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

        let mut_strategy = self.strategies.get_strategy_mut(strategy_index)?;
        let shares = mut_strategy.total_shares().get_change_down(amount, balance);
        mut_strategy.deposit(base_quantity, quote_quantity, shares, &mut self.services);

        let temp_position = Position::LiquidityProvide {
            vault_index: self.id,
            strategy_index,
            shares,
            amount,
            quote_amount: quote_quantity,
        };

        match user_statement.search_mut(&temp_position) {
            Ok(position) => {
                position.increase_amount(amount);
                position.increase_quote_amount(quote_quantity);
                position.increase_shares(shares);
            }
            Err(..) => user_statement.add_position(temp_position)?,
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
    fn deposit_base_and_quote() {
        let mut vault = Vault::new_vault_for_tests().expect("couldn't create vault");
        let user_statement = &mut UserStatement::default();

        assert_eq!(
            vault
                .deposit(user_statement, Token::Base, Quantity::new(2000000), 0)
                .unwrap(),
            Quantity::new(4000000)
        );
        assert_eq!(
            vault
                .deposit(user_statement, Token::Quote, Quantity::new(4000000), 0)
                .unwrap(),
            Quantity::new(2000000)
        );
    }

    #[test]
    fn test_deposit() {
        let mut vault = Vault::new_vault_for_tests().expect("couldn't create vault");
        let user_statement = &mut UserStatement::default();

        vault
            .deposit(user_statement, Token::Base, Quantity::new(100), 0)
            .expect("deposit failed");
    }
}
