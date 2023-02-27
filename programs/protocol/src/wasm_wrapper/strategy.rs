use crate::core_lib::errors::LibErrors;
use crate::core_lib::strategy::Strategy;
use crate::structs::VaultsAccount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    fn strategy(&self, vault: u8, strategy: u8) -> Result<&Strategy, JsValue> {
        Ok(self
            .account
            .arr
            .get_checked(vault as usize)
            .ok_or(JsValue::from(LibErrors::NoVaultOnIndex))?
            .strategies
            .get_checked(strategy as usize)
            .expect("not such strategy"))
    }

    #[wasm_bindgen]
    pub fn does_lend(&self, vault: u8, strategy: u8) -> bool {
        self.strategy(vault, strategy).unwrap().is_lending_enabled()
    }

    #[wasm_bindgen]
    pub fn does_swap(&self, vault: u8, strategy: u8) -> bool {
        self.strategy(vault, strategy)
            .unwrap()
            .is_swapping_enabled()
    }
}
