use wasm_bindgen::prelude::*;

use crate::core_lib::errors::LibErrors;

#[wasm_bindgen]
pub fn ret_error() -> Result<i32, JsValue> {
    Err(JsValue::from(LibErrors::IndexOutOfBounds))
}
