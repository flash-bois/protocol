use crate::core_lib::{decimal::*, errors::LibErrors};

const MAX_FEES: usize = 5;
pub const HOUR_DURATION: u32 = 60 * 60;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CurveSegment {
    #[default]
    None,
    Constant {
        c: Fraction,
    },
    Linear {
        a: Fraction,
        b: Fraction,
    },
}

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Default, Debug, PartialEq, Eq)]
    pub struct FeeCurve {
        pub used: u8,
        pub bounds: [Fraction; MAX_FEES],
        pub values: [CurveSegment; MAX_FEES],
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    pub struct FeeCurve {
        pub used: u8,
        pub bounds: [Fraction; MAX_FEES],
        pub values: [CurveSegment; MAX_FEES],
    }
}

#[cfg(feature = "anchor")]
pub use zero::FeeCurve;

#[cfg(not(feature = "anchor"))]
pub use non_zero::FeeCurve;

impl FeeCurve {
    fn find_index(&self, utilization: Fraction) -> usize {
        (0..self.used as usize)
            .find(|&i| utilization <= self.bounds[i])
            .unwrap_or(0)
    }

    fn find_indexes(
        &self,
        smaller: Fraction,
        greater: Fraction,
    ) -> Result<(usize, usize), LibErrors> {
        let index = self.find_index(smaller);

        Ok((
            index,
            (index..self.used as usize)
                .find(|&i| greater <= self.bounds[i])
                .ok_or(LibErrors::ToBeDefined)?,
        ))
    }

    fn single_segment_mean(
        &self,
        function: CurveSegment,
        lower: Fraction,
        upper: Fraction,
    ) -> Fraction {
        match function {
            CurveSegment::None => Fraction::from_integer(0),
            CurveSegment::Constant { c } => c,
            CurveSegment::Linear { a, b } => {
                (lower + upper).mul_up(a / Fraction::from_integer(2)) + b
            }
        }
    }

    pub fn get_point_fee(&self, utilization: Fraction) -> Result<Fraction, LibErrors> {
        let index = self.find_index(utilization);

        Ok(match self.values[index] {
            CurveSegment::None => Fraction::from_integer(0),
            CurveSegment::Constant { c } => c,
            CurveSegment::Linear { a, b } => a.mul_up(utilization) + b,
        })
    }

    pub fn get_mean(&self, before: Fraction, after: Fraction) -> Result<Fraction, LibErrors> {
        let (smaller, greater) = if before < after {
            (before, after)
        } else {
            (after, before)
        };

        let (smaller_index, greater_index) = self.find_indexes(smaller, greater)?;

        if smaller_index == greater_index {
            // most common case by far
            return Ok(self.single_segment_mean(self.values[smaller_index], smaller, greater));
        }

        let mut sum =
            ((smaller_index + 1)..greater_index).fold(Fraction::from_integer(0), |sum, index| {
                sum + self
                    .single_segment_mean(
                        self.values[index],
                        self.bounds[index - 1],
                        self.bounds[index],
                    )
                    .mul_up(self.bounds[index] - self.bounds[index - 1])
            });

        sum += self
            .single_segment_mean(
                self.values[smaller_index],
                smaller,
                self.bounds[smaller_index],
            )
            .mul_up(self.bounds[smaller_index] - smaller);

        sum += self
            .single_segment_mean(
                self.values[greater_index],
                self.bounds[greater_index - 1],
                greater,
            )
            .mul_up(greater - self.bounds[greater_index - 1]);

        Ok(sum.div_up(greater - smaller))
    }

    pub fn get_value(&self, utilization: Fraction) -> CurveSegment {
        let index = self.find_index(utilization);
        self.values[index]
    }

    pub fn add_constant_fee(&mut self, fee: Fraction, bound: Fraction) -> &mut Self {
        self.add_segment(CurveSegment::Constant { c: fee }, bound);
        self
    }

    pub fn add_linear_fee(&mut self, a: Fraction, b: Fraction, bound: Fraction) -> &mut Self {
        self.add_segment(CurveSegment::Linear { a, b }, bound);
        self
    }

    fn add_segment(&mut self, curve: CurveSegment, bound: Fraction) {
        self.bounds[self.used as usize] = bound;
        self.values[self.used as usize] = curve;
        self.used += 1;

        self.bounds[..self.used as usize].sort();
    }

    pub fn compounded_fee(&self, utilization: Fraction, time: Time) -> Precise {
        let fee = self
            .get_point_fee(utilization)
            .expect("compounded_fee: invalid fee");

        let fee = Precise::from_decimal(fee).div_up(Quantity::new(HOUR_DURATION as u64));
        (Precise::from_integer(1) + fee).big_pow_up(time) - Precise::from_integer(1)
    }

    pub fn compounded_apy(&self, utilization: Fraction, time: Time) -> PreciseApy {
        let fee = self
            .get_point_fee(utilization)
            .expect("compounded_fee: invalid fee");

        let fee = PreciseApy::from_decimal(fee).div_up(Quantity::new(HOUR_DURATION as u64));
        (PreciseApy::from_integer(1) + fee).big_pow_up(time) - PreciseApy::from_integer(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compounded_apy_max() {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::new(5000), Fraction::from_integer(1));

        fee.compounded_apy(Fraction::new(1), 365 * 24 * 60 * 60);
    }

    #[test]
    fn test_find_index() {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::new(1), Fraction::new(1));
        fee.add_constant_fee(Fraction::new(2), Fraction::new(2));
        fee.add_constant_fee(Fraction::new(3), Fraction::new(3));

        assert_eq!(fee.find_index(Fraction::new(0)), 0);
        assert_eq!(fee.find_index(Fraction::new(1)), 0);
        assert_eq!(fee.find_index(Fraction::new(2)), 1);
        assert_eq!(fee.find_index(Fraction::new(3)), 2);
    }

    #[test]
    fn test() -> Result<(), LibErrors> {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::new(1), Fraction::new(1));
        fee.add_constant_fee(Fraction::new(2), Fraction::new(2));
        fee.add_constant_fee(Fraction::new(3), Fraction::new(3));

        assert_eq!(
            fee.find_indexes(Fraction::new(0), Fraction::new(0))?,
            (0, 0)
        );
        assert_eq!(
            fee.find_indexes(Fraction::new(1), Fraction::new(3))?,
            (0, 2)
        );
        assert_eq!(
            fee.find_indexes(Fraction::new(2), Fraction::new(2))?,
            (1, 1)
        );
        assert_eq!(
            fee.find_indexes(Fraction::new(3), Fraction::new(3))?,
            (2, 2)
        );
        assert!(fee
            .find_indexes(Fraction::new(4), Fraction::new(4))
            .is_err());
        Ok(())
    }

    #[test]
    fn test_calculate_compounded() {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_scale(5, 1));
        fee.add_constant_fee(Fraction::from_scale(2, 2), Fraction::from_integer(1));

        assert_eq!(
            fee.compounded_fee(Fraction::from_scale(2, 1), HOUR_DURATION),
            Precise::new(10050153055719590731686) // 1005015305571959072853
        );

        assert_eq!(
            fee.compounded_fee(Fraction::from_scale(6, 1), 60),
            Precise::new(333387968831054398543) // 33338796883105439847515487389689
        );
    }

    #[test]
    fn test_get_mean() -> Result<(), LibErrors> {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_scale(5, 1));
        fee.add_constant_fee(Fraction::from_scale(2, 2), Fraction::from_integer(1));

        assert_eq!(
            fee.get_mean(Fraction::from_scale(0, 0), Fraction::from_scale(5, 1))?,
            Fraction::from_scale(1, 2)
        );
        assert_eq!(
            fee.get_mean(Fraction::from_scale(6, 1), Fraction::from_scale(9, 1))?,
            Fraction::from_scale(2, 2)
        );
        assert_eq!(
            fee.get_mean(Fraction::from_scale(4, 1), Fraction::from_scale(6, 1))?,
            Fraction::from_scale(15, 3)
        );
        Ok(())
    }
}
