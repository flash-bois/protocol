use crate::structs::vaults::Vaults;
use crate::structs::VaultsAccount;
use crate::wasm_wrapper::utils::to_buffer;
use checked_decimal_macro::Decimal;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn load(account_info: &Uint8Array) -> Self {
        console_error_panic_hook::set_once();
        // panic!("not a panic");

        let v = account_info.to_vec();
        let account = *bytemuck::from_bytes::<Vaults>(&v);
        Self { account }
    }

    #[wasm_bindgen]
    pub fn vaults_len(&self) -> u8 {
        self.account.arr.head
    }

    #[wasm_bindgen]
    pub fn size() -> usize {
        std::mem::size_of::<Vaults>()
    }

    #[wasm_bindgen]
    pub fn base_token(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .base_token,
        )
    }

    #[wasm_bindgen]
    pub fn quote_token(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .quote_token,
        )
    }

    #[wasm_bindgen]
    pub fn base_reserve(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .base_reserve,
        )
    }

    #[wasm_bindgen]
    pub fn quote_reserve(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .quote_reserve,
        )
    }

    #[wasm_bindgen]
    pub fn oracle_base(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .base_oracle
                .expect("base oracle not initialized"), // ERROR CODE
        )
    }

    #[wasm_bindgen]
    pub fn oracle_quote(&self, index: u8) -> Uint8Array {
        to_buffer(
            &self
                .account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .quote_oracle
                .expect("quote oracle not initialized"), // ERROR CODE
        )
    }

    #[wasm_bindgen]
    pub fn base_oracle_enabled(&self, index: u8) -> bool {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .oracle
            .is_some()
    }

    #[wasm_bindgen]
    pub fn quote_oracle_enabled(&self, index: u8) -> bool {
        self.account
            .arr
            .get_checked(index as usize)
            .expect("index out of bounds")
            .quote_oracle
            .is_some()
    }
}
