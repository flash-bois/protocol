use checked_decimal_macro::{BetweenDecimals, BigOps, Factories};

use crate::{
    decimal::{Fraction, Precise, Quantity, Shares, Time, Utilization, Value},
    structs::FeeCurve,
    structs::Oracle,
};

use super::ServiceUpdate;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Lend {
    /// liquidity available to borrow by borrower, it's the sum of all strategies containing this service
    /// it should not be modified inside service
    available: Quantity,
    /// liquidity already borrowed
    /// containing accrued fee
    borrowed: Quantity,
    /// fee curve
    fee: FeeCurve,
    /// unix timestamp of last interest rate accrued
    last_fee_paid: Time,
    /// initial fee time for borrow
    initial_fee_time: Time,
    /// current utilization  (borrowed / balance (available + borrowed))
    utilization: Utilization,
    /// max utilization
    max_utilization: Utilization,
    /// borrow shares
    borrow_shares: Shares,
    /// ratio of borrow/collateral at which statement can be liquidated
    borrow_limit: Quantity,
    /// fee that had been accrued, but not yet distributed
    unclaimed_fee: Quantity,
    /// sum of all fees accrued (for statistics)
    total_fee: Quantity,
}

impl ServiceUpdate for Lend {
    fn add_available(&mut self, quantity: Quantity) {
        self.available += quantity;
        self.utilization = self.current_utilization();
    }

    fn remove_available(&mut self, quantity: Quantity) {
        self.available -= quantity;
        self.utilization = self.current_utilization();
    }

    fn available(&self) -> Quantity {
        self.available
    }

    fn locked(&self) -> Quantity {
        self.borrowed
    }

    fn accrue_fee(&mut self, oracle: Option<&Oracle>) -> Quantity {
        let accrued_fee = self.unclaimed_fee;
        self.unclaimed_fee = Quantity(0);
        self.borrowed += accrued_fee;
        accrued_fee
    }
}

impl Lend {
    pub fn new(
        fee: FeeCurve,
        max_utilization: Utilization,
        borrow_limit: Quantity,
        initial_fee_time: Time,
    ) -> Self {
        Lend {
            max_utilization,
            fee,
            borrow_limit,
            initial_fee_time,
            ..Default::default()
        }
    }

    fn calculate_borrow_fee(&self, borrow_amount: Quantity) -> Quantity {
        let future_utilization = Fraction::get_utilization(
            self.borrowed + borrow_amount,
            self.available + self.borrowed,
        );

        borrow_amount.big_mul_up(self.calculate_fee(
            self.last_fee_paid + self.initial_fee_time,
            Fraction::from_decimal(future_utilization),
        ))
    }

    /// Performs repay operation on Service
    ///
    /// ## Arguments
    ///
    /// * `oracle` - oracle reference to calculate value of borrow including initial fees
    /// * `user_desired_borrow` - quantity that user wants to borrow
    /// * `user_max_borrow:` - max quantity that user can borrow
    ///
    /// # Returns
    /// ## BorrowCalculations
    ///
    /// * `borrow` - quantity that have to be borrowed
    /// * `payout` - quantity repaid do user
    ///
    pub fn calculate_borrow_quantity(
        &self,
        oracle: &Oracle,
        user_desired_borrow: Quantity,
        user_allowed_borrow: Value,
    ) -> Result<Quantity, ()> {
        let borrow_fee_quantity = self.calculate_borrow_fee(user_desired_borrow);
        let borrow_quantity = user_desired_borrow + borrow_fee_quantity;
        let borrow_value = oracle.calculate_value(borrow_quantity);

        if borrow_value > user_allowed_borrow {
            return Err(());
        }

        Ok(borrow_quantity)
    }

    pub fn borrow_shares(&self) -> Shares {
        self.borrow_shares
    }

    pub fn borrow_limit(&self) -> Quantity {
        self.borrow_limit
    }

    pub fn allowed_borrow(&self) -> Quantity {
        self.borrow_limit - self.borrowed
    }

    /// calculates utilization - borrowed / (borrowed + available)
    pub fn current_utilization(&self) -> Utilization {
        Utilization::get_utilization(self.borrowed, self.balance())
    }

    /// checks if user can borrow - [(borrowed + borrow_request_amount) / (borrowed + available)] <= max_utilization
    pub fn can_borrow(&self, amount: Quantity) -> bool {
        Utilization::get_utilization(self.borrowed + amount, self.balance()) <= self.max_utilization
            && amount + self.borrowed < self.borrow_limit
    }

    /// Returns balance of lending, which is the sum of available and borrowed
    fn balance(&self) -> Quantity {
        self.available + self.borrowed
    }

    /// returns quantity of accumulated fees, not yet moved to strategy
    #[cfg(test)]
    pub fn unclaimed_fee(&self) -> Quantity {
        self.unclaimed_fee
    }

    // returns sum of all accrued fees
    #[cfg(test)]
    pub fn total_fees(&self) -> Quantity {
        self.total_fee
    }

    // return borrowed amount
    #[cfg(test)]
    pub fn borrowed(&self) -> Quantity {
        self.borrowed
    }

    // return utilization
    #[cfg(test)]
    pub fn utilization(&self) -> Utilization {
        self.utilization
    }

    /// Returns lending fee
    ///
    /// ## Arguments
    ///
    /// * `current_time` - current unix timestamp
    fn calculate_fee(&self, current_time: Time, utilization: Fraction) -> Precise {
        if current_time > self.last_fee_paid {
            let time_period = current_time - self.last_fee_paid;

            self.fee.compounded_fee(utilization, time_period)
        } else {
            Precise::from_integer(0)
        }
    }

    /// Updates unclaimed_fee, total_fee and last_fee_paid
    /// also method checks if current_time >= last_fee_paid
    ///
    /// ## Arguments
    ///
    /// * `current_time` - current unix timestamp
    pub fn accrue_interest_rate(&mut self, current_time: Time) {
        let fee_whole = self
            .borrowed
            .big_mul_up(self.calculate_fee(current_time, Fraction::from_decimal(self.utilization)));
        self.last_fee_paid = current_time;
        self.unclaimed_fee += fee_whole;
        self.total_fee += fee_whole;
    }
}

pub trait Borrowable {
    fn borrow(&mut self, quantity: Quantity) -> Result<Shares, ()>;
    fn repay(
        &mut self,
        repay_quantity: Quantity,
        borrowed: Quantity,
        borrowed_shares: Shares,
    ) -> Result<(Quantity, Shares), ()>;
}

impl Borrowable for Lend {
    /// Performs repay operation on Service
    ///
    /// ## Arguments
    ///
    /// * `repay_quantity` - quantity which user wants to repay
    /// * `borrowed` - initial user borrowed quantity (with no fee)
    /// * `borrowed_shares` - initial user borrowed shares
    ///
    /// # Returns
    /// ## (`quantity`, `shares`)
    ///
    /// * `quantity` is amount to be unlocked in strategy, includes all the fees accrued
    ///
    /// * `shares` is amount of borrow shares repaid
    ///
    fn repay(
        &mut self,
        repay_quantity: Quantity,
        borrowed: Quantity,
        borrowed_shares: Shares,
    ) -> Result<(Quantity, Shares), ()> {
        let owed_quantity = self
            .borrow_shares
            .calculate_owed(borrowed_shares, self.borrowed);

        let fee_owed = owed_quantity - borrowed;

        if repay_quantity > fee_owed {
            let shares_to_burn = self
                .borrow_shares
                .get_change_down(repay_quantity, self.borrowed);

            self.borrowed -= repay_quantity;
            self.borrow_shares -= shares_to_burn;
            self.utilization = self.current_utilization();

            Ok((repay_quantity, shares_to_burn))
        } else {
            Err(())
        }
    }

    /// Performs borrow operation on Service
    ///
    /// ## Arguments
    ///
    /// * `quantity` - quantity which user wants to borrow
    ///
    /// # Returns
    /// ## Shares
    ///
    /// * `Shares` is amount of shares user is in debt to the system
    ///
    fn borrow(&mut self, quantity: Quantity) -> Result<Shares, ()> {
        if !self.can_borrow(quantity) {
            return Err(());
        }

        let additional_shares = self.borrow_shares.get_change_up(quantity, self.borrowed);

        self.borrowed += quantity;
        self.borrow_shares += additional_shares;
        self.utilization = self.current_utilization();

        Ok(additional_shares)
    }
}
