use super::Strategy;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::{
    ops::Range,
    slice::{Iter, IterMut},
};
use vec_macro::fixed_vector;

#[derive(Clone, Debug)]
#[fixed_vector(Strategy, 6)]
pub struct Strategies {
    head: u8,
    elements: [Strategy; 6],
}
