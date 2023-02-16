use super::*;
use crate::services::lending::Borrowable;
use checked_decimal_macro::Decimal;

impl Vault {
    pub fn borrow(
        &mut self,
        //user_statement: &mut UserStatement,
        amount: Quantity,
        current_time: Time,
    ) -> Result<(), ()> {
        self.refresh(current_time)?; // should be called in the outer function and after that user_statement.refresh

        let oracle = self.oracle.as_ref().unwrap();
        let lend = self.services.lend.as_mut().ok_or(()).unwrap();
        let total_available = lend.available();

        //let user_allowed_borrow = user_statement.allowed_borrow();

        let user_allowed_borrow = Value::new(u128::MAX);

        let borrow_quantity_with_fee =
            lend.calculate_borrow_quantity(oracle, amount, user_allowed_borrow)?;

        let _shares = lend.borrow(borrow_quantity_with_fee)?;
        self.lock(
            borrow_quantity_with_fee,
            total_available.base,
            ServiceType::Lend,
        )?;

        //todo add to UserStatement

        Ok(())
    }

    pub fn repay(
        &mut self,
        // user_statement: &mut UserStatement,
        repay_quantity: Quantity,
        borrowed_quantity: Quantity,
        borrowed_shares: Shares,
        current_time: Time,
    ) -> Result<Shares, ()> {
        self.refresh(current_time)?;

        let lend = self.lend_service()?;
        let total_locked = lend.locked();

        let (unlock_quantity, burned_shares) =
            lend.repay(repay_quantity, borrowed_quantity, borrowed_shares)?;

        //add update position

        self.unlock(unlock_quantity, total_locked.base, ServiceType::Lend)?;

        Ok(burned_shares)
    }
}
