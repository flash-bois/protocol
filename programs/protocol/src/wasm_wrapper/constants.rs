use crate::core_lib::decimal::{Decimal, Price};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn price_denominator() -> u64 {
    Price::one()
}
