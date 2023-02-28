use crate::{
    core_lib::{errors::LibErrors, strategy::Strategy},
    structs::VaultsAccount,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    fn strategy(&self, vault: u8, strategy: u8) -> Result<&Strategy, LibErrors> {
        Ok(self
            .account
            .vault_checked(vault)?
            .strategies
            .get_strategy(strategy)?)
    }

    #[wasm_bindgen]
    pub fn does_lend(&self, vault: u8, strategy: u8) -> Result<bool, JsError> {
        Ok(self.strategy(vault, strategy)?.is_lending_enabled())
    }

    #[wasm_bindgen]
    pub fn does_swap(&self, vault: u8, strategy: u8) -> Result<bool, JsError> {
        Ok(self.strategy(vault, strategy)?.is_swapping_enabled())
    }
}
