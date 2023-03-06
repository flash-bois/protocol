use super::*;
use crate::core_lib::{
    errors::LibErrors,
    services::lending::Borrowable,
    user::{Position, UserStatement},
};
use checked_decimal_macro::Decimal;

impl Vault {
    fn lend_and_oracle(&mut self) -> Result<(&mut Lend, &Oracle), LibErrors> {
        let Self {
            services: Services { lend, .. },
            oracle,
            ..
        } = self;

        let lend = lend.as_mut().ok_or(LibErrors::LendServiceNone)?;
        let oracle = oracle.as_mut().ok_or(LibErrors::OracleNone)?;

        Ok((lend, oracle))
    }

    pub fn borrow(
        &mut self,
        user_statement: &mut UserStatement,
        amount: Quantity,
    ) -> Result<Quantity, LibErrors> {
        let (lend, oracle) = self.lend_and_oracle()?;
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
            Ok(position) => {
                position.increase_amount(borrow_quantity);
                position.increase_shares(shares);
            }
            Err(..) => user_statement.add_position(position_temp)?,
        }

        Ok(borrow_quantity)
    }

    pub fn repay(
        &mut self,
        user_statement: &mut UserStatement,
        repay_quantity: Quantity,
    ) -> Result<Quantity, LibErrors> {
        let position_temp = Position::Borrow {
            vault_index: self.id,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };

        let (id, position) = user_statement.search_mut_id(&position_temp)?;

        let lend = self.lend_service()?;
        let total_locked = lend.locked().base;

        let (unlock_quantity, burned_shares) =
            lend.repay(repay_quantity, *position.amount(), *position.shares())?;

        self.unlock_base(unlock_quantity, total_locked, ServiceType::Lend)?;

        if burned_shares.ge(&position.shares()) {
            user_statement.delete_position(id)
        } else {
            position.decrease_amount(unlock_quantity);
            position.decrease_shares(burned_shares);
        }

        Ok(unlock_quantity)
    }
}
