use crate::{
    decimal::{Quantity, Time},
    services::{ServiceType, ServiceUpdate},
    structs::Side,
    user::UserStatement,
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
        let (trade, oracle, quote_oracle) = self.trade_and_oracles()?;

        let _receipt = match side {
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

        // TODO: add receipt to user_statement

        Ok(())
    }
}
