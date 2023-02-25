use crate::core_lib::decimal::{Decimal, Price};
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

    #[wasm_bindgen]
    pub fn get_price_quote(&self, index: u8) -> u64 {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .oracle()
            .expect("oracle not initialized")
            .price
            .get()
    }

    #[wasm_bindgen]
    pub fn get_confidence_quote(&self, index: u8) -> u64 {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .quote_oracle()
            .expect("oracle not initialized")
            .confidence
            .get()
    }

    #[wasm_bindgen]
    pub fn update_oracle(&mut self, index: u8, price: u64, confidence: u64, time: u32) {
        let oracle = self
            .account
            .arr
            .get_mut_checked(index as usize)
            .expect("index out of bounds")
            .oracle_mut()
            .expect("oracle not initialized");

        oracle
            .update(Price::new(price), Price::new(confidence), time)
            .expect("Could not update oracle");
    }

    #[wasm_bindgen]
    pub fn update_quote_oracle(&mut self, index: u8, price: u64, confidence: u64, time: u32) {
        let oracle = self
            .account
            .arr
            .get_mut_checked(index as usize)
            .expect("index out of bounds")
            .quote_oracle_mut()
            .expect("oracle not initialized");

        oracle
            .update(Price::new(price), Price::new(confidence), time)
            .expect("Could not update oracle");
    }
}
