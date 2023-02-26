use crate::core_lib::strategy::Strategy;
use crate::structs::VaultsAccount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    fn strategy(&self, vault: u8, strategy: u8) -> &Strategy {
        self.account
            .arr
            .get_checked(vault as usize)
            .expect("not such vault")
            .strategies
            .get_checked(strategy as usize)
            .expect("not such strategy")
    }

    #[wasm_bindgen]
    pub fn does_lend(&self, vault: u8, strategy: u8) -> bool {
        self.strategy(vault, strategy).is_lending_enabled()
    }

    #[wasm_bindgen]
    pub fn does_swap(&self, vault: u8, strategy: u8) -> bool {
        self.strategy(vault, strategy).is_swapping_enabled()
    }
}
