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
                let (change, unlock) = trade.close_long(*receipt, oracle)?;
                match change {
                    BalanceChange::Profit(profit) => self.unlock_with_loss_base(
                        unlock,
                        profit,
                        total_locked,
                        ServiceType::Trade,
                    )?,
                    BalanceChange::Loss(loss) => {
                        self.unlock_base(unlock, total_locked, ServiceType::Trade)?;
                        self.profit_base(loss, total_locked, ServiceType::Trade)?;
                    }
                };

                (change, Side::Long)
            }
            Side::Short => {
                let total_locked = trade.locked().quote;
                let (change, unlock) = trade.close_short(*receipt, oracle, &quote_oracle)?;
                match change {
                    BalanceChange::Profit(profit) => self.unlock_with_loss_quote(
                        unlock,
                        profit,
                        total_locked,
                        ServiceType::Trade,
                    )?,
                    BalanceChange::Loss(loss) => {
                        self.unlock_quote(unlock, total_locked, ServiceType::Trade)?;
                        // TODO: do something with system profit
                        self.profit_quote(loss, total_locked, ServiceType::Trade)?
                    }
                }
                (change, Side::Short)
            }
        };

        user_statement.delete_position(position_id);

        Ok(change)
    }
}
