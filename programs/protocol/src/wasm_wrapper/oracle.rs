use crate::core_lib::decimal::Decimal;
use crate::structs::VaultsAccount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn get_price(&self, index: u8) -> u64 {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .oracle
            .expect("oracle not initialized")
            .price
            .get()
    }

    #[wasm_bindgen]
    pub fn get_confidence(&self, index: u8) -> u64 {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .oracle
            .expect("oracle not initialized")
            .confidence
            .get()
    }
}
