use crate::{
    core_lib::{
        decimal::{Decimal, Price},
        structs::Oracle,
    },
    structs::VaultsAccount,
};

use wasm_bindgen::prelude::*;

impl VaultsAccount {
    pub fn oracle(&self, index: u8) -> Result<&Oracle, JsValue> {
        Ok(self.vault_checked(index)?.oracle()?)
    }

    pub fn quote_oracle(&self, index: u8) -> Result<&Oracle, JsValue> {
        Ok(self.vault_checked(index)?.quote_oracle()?)
    }

    pub fn oracle_mut(&mut self, index: u8) -> Result<&mut Oracle, JsValue> {
        Ok(self.vault_checked_mut(index)?.oracle_mut()?)
    }

    pub fn quote_oracle_mut(&mut self, index: u8) -> Result<&mut Oracle, JsValue> {
        Ok(self.vault_checked_mut(index)?.quote_oracle_mut()?)
    }
}

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn get_price(&self, index: u8) -> Result<u64, JsValue> {
        Ok(self.oracle(index)?.price.get())
    }

    #[wasm_bindgen]
    pub fn get_confidence(&self, index: u8) -> Result<u64, JsValue> {
        Ok(self.oracle(index)?.confidence.get())
    }

    #[wasm_bindgen]
    pub fn get_price_quote(&self, index: u8) -> Result<u64, JsValue> {
        Ok(self.quote_oracle(index)?.price.get())
    }

    #[wasm_bindgen]
    pub fn get_confidence_quote(&self, index: u8) -> Result<u64, JsValue> {
        Ok(self.quote_oracle(index)?.confidence.get())
    }

    #[wasm_bindgen]
    pub fn update_oracle(
        &mut self,
        index: u8,
        price: u64,
        confidence: u64,
        time: u32,
    ) -> Result<(), JsValue> {
        let oracle = self.oracle_mut(index)?;

        Ok(oracle.update(Price::new(price), Price::new(confidence), time)?)
    }

    #[wasm_bindgen]
    pub fn update_quote_oracle(
        &mut self,
        index: u8,
        price: u64,
        confidence: u64,
        time: u32,
    ) -> Result<(), JsValue> {
        let oracle = self.quote_oracle_mut(index)?;

        Ok(oracle.update(Price::new(price), Price::new(confidence), time)?)
    }
}
