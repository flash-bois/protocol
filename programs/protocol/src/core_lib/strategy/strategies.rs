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
    #[derive(Debug, SafeArray, PartialEq)]
    pub struct Strategies {
        pub head: u8,
        pub elements: [Strategy; 6],
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;
    #[repr(packed)]
    #[derive(Clone, Copy, Debug, SafeArray, PartialEq)]
    pub struct Strategies {
        pub head: u8,
        pub elements: [Strategy; 6],
    }
}

#[cfg(feature = "anchor")]
pub use zero::Strategies;

#[cfg(not(feature = "anchor"))]
pub use mon_zero::Strategies;
