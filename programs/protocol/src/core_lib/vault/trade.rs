use crate::core_lib::{
    decimal::{BalanceChange, Quantity, Time},
    errors::LibErrors,
    services::{ServiceType, ServiceUpdate},
    structs::{Receipt, Side},
    user::{Position, UserStatement},
};

use super::Vault;

impl Vault {
    pub fn open_position(
        &mut self,
        user_statement: &mut UserStatement,
        quantity: Quantity,
        side: Side,
    ) -> Result<(), LibErrors> {
        let collateral = user_statement.permitted_debt();

        let position_temp = Position::Trading {
            vault_index: self.id,
            receipt: Receipt::default(),
        };

        if user_statement.search_mut(&position_temp).is_ok() {
            return Err(LibErrors::PositionAlreadyExists);
        }

        let (trade, oracle, quote_oracle) = self.trade_mut_and_oracles()?;

        let receipt = match side {
            Side::Long => {
                let receipt = trade.open_long(quantity, collateral, oracle)?;
                let base_available = trade.available().base;
                self.lock_base(receipt.locked, base_available, ServiceType::Trade)?;
                receipt
            }
            Side::Short => {
                let receipt = trade.open_short(quantity, collateral, oracle, quote_oracle)?;

                let total_available = trade.available().quote;
                self.lock_quote(receipt.locked, total_available, ServiceType::Trade)?;

                receipt
            }
        };

        let position = Position::Trading {
            vault_index: self.id,
            receipt,
        };

        user_statement.add_position(position)?;

        Ok(())
    }

    pub fn close_position(
        &mut self,
        user_statement: &mut UserStatement,
        now: Time,
    ) -> Result<(BalanceChange, Side), LibErrors> {
        let temp_position = Position::Trading {
            vault_index: self.id,
            receipt: Receipt::default(),
        };

        let (trade, oracle, quote_oracle) = self.trade_mut_and_oracles()?;

        let (position_id, found_position) = user_statement.search_mut_id(&temp_position)?;
        let receipt = found_position.receipt();

        let change = match receipt.side {
            Side::Long => {
                let total_locked = trade.locked().base;
                let (change, unlock) = trade.close_long(receipt, oracle)?;
                match change {
                    BalanceChange::Profit(profit) => self.unlock_with_loss_base(
                        unlock,
                        profit,
                        total_locked,
                        ServiceType::Trade,
                    )?,
                    BalanceChange::Loss(loss) => {
                        self.unlock_with_profit_base(
                            unlock,
                            loss,
                            total_locked,
                            ServiceType::Trade,
                        )?;
                    }
                };

                (change, Side::Long)
            }
            Side::Short => {
                let total_locked = trade.locked().quote;
                let (change, unlock) = trade.close_short(&receipt, oracle, &quote_oracle)?;
                match change {
                    BalanceChange::Profit(profit) => self.unlock_with_loss_quote(
                        unlock,
                        profit,
                        total_locked,
                        ServiceType::Trade,
                    )?,
                    BalanceChange::Loss(loss) => {
                        self.unlock_with_profit_quote(
                            unlock,
                            loss,
                            total_locked,
                            ServiceType::Trade,
                        )?;
                    }
                }
                (change, Side::Short)
            }
        };

        user_statement.delete_position(position_id);

        Ok(change)
    }
}

#[cfg(test)]
mod vault_trade {
    use super::*;
    use crate::core_lib::{
        decimal::{DecimalPlaces, Fraction, Price, Utilization},
        structs::FeeCurve,
        Token,
    };
    use checked_decimal_macro::{Decimal, Factories};

    fn test_vault(user: &mut UserStatement) -> Result<Vault, LibErrors> {
        let mut vault = Vault::default();

        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(2),
            Price::from_scale(5, 3),
            Price::from_scale(2, 2),
            0,
            Token::Base,
            0,
        )?;

        vault.enable_oracle(
            DecimalPlaces::Six,
            Price::from_integer(1),
            Price::from_scale(1, 3),
            Price::from_scale(2, 2),
            0,
            Token::Quote,
            0,
        )?;

        vault.enable_lending(
            FeeCurve::default(),
            Utilization::from_integer(1),
            Quantity::new(u64::MAX),
            0,
            0,
        )?;

        vault.enable_swapping(
            FeeCurve::default(),
            FeeCurve::default(),
            Fraction::from_scale(1, 1),
        )?;

        vault.enable_trading(
            Fraction::new(100),
            Fraction::from_integer(3),
            Fraction::from_integer(1),
            Fraction::from_integer(1),
            0,
        )?;

        vault.add_strategy(
            true,
            true,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        vault.add_strategy(
            true,
            false,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        vault.add_strategy(
            false,
            true,
            true,
            Fraction::from_integer(1),
            Fraction::from_integer(1),
        )?;

        vault.deposit(user, Token::Base, Quantity::new(397512473195), 0)?;
        vault.deposit(user, Token::Base, Quantity::new(8432214580093), 1)?;
        vault.deposit(user, Token::Base, Quantity::new(6334216739056), 2)?;

        Ok(vault)
    }

    #[test]
    fn check_unlock_profit_long() -> Result<(), LibErrors> {
        let mut user = UserStatement::default();
        let mut vault = test_vault(&mut user)?;
        user.refresh(&mut [vault])?;

        let mut sum_before = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum_before += strategy.available();
        }

        vault.open_position(&mut user, Quantity::new(2000000), Side::Long)?;

        vault
            .oracle_mut()?
            .update(Price::new(2100000000), Price::new(2000000), 0)?;

        let (balance_change, side) = vault.close_position(&mut user, 0)?;

        let mut sum = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum += strategy.available();
        }

        assert_eq!(balance_change.quantity(), Quantity::new(95038));
        assert_eq!(sum, sum_before - balance_change.quantity());
        assert_eq!(side, Side::Long);

        Ok(())
    }

    #[test]
    fn check_unlock_loss_long() -> Result<(), LibErrors> {
        let mut user = UserStatement::default();
        let mut vault = test_vault(&mut user)?;
        user.refresh(&mut [vault])?;

        let mut sum_before = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum_before += strategy.available();
        }

        vault.open_position(&mut user, Quantity::new(2000000), Side::Long)?;

        vault
            .oracle_mut()?
            .update(Price::new(1900000000), Price::new(2000000), 0)?;

        let (balance_change, side) = vault.close_position(&mut user, 0)?;

        let mut sum = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum += strategy.available();
        }

        assert_eq!(balance_change.quantity(), Quantity::new(105464));
        assert_eq!(sum, sum_before + balance_change.quantity());
        assert_eq!(side, Side::Long);

        Ok(())
    }

    #[test]
    fn unlock_with_short_profit() -> Result<(), LibErrors> {
        let mut user = UserStatement::default();
        let mut vault = test_vault(&mut user)?;
        user.refresh(&mut [vault])?;

        let mut sum_before = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum_before += strategy.available_quote();
        }

        vault.open_position(&mut user, Quantity::new(2000000), Side::Short)?;

        vault
            .oracle_mut()?
            .update(Price::new(1900000000), Price::new(2000000), 0)?;

        let (balance_change, side) = vault.close_position(&mut user, 0)?;

        let mut sum = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum += strategy.available_quote();
        }

        assert_eq!(balance_change.quantity(), Quantity::new(199600));
        assert_eq!(sum, sum_before - balance_change.quantity());
        assert_eq!(side, Side::Short);

        Ok(())
    }

    #[test]
    fn unlock_with_short_loss() -> Result<(), LibErrors> {
        let mut user = UserStatement::default();
        let mut vault = test_vault(&mut user)?;
        user.refresh(&mut [vault])?;

        let mut sum_before = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum_before += strategy.available_quote();
        }

        vault.open_position(&mut user, Quantity::new(2000000), Side::Short)?;

        vault
            .oracle_mut()?
            .update(Price::new(2100000000), Price::new(2000000), 0)?;

        let (balance_change, side) = vault.close_position(&mut user, 0)?;

        let mut sum = Quantity::new(0);
        for strategy in vault.strategies.iter().unwrap() {
            sum += strategy.available_quote();
        }

        assert_eq!(balance_change.quantity(), Quantity::new(200400));
        assert_eq!(sum, sum_before + balance_change.quantity());
        assert_eq!(side, Side::Short);

        Ok(())
    }
}
