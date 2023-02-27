use crate::core_lib::errors::LibErrors;

use super::Strategy;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};
use vec_macro::SafeArray;

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, SafeArray, PartialEq)]
    pub struct Strategies {
        pub head: u8,
        pub elements: [Strategy; 6],
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, SafeArray, PartialEq)]
    pub struct Strategies {
        pub head: u8,
        pub elements: [Strategy; 6],
    }
}

#[cfg(feature = "anchor")]
pub use zero::Strategies;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Strategies;

impl Strategies {
    pub fn get_strategy_mut(&mut self, id: u8) -> Result<&mut Strategy, LibErrors> {
        Ok(self
            .get_mut_checked(id as usize)
            .ok_or(LibErrors::NoStrategyOnIndex)?)
    }

    pub fn get_strategy(&self, id: u8) -> Result<&Strategy, LibErrors> {
        Ok(self
            .get_checked(id as usize)
            .ok_or(LibErrors::NoStrategyOnIndex)?)
    }
}

#[cfg(test)]
mod tt {
    use super::*;

    #[test]
    fn get_size() {
        println!("{}", std::mem::size_of::<Strategy>());
        println!()
    }
}
