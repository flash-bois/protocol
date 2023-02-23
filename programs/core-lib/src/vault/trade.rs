use crate::{
    decimal::{BalanceChange, Quantity, Time},
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
        now: Time,
    ) -> Result<(), ()> {
        self.refresh(now)?;

        let collateral = user_statement.permitted_debt();
        let (trade, oracle, quote_oracle) = self.trade_mut_and_oracles()?;

        let receipt = match side {
            Side::Long => {
                let receipt = trade.open_long(quantity, collateral, oracle, now)?;
                let base_available = trade.available().base;
                self.lock_base(receipt.locked, base_available, ServiceType::Trade)?;
                receipt
            }
            Side::Short => {
                let receipt = trade.open_short(quantity, collateral, oracle, quote_oracle, now)?;

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
        side: Side,
        now: Time,
    ) -> Result<BalanceChange, ()> {
        let temp_position = Position::Trading {
            vault_index: self.id,
            receipt: Receipt {
                side,
                ..Default::default()
            },
        };

        let (trade, oracle, quote_oracle) = self.trade_mut_and_oracles()?;

        let (position_id, found_position) =
            user_statement.search_mut_id(&temp_position).ok_or(())?;
        let receipt = found_position.receipt();

        let change = match receipt.side {
            Side::Long => {
                let (change, unlock) = trade.close_long(*receipt, oracle)?;
                match change {
                    BalanceChange::Profit(profit) => {
                        let total_locked = trade.locked().base;
                        self.unlock_with_loss_base(
                            unlock,
                            profit,
                            total_locked,
                            ServiceType::Trade,
                        )?
                    }
                    BalanceChange::Loss(loss) => {
                        let total_locked = trade.locked().base;
                        self.unlock_base(unlock, total_locked, ServiceType::Trade)?;
                        // TODO: do something with system profit
                        self.profit_base(loss, total_locked, ServiceType::Trade)?;
                    }
                };
                change
            }
            Side::Short => {
                let (change, unlock) = trade.close_short(*receipt, oracle, &quote_oracle, now)?;
                let total_locked = trade.locked().quote;
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
                change
            }
        };

        user_statement.delete_position(position_id);

        Ok(change)
    }
}
