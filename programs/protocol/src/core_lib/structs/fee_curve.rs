use crate::core_lib::decimal::*;

const MAX_FEES: usize = 5;

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
    #[repr(packed)]
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
    #[repr(packed)]
    pub struct FeeCurve {
        pub used: usize,
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

    fn find_indexes(&self, smaller: Fraction, greater: Fraction) -> Result<(usize, usize), ()> {
        let index = (0..self.used as usize)
            .find(|&i| smaller <= self.bounds[i])
            .unwrap_or(0);

        Ok((
            index,
            (index..self.used as usize)
                .find(|&i| greater <= self.bounds[i])
                .ok_or(())?,
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

    pub fn get_mean(&self, before: Fraction, after: Fraction) -> Result<Fraction, ()> {
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
        if let CurveSegment::Constant { c } = self.get_value(utilization) {
            let fee = Precise::from_decimal(c);
            (Precise::from_integer(1) + fee).big_pow_up(time) - Precise::from_integer(1)
        } else {
            panic!("compounded_fee: invalid function");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test() -> Result<(), ()> {
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
    fn test_calculate_static() {
        let mut fee = FeeCurve::default();
        fee.add_constant_fee(Fraction::from_scale(1, 2), Fraction::from_scale(5, 1));
        fee.add_constant_fee(Fraction::from_scale(2, 2), Fraction::from_integer(1));

        assert_eq!(
            fee.compounded_fee(Fraction::from_scale(2, 1), 1),
            Precise::from_scale(1, 2)
        );
        assert_eq!(
            fee.compounded_fee(Fraction::from_scale(6, 1), 2),
            Precise::from_scale(404, 4)
        );
    }

    #[test]
    fn test_get_mean() -> Result<(), ()> {
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
