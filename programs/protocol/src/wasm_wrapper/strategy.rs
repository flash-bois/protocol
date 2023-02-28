use super::vault::VaultsAccount;
use crate::core_lib::strategy::Strategy;
use checked_decimal_macro::Decimal;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    fn strategy(&self, vault: u8, strategy: u8) -> Result<&Strategy, JsValue> {
        Ok(self
            .vault_checked(vault)?
            .strategies
            .get_strategy(strategy)?)
    }

    #[wasm_bindgen]
    pub fn strategies(&self, vault: u8) -> Result<u8, JsValue> {
        Ok(self.vault_checked(vault)?.strategies.head - 1)
    }

    #[wasm_bindgen]
    pub fn does_lend(&self, vault: u8, strategy: u8) -> Result<bool, JsValue> {
        Ok(self.strategy(vault, strategy)?.is_lending_enabled())
    }

    #[wasm_bindgen]
    pub fn does_swap(&self, vault: u8, strategy: u8) -> Result<bool, JsValue> {
        Ok(self.strategy(vault, strategy)?.is_swapping_enabled())
    }

    #[wasm_bindgen]
    pub fn balance_base(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        Ok(self.strategy(vault, strategy)?.balance().get())
    }

    #[wasm_bindgen]
    pub fn balance_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        Ok(self.strategy(vault, strategy)?.balance_quote().get())
    }

    #[wasm_bindgen]
    pub fn lock_base(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        Ok(self.strategy(vault, strategy)?.locked().get())
    }

    #[wasm_bindgen]
    pub fn lock_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        Ok(self.strategy(vault, strategy)?.locked_quote().get())
    }

    #[wasm_bindgen]
    pub fn utilization_base(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        let strategy = self.strategy(vault, strategy)?;
        Ok((strategy.locked() / strategy.balance()).get())
    }

    #[wasm_bindgen]
    pub fn utilization_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsValue> {
        let strategy = self.strategy(vault, strategy)?;
        Ok((strategy.locked_quote() / strategy.balance_quote()).get())
    }
}
