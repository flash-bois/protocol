use checked_decimal_macro::{BetweenDecimals, BigOps, Decimal, Factories};

use crate::core_lib::{
    decimal::{Balances, Fraction, Precise, Quantity, Shares, Time, Utilization, Value},
    errors::LibErrors,
    structs::FeeCurve,
    structs::Oracle,
};

use std::cmp::min;

use super::ServiceUpdate;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct Lend {
        /// liquidity available to borrow by borrower, it's the sum of all strategies containing this service
        /// it should not be modified inside service
        pub available: Quantity,
        /// liquidity already borrowed
        /// containing accrued fee
        pub borrowed: Quantity,
        /// fee curve
        pub fee: FeeCurve,
        /// unix timestamp of last interest rate accrued
        pub last_fee_paid: u32,
        /// initial fee time for borrow
        pub initial_fee_time: u32,
        /// current utilization  (borrowed / balance (available + borrowed))
        pub utilization: Utilization,
        /// max utilization
        pub max_utilization: Utilization,
        /// borrow shares
        pub borrow_shares: Shares,
        /// ratio of borrow/collateral at which statement can be liquidated
        pub borrow_limit: Quantity,
        /// fee that had been accrued, but not yet distributed
        pub unclaimed_fee: Quantity,
        /// sum of all fees accrued (for statistics)
        pub total_fee: Quantity,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    #[repr(C)]
    pub struct Lend {
        /// liquidity available to borrow by borrower, it's the sum of all strategies containing this service
        /// it should not be modified inside service
        pub available: Quantity,
        /// liquidity already borrowed
        /// containing accrued fee
        pub borrowed: Quantity,
        /// fee curve
        pub fee: FeeCurve,
        /// unix timestamp of last interest rate accrued
        pub last_fee_paid: u32,
        /// initial fee time for borrow
        pub initial_fee_time: u32,
        /// current utilization  (borrowed / balance (available + borrowed))
        pub utilization: Utilization,
        /// max utilization
        pub max_utilization: Utilization,
        /// borrow shares
        pub borrow_shares: Shares,
        /// ratio of borrow/collateral at which statement can be liquidated
        pub borrow_limit: Quantity,
        /// fee that had been accrued, but not yet distributed
        pub unclaimed_fee: Quantity,
        /// sum of all fees accrued (for statistics)
        pub total_fee: Quantity,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Lend;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Lend;

impl ServiceUpdate for Lend {
    fn add_liquidity_base(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn add_liquidity_quote(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn remove_liquidity_base(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn remove_liquidity_quote(&mut self, _: Quantity) {
        unreachable!("Lending does not need data about liquidity")
    }
    fn add_available_quote(&mut self, _: Quantity) {
        unreachable!("Lending of quote tokens is separate")
    }
    fn remove_available_quote(&mut self, _: Quantity) {
        unreachable!()
    }

    fn add_available_base(&mut self, quantity: Quantity) {
        self.available += quantity;
        self.utilization = self.current_utilization();
    }

    fn remove_available_base(&mut self, quantity: Quantity) {
        self.available -= quantity;
        self.utilization = self.current_utilization();
    }

    fn available(&self) -> Balances {
        Balances {
            base: self.available,
            quote: Quantity::new(0),
        }
    }

    fn locked(&self) -> Balances {
        Balances {
            base: self.borrowed,
            quote: Quantity::new(0),
        }
    }

    fn accrue_fee(&mut self) -> Balances {
        let accrued_fee = self.unclaimed_fee;
        self.unclaimed_fee = Quantity::new(0);
        self.borrowed += accrued_fee;

        Balances {
            base: accrued_fee,
            quote: Quantity::new(0),
        }
    }
}

impl Lend {
    pub fn new(
        fee: FeeCurve,
        max_utilization: Utilization,
        borrow_limit: Quantity,
        initial_fee_time: Time,
        last_fee_paid: Time,
    ) -> Self {
        Lend {
            max_utilization,
            fee,
            borrow_limit,
            initial_fee_time,
            last_fee_paid,
            ..Default::default()
        }
    }

    fn calculate_borrow_fee(&self, borrow_amount: Quantity) -> Quantity {
        let future_utilization = Fraction::get_utilization(
            self.borrowed + borrow_amount,
            self.available + self.borrowed,
        );

        let fee = self.calculate_fee(
            self.initial_fee_time,
            Fraction::from_decimal(future_utilization),
        );

        borrow_amount.big_mul_up(fee)
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
    //
    pub fn calculate_borrow_quantity(
        &self,
        oracle: &Oracle,
        user_desired_borrow: Quantity,
        user_allowed_borrow: Value,
    ) -> Result<Quantity, LibErrors> {
        let borrow_fee_quantity = self.calculate_borrow_fee(user_desired_borrow);
        let borrow_quantity = user_desired_borrow + borrow_fee_quantity;
        let borrow_value = oracle.calculate_value(borrow_quantity);

        if borrow_value > user_allowed_borrow {
            return Err(LibErrors::UserAllowedBorrowExceeded);
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

    pub fn get_apy(&self, time: Time) -> Fraction {
        if self.available == Quantity::new(0) || self.locked().base == Quantity::new(0) {
            return Fraction::new(0);
        }

        Fraction::from_decimal(
            self.fee
                .compounded_apy(Fraction::from_decimal(self.utilization), time)
                * self.locked().base
                / self.available().base,
        )
    }

    pub fn current_fee(&self) -> Result<Fraction, LibErrors> {
        self.fee
            .get_point_fee(Fraction::from_decimal(self.utilization))
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
        if self.borrowed > Quantity::new(0) {
            let fee_whole = self.borrowed.big_mul_up(
                self.calculate_fee(current_time, Fraction::from_decimal(self.utilization)),
            );
            self.unclaimed_fee += fee_whole;
            self.total_fee += fee_whole;
        }

        self.last_fee_paid = current_time;
    }

    pub fn fee_curve(&mut self) -> &mut FeeCurve {
        &mut self.fee
    }
}

pub trait Borrowable {
    fn borrow(&mut self, quantity: Quantity) -> Result<Shares, LibErrors>;
    fn repay(
        &mut self,
        repay_quantity: Quantity,
        borrowed: Quantity,
        borrowed_shares: Shares,
    ) -> Result<(Quantity, Shares, Quantity), LibErrors>;
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
    ) -> Result<(Quantity, Shares, Quantity), LibErrors> {
        let owed_quantity = self
            .borrow_shares
            .calculate_owed(borrowed_shares, self.borrowed);

        let fee_owed = owed_quantity - borrowed;
        let repay_amount = min(repay_quantity, owed_quantity);

        if repay_quantity >= fee_owed {
            let shares_to_burn = self
                .borrow_shares
                .get_change_down(repay_amount, self.borrowed);

            self.borrowed -= repay_quantity;
            self.borrow_shares -= shares_to_burn;
            self.utilization = self.current_utilization();

            Ok((
                repay_quantity,
                shares_to_burn,
                min(repay_quantity, borrowed),
            ))
        } else {
            Err(LibErrors::RepayLowerThanFee)
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
    fn borrow(&mut self, quantity: Quantity) -> Result<Shares, LibErrors> {
        if !self.can_borrow(quantity) {
            return Err(LibErrors::CannotBorrow);
        }

        let additional_shares = self.borrow_shares.get_change_up(quantity, self.borrowed);

        self.borrowed += quantity;
        self.borrow_shares += additional_shares;
        self.utilization = self.current_utilization();

        Ok(additional_shares)
    }
}

#[cfg(test)]
mod lend_tests {
    use super::*;
    use crate::core_lib::decimal::Price;
    use crate::core_lib::structs::fee_curve::HOUR_DURATION;
    use checked_decimal_macro::Decimal;

    #[test]
    fn borrow_fee() -> Result<(), LibErrors> {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::new(10), Fraction::new(1));
        let lend = Lend::new(
            fee,
            Utilization::from_integer(1),
            Quantity::new(10000000000),
            60 * 60,
            0,
        );

        let oracle = Oracle::new(
            crate::core_lib::decimal::DecimalPlaces::Six,
            Price::from_integer(2),
            Price::new(200000),
            Price::from_scale(2, 2),
            0,
            0,
        );
        let borrow_amount = Quantity::new(200_000_000);

        let borrow_fee = lend.calculate_borrow_fee(borrow_amount);
        assert_eq!(borrow_fee, Quantity::new(2001));

        let borrow =
            lend.calculate_borrow_quantity(&oracle, borrow_amount, Value::from_integer(420))?;

        assert_eq!(borrow, Quantity::new(200_002_001));

        Ok(())
    }

    #[test]
    fn borrows_max_repay_max() -> Result<(), LibErrors> {
        let current_time = 100000;
        let max_utilization = Utilization::from_scale(80, 2);

        let mut lending = Lend::new(
            FeeCurve::default(),
            max_utilization,
            Quantity::new(u64::MAX),
            0,
            0,
        );

        lending.add_available_base(Quantity::new(2_000_000));
        lending.accrue_interest_rate(current_time);

        assert_eq!(
            lending,
            Lend {
                available: Quantity::new(2_000_000),
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        assert!(
            lending.borrow(Quantity::new(1_600_001)).is_err(),
            "can't borrow due to too high utilization"
        );

        assert!(
            lending.borrow(Quantity::new(1_600_000)).is_ok(),
            "can borrow"
        );
        lending.remove_available_base(Quantity::new(1_600_000));

        assert_eq!(
            lending,
            Lend {
                available: Quantity::new(400_000),
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                utilization: max_utilization,
                borrow_shares: Shares::from_integer(1_600_000),
                borrowed: Quantity::new(1_600_000),
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        assert!(lending.borrow(Quantity::new(1)).is_err(), "can't borrow");

        let (partially_repaid, shares_partially_repaid, _) = lending.repay(
            Quantity::new(1_530_264),
            Quantity::new(1_600_000),
            Shares::from_integer(1_600_000),
        )?;

        lending.add_available_base(partially_repaid);

        let (full_repaid, _shares_fully_repaid, _) = lending.repay(
            Quantity::new(1_600_000) - partially_repaid,
            Quantity::new(1_600_000) - partially_repaid,
            Shares::from_integer(1_600_000) - shares_partially_repaid,
        )?;

        lending.add_available_base(full_repaid);

        assert_eq!(
            lending,
            Lend {
                available: Quantity::new(2_000_000),
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        Ok(())
    }

    #[test]
    fn fee_accruing() -> Result<(), LibErrors> {
        let mut current_time = 0;

        let max_utilization = Utilization::from_scale(80, 2); // 0,8 === 80 %

        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::new(100), Fraction::from_scale(40, 2)); // 0.01% 1 basis point
        fee.add_constant_fee(Fraction::new(1000), Fraction::from_scale(60, 2)); // 0.1% 10 basis point
        fee.add_constant_fee(Fraction::new(10000), Fraction::from_scale(80, 2)); // 1% 100 basis point
        fee.add_constant_fee(Fraction::new(20000), Fraction::from_scale(100, 2)); // 2% 200 basis point

        let mut lending = Lend::new(fee, max_utilization, Quantity::new(u64::MAX), 0, 0);

        lending.add_available_base(Quantity::new(736796576003955192));

        current_time += 100 * HOUR_DURATION;
        lending.accrue_interest_rate(current_time);
        lending.accrue_fee();

        lending.add_available_base(Quantity::new(536908355173637734));

        // available, shares = 736796576003955192 + 536908355173637734 = 1273704931177592926
        assert_eq!(
            lending,
            Lend {
                available: Quantity::new(1273704931177592926),
                borrow_limit: Quantity::new(u64::MAX),
                max_utilization,
                fee,
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        current_time += 100 * HOUR_DURATION;
        lending.accrue_interest_rate(current_time);
        lending.accrue_fee();

        lending.borrow(Quantity::new(184186871548154787))?;
        lending.remove_available_base(Quantity::new(184186871548154787));

        assert_eq!(
            lending,
            Lend {
                // available = 1273704931177592926 - 184186871548154787 = 1089518059629438139
                available: Quantity::new(1089518059629438139),
                borrowed: Quantity::new(184186871548154787),
                borrow_shares: Shares::new(184186871548154787),
                borrow_limit: Quantity::new(u64::MAX),
                max_utilization,
                fee,
                // utilization = Divide[184186871548154787,184186871548154787 + 1089518059629438139] = 0.14460717473
                utilization: Utilization::from_scale(144608, 6),
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        current_time += 50 * HOUR_DURATION;
        lending.accrue_interest_rate(current_time);
        let fee_q = lending.accrue_fee();
        assert_eq!(
            fee_q,
            Balances {
                base: Quantity::new(923240522808082),
                quote: Quantity::new(0)
            }
        );
        lending.add_available_base(Quantity::new(71548154787));

        // fee after 50 cycles 923240522808081.446499692862113448 (EXACT)
        // fee = 184186871548154787 * (Power[1+Divide[0.0001,3600],50*3600] - 1) = 923240522808082  (ROUNDED UP)

        assert_eq!(
            lending,
            Lend {
                // available = 1089518059629438139 + 71548154787 = 1089518131177592926
                available: Quantity::new(1089518131177592926),
                // borrowed = 184186871548154787 + 923240522808082 (ROUNDED UP) = 185110112070962869
                borrowed: Quantity::new(185110112070962869),
                borrow_shares: Shares::new(184186871548154787),
                unclaimed_fee: Quantity::new(0),
                borrow_limit: Quantity::new(u64::MAX),
                total_fee: Quantity::new(923240522808082),
                // utilization = Divide[185110065809380438,185110065809380438 + 1089518131177592926] = 0.1452267149 (ROUND UP)
                utilization: Utilization::from_scale(145227, 6),
                max_utilization,
                fee,
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        current_time += 50 * HOUR_DURATION;
        lending.accrue_interest_rate(current_time);
        let fee_q = lending.accrue_fee();

        assert_eq!(
            fee_q,
            Balances {
                base: Quantity::new(927868285122467),
                quote: Quantity::new(0)
            }
        );

        lending.borrow(Quantity::new(11051825915530))?;
        lending.remove_available_base(Quantity::new(11051825915530));

        // fee after 100 cycles : 923240522808082 + 927868285122466.00435945= 1851108807930549(ROUND UP)

        let _fee_q = lending.accrue_fee();

        assert_eq!(
            lending,
            Lend {
                // available = 1089518131177592926 - 11051825915530 = 1089507079351677396
                available: Quantity::new(1089507079351677396),
                // borrowed = 185110112070962869 + 927868285122467 (ROUNDED UP) + 11051825915530
                borrowed: Quantity::new(186049032182000866),
                // borrow_shares = 184186871548154787 * Divide[11051825915530, 186037980356085336]  + 184186871548154787
                // borrow_shares = 184197813406566601.9259
                borrow_shares: Shares::new(184197813406566602),
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                fee,
                utilization: Utilization::from_scale(145858, 6), // 0.145857129
                unclaimed_fee: Quantity::new(0),                 // ROUNDED UP
                total_fee: Quantity::new(1851108807930549),      // ROUNDED UP
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        let (repaid, first_repaid_shares, _) = lending.repay(
            Quantity::new(184186871548154787),
            Quantity::new(184186871548154787),
            Shares::new(184186871548154787),
        )?;

        lending.add_available_base(repaid);

        assert_eq!(
            lending,
            Lend {
                // available =  1089507079351677396 + 184186871548154787 = 1273693950899832183
                available: Quantity::new(1273693950899832183),
                // borrowed = 186049032182000866 - 184186871548154787 = 1862160633846079
                borrowed: Quantity::new(1862160633846079),
                // borrow_shares = 184197813406566602 - (184197813406566602 * Divide[184186871548154787, 186049032182000866] = 1843631826209616 (ROUND UP)
                borrow_shares: Shares::new(1843631826209616),
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                fee,
                // Divide[1862160633846079, 1273693950899832183 + 1862160633846079]
                utilization: Utilization::from_scale(1460, 6),
                unclaimed_fee: Quantity::new(0),
                total_fee: Quantity::new(1851108807930549),
                last_fee_paid: current_time,
                ..Default::default()
            }
        );

        let (repaid, second_repaid_shares, _) = lending.repay(
            Quantity::new(11051825915530),
            Quantity::new(11051825915530),
            Shares::new(10941858411815),
        )?;

        lending.add_available_base(repaid);

        assert_eq!(
            lending,
            Lend {
                // available = 1273693950899832183 + 11051825915530 = 1275556018546750731
                available: Quantity::new(1273705002725747713),
                borrowed: Quantity::new(1851108807930549),
                // borrow_shares = 1843631826209616 - (1843631826209616 * Divide[11051825915530, 1862160633846079] = 1832689967797802 (ROUND UP)
                borrow_shares: Shares::new(1832689967797802),
                // Divide[1851108807930549, 1273705002725747713 + 1851108807930549]
                utilization: Utilization::from_scale(1452, 6),
                fee,
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                last_fee_paid: current_time,
                unclaimed_fee: Quantity::new(0),
                total_fee: Quantity::new(1851108807930549),
                ..Default::default()
            }
        );

        assert_eq!(
            first_repaid_shares + second_repaid_shares,
            Shares::new(182365123438768800)
        );

        //repay to zero, merge 2 debts

        let (repaid, _, _) = lending.repay(
            Quantity::new(1851108807930549),
            Quantity::new(1851108807930549),
            Shares::new(1832689967797802),
        )?;

        lending.add_available_base(repaid);

        assert_eq!(
            lending,
            Lend {
                // available = 1273705002725747713 + 1851108807930549 = 1275556111533678262
                available: Quantity::new(1275556111533678262),
                fee,
                max_utilization,
                borrow_limit: Quantity::new(u64::MAX),
                last_fee_paid: current_time,
                total_fee: Quantity::new(1851108807930549),
                ..Default::default()
            }
        );

        Ok(())
    }
}
