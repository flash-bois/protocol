use crate::{
    core_lib::{decimal::Fraction, errors::LibErrors, Vault},
    structs::{VaultKeys, Vaults, VaultsAccount},
    wasm_wrapper::utils::to_buffer,
    ZeroCopyDecoder,
};
use checked_decimal_macro::{BetweenDecimals, Decimal};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

impl VaultsAccount {
    pub fn vault_checked(&self, index: u8) -> Result<&Vault, JsValue> {
        Ok(self
            .account
            .arr
            .get_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }

    pub fn keys_checked(&self, index: u8) -> Result<&VaultKeys, JsValue> {
        Ok(self
            .account
            .keys
            .get_checked(index as usize)
            .ok_or(LibErrors::IndexOutOfBounds)?)
    }

    pub fn vault_checked_mut(&mut self, index: u8) -> Result<&mut Vault, JsValue> {
        Ok(self
            .account
            .arr
            .get_mut_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }
}

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
    pub fn base_token(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(&self.keys_checked(index)?.base_token))
    }

    #[wasm_bindgen]
    pub fn quote_token(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(&self.keys_checked(index)?.quote_token))
    }

    #[wasm_bindgen]
    pub fn base_reserve(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(&self.keys_checked(index)?.base_reserve))
    }

    #[wasm_bindgen]
    pub fn quote_reserve(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(&self.keys_checked(index)?.quote_reserve))
    }

    #[wasm_bindgen]
    pub fn oracle_base(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(
            &self
                .keys_checked(index)?
                .base_oracle
                .ok_or(LibErrors::OracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn oracle_quote(&self, index: u8) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(
            &self
                .keys_checked(index)?
                .quote_oracle
                .ok_or(LibErrors::QuoteOracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn base_oracle_enabled(&self, index: u8) -> Result<bool, JsValue> {
        Ok(self.vault_checked(index)?.oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn quote_oracle_enabled(&self, index: u8) -> Result<bool, JsValue> {
        Ok(self.vault_checked(index)?.quote_oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn has_lending(&mut self, index: u8) -> Result<bool, JsValue> {
        Ok(self.vault_checked_mut(index)?.lend_service().is_ok())
    }

    #[wasm_bindgen]
    pub fn has_swap(&mut self, index: u8) -> Result<bool, JsValue> {
        Ok(self.vault_checked_mut(index)?.swap_service().is_ok())
    }

    #[wasm_bindgen]
    pub fn lending_apy(&mut self, index: u8) -> Result<u64, JsValue> {
        Ok(
            if let Ok(lend) = self.vault_checked_mut(index)?.lend_service() {
                let utilization = lend.current_utilization();
                let fee_curve = lend.fee_curve();
                Fraction::from_decimal(
                    fee_curve
                        .compounded_fee(Fraction::from_decimal(utilization), 60 * 60 * 24 * 365),
                )
                .get()
            } else {
                0
            },
        )
    }
}
