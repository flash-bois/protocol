use crate::{
    core_lib::errors::LibErrors,
    structs::{Vaults, VaultsAccount},
    wasm_wrapper::utils::to_buffer,
    ZeroCopyDecoder,
};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn load(account_info: &Uint8Array) -> Self {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode_account_info::<Vaults>(&v);
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
    pub fn base_token(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.account.keys_checked(index)?.base_token))
    }

    #[wasm_bindgen]
    pub fn quote_token(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.account.keys_checked(index)?.quote_token))
    }

    #[wasm_bindgen]
    pub fn base_reserve(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.account.keys_checked(index)?.base_reserve))
    }

    #[wasm_bindgen]
    pub fn quote_reserve(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.account.keys_checked(index)?.quote_reserve))
    }

    #[wasm_bindgen]
    pub fn oracle_base(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(
            &self
                .account
                .keys_checked(index)?
                .base_oracle
                .ok_or(LibErrors::OracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn oracle_quote(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(
            &self
                .account
                .keys_checked(index)?
                .quote_oracle
                .ok_or(LibErrors::QuoteOracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn base_oracle_enabled(&self, index: u8) -> Result<bool, JsError> {
        Ok(self.account.vault_checked(index)?.oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn quote_oracle_enabled(&self, index: u8) -> Result<bool, JsError> {
        Ok(self.account.vault_checked(index)?.quote_oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn has_lending(&mut self, index: u8) -> Result<bool, JsError> {
        Ok(self
            .account
            .vault_checked_mut(index)?
            .lend_service()
            .is_ok())
    }

    #[wasm_bindgen]
    pub fn has_swap(&mut self, index: u8) -> Result<bool, JsError> {
        Ok(self
            .account
            .vault_checked_mut(index)?
            .swap_service()
            .is_ok())
    }
}
