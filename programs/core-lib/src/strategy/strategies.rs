use super::Strategy;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};
use vec_macro::SafeArray;

#[derive(Clone, Debug, SafeArray)]
pub struct Strategies {
    head: u8,
    elements: [Strategy; 6],
}
