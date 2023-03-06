use super::*;
use crate::{
    services::lending::Borrowable,
    user::{Position, UserStatement},
};
use checked_decimal_macro::Decimal;

impl Vault {
    pub fn borrow(
        &mut self,
        user_statement: &mut UserStatement,
        amount: Quantity,
        current_time: Time,
    ) -> Result<(), ()> {
        self.refresh(current_time)?; // should be called in the outer function and after that user_statement.refresh

        let oracle = self.oracle.as_ref().unwrap();
        let lend = self.services.lend.as_mut().ok_or(()).unwrap();
        let total_available = lend.available().base;

        let user_allowed_borrow = user_statement.permitted_debt();

        let borrow_quantity =
            lend.calculate_borrow_quantity(oracle, amount, user_allowed_borrow)?;

        let shares = lend.borrow(borrow_quantity)?;
        self.lock_base(borrow_quantity, total_available, ServiceType::Lend)?;

        let position_temp = Position::Borrow {
            vault_index: self.id,
            shares,
            amount: borrow_quantity,
        };

        match user_statement.search_mut(&position_temp) {
            Some(position) => {
                position.increase_amount(borrow_quantity);
                position.increase_shares(shares);
            }
            None => {
                user_statement.add_position(position_temp)?;
            }
        }

        Ok(())
    }

    pub fn repay(
        &mut self,
        user_statement: &mut UserStatement,
        repay_quantity: Quantity,
        borrowed_quantity: Quantity,
        borrowed_shares: Shares,
        current_time: Time,
    ) -> Result<Shares, ()> {
        self.refresh(current_time)?;

        let lend = self.lend_service()?;
        let total_locked = lend.locked().base;

        let (unlock_quantity, burned_shares) =
            lend.repay(repay_quantity, borrowed_quantity, borrowed_shares)?;

        self.unlock_base(unlock_quantity, total_locked, ServiceType::Lend)?;

        let position_temp = Position::Borrow {
            vault_index: self.id,
            shares: Shares::new(0),
            amount: Quantity(0),
        };

        match user_statement.search_mut_id(&position_temp) {
            Some((id, position)) => {
                if burned_shares.ge(position.shares()) {
                    user_statement.delete_position(id);
                } else {
                    position.decrease_shares(burned_shares);
                    position.decrease_amount(unlock_quantity);
                }
            }
            None => panic!("fatal"),
        };

        Ok(burned_shares)
    }
}
