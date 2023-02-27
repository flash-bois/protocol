use crate::core_lib::decimal::{Decimal, Fraction, Price};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn price_denominator() -> u64 {
    Price::one()
}

#[wasm_bindgen]
pub fn fraction_denominator() -> u64 {
    Fraction::one()
}
