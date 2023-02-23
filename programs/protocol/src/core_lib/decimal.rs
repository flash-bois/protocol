use std::{
    default,
    ops::{Add, AddAssign, Sub, SubAssign},
};

pub use checked_decimal_macro::*;

pub type Time = u32;

/// Used to represent number of decimal points in a quantity of token
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DecimalPlaces {
    #[default]
    Six = 6,
    Nine = 9,
}

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Balances {
        /// Token characteristic for vault
        pub base: Quantity,
        /// Stable token
        pub quote: Quantity,
    }
    /// Used to represent a quantity of token (of its smallest unit)

    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(0, U256)]
    pub struct Quantity {
        pub val: u64,
    }

    /// Keeps fractions that need less precision
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(6)]
    pub struct Fraction {
        pub val: u64,
    }

    /// Keeps fractions that need less precision
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(6)]
    pub struct Utilization {
        pub val: u128,
    }

    /// Keeps fractions that need greater precision
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(12)]
    pub struct BigFraction {
        pub val: u128,
    }

    /// Keeps shares of pool or debt
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(0)]
    pub struct Shares {
        pub number_of_shares: u128,
    }

    /// Keeps price data
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(9)]
    pub struct Price {
        pub val: u64,
    }

    /// Keeps the value of a token, pool or position
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(9)]
    pub struct Value {
        pub val: u128,
    }

    /// Used to keep cumulative funding rate (can be positive or negative)
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(24)]
    pub struct FundingRate {
        pub val: i128,
    }

    /// Used for calculations that need more precision
    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(24)]
    pub struct Precise {
        pub val: u128,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
    #[repr(packed)]
    pub struct Balances {
        /// Token characteristic for vault
        pub base: Quantity,
        /// Stable token
        pub quote: Quantity,
    }

    /// Used to represent a quantity of token (of its smallest unit)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(0, U256)]
    pub struct Quantity {
        pub val: u64,
    }

    /// Keeps fractions that need less precision
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(6)]
    pub struct Fraction {
        pub val: u64,
    }

    /// Keeps fractions that need less precision
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(6)]
    pub struct Utilization {
        pub val: u128,
    }

    /// Keeps fractions that need greater precision
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[decimal(12)]
    pub struct BigFraction {
        pub val: u128,
    }

    /// Keeps shares of pool or debt
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(0)]
    pub struct Shares {
        pub number_of_shares: u128,
    }

    /// Keeps price data
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(9)]
    pub struct Price {
        pub val: u64,
    }

    /// Keeps the value of a token, pool or position
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(9)]
    pub struct Value {
        pub val: u128,
    }

    /// Used to keep cumulative funding rate (can be positive or negative)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(24)]
    pub struct FundingRate {
        pub val: i128,
    }

    /// Used for calculations that need more precision
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
    #[repr(packed)]
    #[decimal(24)]
    pub struct Precise {
        pub val: u128,
    }
}

#[cfg(feature = "anchor")]
pub use zero::*;

#[cfg(not(feature = "anchor"))]
pub use mon_zero::*;

impl Sub for Balances {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            base: self.base - other.base,
            quote: self.quote - other.quote,
        }
    }
}

impl Add for Balances {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            base: self.base + other.base,
            quote: self.quote + other.quote,
        }
    }
}

impl AddAssign for Balances {
    fn add_assign(&mut self, other: Self) {
        self.base += other.base;
        self.quote += other.quote;
    }
}

impl SubAssign for Balances {
    fn sub_assign(&mut self, other: Self) {
        self.base -= other.base;
        self.quote -= other.quote;
    }
}

impl Quantity {
    pub fn big_mul_div(&self, mul: Self, div: Self) -> Self {
        let res = self
            .big_mul_to_value(mul)
            .checked_div(U256::from(div.get()))
            .unwrap();

        Self { val: res.as_u64() }
    }
}

/// Number of seconds in 6 hours
pub const _RATE_INTERVAL: Time = 21600000u32;

impl Fraction {
    pub fn get_utilization(used: Quantity, total: Quantity) -> Self {
        if used == Quantity::from_integer(0) {
            return Self::from_integer(0);
        }

        Self::from_decimal(used).div_up(total)
    }
}

impl Utilization {
    pub fn get_utilization(used: Quantity, total: Quantity) -> Self {
        if used == Quantity::from_integer(0) {
            return Self::from_integer(0);
        }

        Self::from_decimal(used).div_up(total)
    }
}

impl Precise {
    pub fn pow(self, exp: u32) -> Self {
        let mut result = Self::from_integer(1);
        let mut base = self;
        let mut exp = exp;

        while exp > 0 {
            if exp % 2 == 1 {
                result *= base;
            }

            exp /= 2;
            base *= base;
        }

        result
    }

    pub fn pow_up(self, exp: u32) -> Self {
        let mut result = Self::from_integer(1);
        let mut base = self;
        let mut exp = exp;

        while exp > 0 {
            if exp % 2 == 1 {
                result = result.mul_up(base);
            }

            exp /= 2;
            base = base.mul_up(base);
        }

        result
    }

    pub fn big_pow_up(self, exp: u32) -> Self {
        let mut result = Self::from_integer(1);
        let mut base = self;
        let mut exp = exp;

        while exp > 0 {
            if exp % 2 == 1 {
                result = result.big_mul_up(base);
            }

            exp /= 2;
            base = base.big_mul_up(base);
        }

        result
    }
}

impl Shares {
    /// Calculates change in shares
    /// Change is rounded down
    ///
    /// ## Arguments
    ///
    /// * `amount` - Quantity of token to be shared
    /// * `all_liquidity` - Quantity of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * amount of shares to be changed
    pub fn get_change_down(self, amount: Quantity, all_liquidity: Quantity) -> Self {
        if self == Self::from_integer(0) {
            return amount.into();
        }

        self * amount / all_liquidity
    }

    /// Calculates change in shares by value
    /// Change is rounded down
    ///
    /// ## Arguments
    ///
    /// * `amount` - value of token to be shared
    /// * `all_liquidity` - value of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * amount of shares to be changed
    pub fn get_change_down_by_value(self, value: Value, all_value: Value) -> Self {
        if self == Self::from_integer(0) {
            return value.into();
        }

        self * value / all_value
    }

    /// Calculates change in shares
    /// Change is rounded up
    ///
    /// ## Arguments
    ///
    /// * `amount` - Quantity of token to be shared
    /// * `all_liquidity` - Quantity of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * amount of shares to be changed
    pub fn get_change_up(self, amount: Quantity, all_liquidity: Quantity) -> Self {
        if self == Self::from_integer(0) {
            return amount.into();
        }

        self.mul_up(amount).div_up(all_liquidity)
    }

    /// Calculates change in shares
    /// Change is rounded up
    ///
    /// ## Arguments
    ///
    /// * `amount` - Quantity of token to be shared
    /// * `all_liquidity` - Quantity of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * amount of shares to be changed
    pub fn get_change_up_by_value(self, value: Value, all_value: Value) -> Self {
        if self == Self::from_integer(0) {
            return value.into();
        }

        self.mul_up(value).div_up(all_value)
    }

    /// Calculate owned amount from total shares and provided shares
    /// Owed amount is rounded up
    ///
    /// ## Arguments
    ///
    /// * `shares_to_burn` - shares representing debt that are to be burned
    /// * `all_liquidity` - Quantity of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * Quantity owned
    pub fn calculate_owed(self, shares_to_burn: Shares, all_liquidity: Quantity) -> Quantity {
        shares_to_burn.mul_up(all_liquidity).div_up(self).into()
    }

    /// Calculate earned amount from total shares and provided shares
    /// Owed amount is rounded down
    ///
    /// ## Arguments
    ///
    /// * `shares_to_burn` - shares representing debt that are to be burned
    /// * `all_liquidity` - Quantity of token already shared
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * Quantity earned
    pub fn calculate_earned(self, shares_to_burn: Shares, all_liquidity: Quantity) -> Quantity {
        (shares_to_burn * all_liquidity / self).into()
    }

    /// Calculate earned value from total shares and provided shares
    /// Owed amount is rounded down
    ///
    /// ## Arguments
    ///
    /// * `shares_to_burn` - shares representing debt that are to be burned
    /// * `all_value` - value of shared tokens
    /// * `self` - shares of already shared token (sum of all shares)
    ///
    /// ## Returns:
    /// * Value earned
    pub fn calculate_earned_by_value(self, shares_to_burn: Shares, all_value: Value) -> Value {
        all_value * shares_to_burn / self
    }
}

impl From<Quantity> for Shares {
    fn from(q: Quantity) -> Self {
        Shares::from_decimal(q)
    }
}

impl From<Value> for Shares {
    fn from(q: Value) -> Self {
        Shares::from_decimal(q)
    }
}

impl From<Shares> for Quantity {
    fn from(f: Shares) -> Self {
        Quantity::from_decimal(f)
    }
}

#[cfg(test)]
mod big_tests {
    use super::*;

    #[test]
    fn big_mul_div() {
        let q = Quantity::new(12345678);
        let r = Quantity::new(65421512);
        let s = Quantity::new(42143214);

        assert_eq!(q.big_mul_div(r, s), q * r / s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let a = Quantity::new(1);
        let b = Quantity::new(2);
        let c = Quantity::new(3);

        assert_eq!(a + b, c);
        assert_eq!(b - a, a);
        assert_eq!(a * b, b);
        assert_eq!(b / a, b);
    }

    #[test]
    fn test_pow() {
        let two = Precise::from_integer(2);
        assert_eq!(two.big_pow_up(0), Precise::from_integer(1));
        assert_eq!(two.big_pow_up(5), Precise::from_integer(32));
        assert_eq!(
            Precise::from_scale(2, 1).big_pow_up(3),
            Precise::from_scale(8, 3)
        );
    }
}

#[cfg(test)]
mod test_shares {
    use checked_decimal_macro::Decimal;

    use super::{Quantity, Shares};

    fn increase_down_and_check(
        shares: &mut Shares,
        all_liquidity: &mut Quantity,
        input: Quantity,
        expected_increase: Shares,
    ) {
        assert_eq!(
            shares.get_change_down(input, *all_liquidity),
            expected_increase,
        );

        *shares += expected_increase;
        *all_liquidity += input;
    }

    fn increase_up_and_check(
        shares: &mut Shares,
        all_liquidity: &mut Quantity,
        input: Quantity,
        expected_increase: Shares,
    ) {
        assert_eq!(
            shares.get_change_up(input, *all_liquidity),
            expected_increase,
        );

        *shares += expected_increase;
        *all_liquidity += input;
    }

    fn decrease_down_and_check(
        shares: &mut Shares,
        all_liquidity: &mut Quantity,
        input: Quantity,
        expected_decrease: Shares,
    ) {
        assert_eq!(
            shares.get_change_down(input, *all_liquidity),
            expected_decrease,
        );

        *shares -= expected_decrease;
        *all_liquidity -= input;
    }

    fn decrease_up_and_check(
        shares: &mut Shares,
        all_liquidity: &mut Quantity,
        input: Quantity,
        expected_decrease: Shares,
    ) {
        assert_eq!(
            shares.get_change_up(input, *all_liquidity),
            expected_decrease,
        );

        *shares -= expected_decrease;
        *all_liquidity -= input;
    }

    #[test]
    fn increases_down() {
        let mut shares = Shares::new(0);
        let mut all_liquidity = Quantity::new(0);

        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(0),
            Shares::new(0),
        );
        // shares after = 0
        // liquidity after = 0
        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(1),
            Shares::new(1),
        );
        // shares after = 1
        // liquidity after = 1
        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(1),
            Shares::new(1),
        );
        // shares after = 2
        // liquidity after = 2

        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(2_135_642_322_912_235),
            Shares::new(2_135_642_322_912_235),
        );
        //shares after = 2135642322912237
        //liquidity after = 2135642322912237

        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(146_545_765_475_763),
            Shares::new(146_545_765_475_763),
        );
        // shares after = 2282188088388000
        // liquidity after = 2282188088388000

        //#######################################################
        //  now We're modifying all_liquidity to change "debt"  #

        all_liquidity += Quantity::new(3_412_563_665_124);

        // shares after = 2282188088388000
        // liquidity after = 2285600652053124
        //                                                      #
        //#######################################################

        increase_down_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(686_455_763_423),
            Shares::new(685_430_836_345),
        );

        // shares after = 2282873519224346
        // liquidity after = 2286287107816547

        assert_eq!(all_liquidity, Quantity::new(2_286_287_107_816_547));
        assert_eq!(shares, Shares::new(2_282_873_519_224_345))
    }

    #[test]
    fn increases_up() {
        let mut shares = Shares::new(0);
        let mut all_liquidity = Quantity::new(0);

        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(0),
            Shares::new(0),
        );
        // shares after = 0
        // liquidity after = 0
        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(1),
            Shares::new(1),
        );
        // shares after = 1
        // liquidity after = 1
        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(1),
            Shares::new(1),
        );
        // shares after = 2
        // liquidity after = 2
        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(2_135_642_322_912_235),
            Shares::new(2_135_642_322_912_235),
        );
        // shares after = 2135642322912237
        // liquidity after = 2135642322912237

        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(146_545_765_475_763),
            Shares::new(146_545_765_475_763),
        );
        // shares after = 2282188088388000
        // liquidity after = 2282188088388000

        //#######################################################
        //  now We're modifying all_liquidity to change "debt"  #

        all_liquidity += Quantity::new(3_412_563_665_124);

        // shares after = 2282188088388000
        // liquidity after = 2285600652053124
        //                                                      #
        //#######################################################

        increase_up_and_check(
            &mut shares,
            &mut all_liquidity,
            Quantity::new(686_455_763_423),
            Shares::new(685_430_836_346),
        );

        // shares after = 2282873519224346
        // liquidity after = 2286287107816547

        assert_eq!(all_liquidity, Quantity::new(2_286_287_107_816_547));
        assert_eq!(shares, Shares::new(2_282_873_519_224_346))
    }

    #[test]
    fn decrease_down_with_owned() {
        let mut shares = Shares::new(2282873519224346);
        let mut all_liquidity = Quantity::new(2286287107816547);

        let mut owed = shares.calculate_owed(Shares::new(0), all_liquidity);
        //0.0
        assert_eq!(owed, Quantity::new(0));
        decrease_down_and_check(&mut shares, &mut all_liquidity, owed, Shares::new(0));

        owed = shares.calculate_owed(Shares::new(1), all_liquidity);
        // 1.00149530342502
        assert_eq!(owed, Quantity::new(2));
        decrease_down_and_check(&mut shares, &mut all_liquidity, owed, Shares::new(1));

        // 1.00149530342502
        decrease_down_and_check(&mut shares, &mut all_liquidity, owed, Shares::new(1));

        owed = shares.calculate_owed(Shares::new(2_135_642_322_912_235), all_liquidity);
        // 2138835756192317.719
        assert_eq!(owed, Quantity::new(2138835756192318));
        decrease_down_and_check(
            &mut shares,
            &mut all_liquidity,
            owed,
            Shares::new(2_135_642_322_912_235),
        );

        owed = shares.calculate_owed(Shares::new(146_545_765_475_763), all_liquidity);
        // 146764895860801.79419
        assert_eq!(owed, Quantity::new(146764895860802));
        decrease_down_and_check(
            &mut shares,
            &mut all_liquidity,
            owed,
            Shares::new(146_545_765_475_763),
        );

        owed = shares.calculate_owed(Shares::new(685_430_836_346), all_liquidity);
        // 686455763423
        assert_eq!(owed, Quantity::new(686455763423));
        decrease_down_and_check(
            &mut shares,
            &mut all_liquidity,
            owed,
            Shares::new(685_430_836_346),
        );

        assert_eq!(shares, Shares::new(0));
        assert_eq!(all_liquidity, Quantity::new(0));
    }

    #[test]
    fn decrease_up_with_earned() {
        let mut shares = Shares::new(2282873519224345);
        let mut all_liquidity = Quantity::new(2286287107816547);

        let mut earned = shares.calculate_earned(Shares::new(0), all_liquidity);
        //0.0
        assert_eq!(earned, Quantity::new(0));
        decrease_up_and_check(&mut shares, &mut all_liquidity, earned, Shares::new(0));

        earned = shares.calculate_earned(Shares::new(1), all_liquidity);
        // 1.00149530342502 (DOWN) so 1
        assert_eq!(earned, Quantity::new(1));
        decrease_up_and_check(&mut shares, &mut all_liquidity, earned, Shares::new(1));

        // 1.00149530342502 (DOWN) so 1
        decrease_up_and_check(&mut shares, &mut all_liquidity, earned, Shares::new(1));

        earned = shares.calculate_earned(Shares::new(2_135_642_322_912_235), all_liquidity);
        // 2.138835756192320.5275 (DOWN)
        assert_eq!(earned, Quantity::new(2138835756192320));
        decrease_up_and_check(
            &mut shares,
            &mut all_liquidity,
            earned,
            Shares::new(2_135_642_322_912_235),
        );

        earned = shares.calculate_earned(Shares::new(146_545_765_475_763), all_liquidity);
        // 1.46764895860802.7910 (DOWN)
        assert_eq!(earned, Quantity::new(146764895860802));
        decrease_up_and_check(
            &mut shares,
            &mut all_liquidity,
            earned,
            Shares::new(146_545_765_475_763),
        );

        earned = shares.calculate_earned(Shares::new(685_430_836_345), all_liquidity);
        // 6.86455763423 (DOWN)
        assert_eq!(earned, Quantity::new(686455763423));
        decrease_up_and_check(
            &mut shares,
            &mut all_liquidity,
            earned,
            Shares::new(685_430_836_345),
        );

        assert_eq!(shares, Shares::new(0));
        assert_eq!(all_liquidity, Quantity::new(0));
    }
}
