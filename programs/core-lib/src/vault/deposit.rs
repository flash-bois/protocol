use super::*;
use crate::structs::OraclePriceType;

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

        match input_token {
            Token::Quote => input_quantity.big_mul_div(base_balance, quote_balance),
            Token::Base => input_quantity.big_mul_div(quote_balance, base_balance),
        }
    }

    fn strategy_mut(&mut self, index: u8) -> Result<&mut Strategy, ()> {
        self.strategies.get_mut_checked(index as usize).ok_or(())
    }

    fn strategy(&self, index: u8) -> Result<&Strategy, ()> {
        self.strategies.get_checked(index as usize).ok_or(())
    }

    pub fn deposit(
        &mut self,
        //user_statement: &mut UserStatement,
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

        let (base_quantity, quote_quantity, input_value) = match deposit_token {
            Token::Base => (
                amount,
                opposite_quantity,
                base_oracle.calculate_value(amount),
            ),
            Token::Quote => (
                opposite_quantity,
                amount,
                quote_oracle.calculate_value(amount),
            ),
        };

        let mut_strategy = self
            .strategies
            .get_mut_checked(strategy_index as usize)
            .ok_or(())?;

        let shares = mut_strategy.deposit(
            input_value,
            strategy_value,
            base_quantity,
            quote_quantity,
            &mut self.services,
        )?;

        // TODO add it to user struct

        Ok(())
    }
}
