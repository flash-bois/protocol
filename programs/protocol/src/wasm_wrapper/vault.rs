use std::ops::{Deref, DerefMut};

use crate::{
    core_lib::{errors::LibErrors, structs::Side, Vault},
    structs::{VaultKeys, Vaults},
    wasm_wrapper::utils::to_buffer,
    ZeroCopyDecoder,
};
use checked_decimal_macro::Decimal;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VaultsAccount {
    account: Vaults,
}

impl Deref for VaultsAccount {
    type Target = Vaults;
    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl DerefMut for VaultsAccount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

impl VaultsAccount {
    pub fn vault_checked(&self, index: u8) -> Result<&Vault, LibErrors> {
        Ok(self
            .arr
            .get_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }

    pub fn keys_checked(&self, index: u8) -> Result<&VaultKeys, LibErrors> {
        Ok(self
            .keys
            .get_checked(index as usize)
            .ok_or(LibErrors::IndexOutOfBounds)?)
    }

    pub fn vault_checked_mut(&mut self, index: u8) -> Result<&mut Vault, LibErrors> {
        Ok(self
            .arr
            .get_mut_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct VaultsKeysWithId {
    pub base_key: Uint8Array,
    pub quote_key: Uint8Array,
    pub index: u8,
}

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn load(account_info: &Uint8Array) -> Self {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode::<Vaults>(&v);
        Self { account }
    }

    pub fn reload(&mut self, account_info: &Uint8Array) {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode::<Vaults>(&v);

        self.account = account
    }

    pub fn buffer(&self) -> Uint8Array {
        to_buffer(ZeroCopyDecoder::encode(&self.account))
    }

    #[wasm_bindgen]
    pub fn vaults_len(&self) -> u8 {
        self.arr.head
    }

    #[wasm_bindgen]
    pub fn size() -> usize {
        std::mem::size_of::<Vaults>()
    }

    #[wasm_bindgen]
    pub fn vaults_keys_with_id(&self) -> Result<Array, JsError> {
        let arr = Array::new();

        for index in self.arr.indexes() {
            let base_key = to_buffer(&self.keys_checked(index as u8)?.base_token);
            let quote_key = to_buffer(&self.keys_checked(index as u8)?.quote_token);

            arr.push(&JsValue::from(VaultsKeysWithId {
                base_key,
                quote_key,
                index: index as u8,
            }));
        }

        Ok(arr)
    }

    #[wasm_bindgen]
    pub fn base_token(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.keys_checked(index)?.base_token))
    }

    #[wasm_bindgen]
    pub fn quote_token(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.keys_checked(index)?.quote_token))
    }

    #[wasm_bindgen]
    pub fn base_reserve(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.keys_checked(index)?.base_reserve))
    }

    #[wasm_bindgen]
    pub fn quote_reserve(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.keys_checked(index)?.quote_reserve))
    }

    #[wasm_bindgen]
    pub fn oracle_base(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(
            &self
                .keys_checked(index)?
                .base_oracle
                .ok_or(LibErrors::OracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn oracle_quote(&self, index: u8) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(
            &self
                .keys_checked(index)?
                .quote_oracle
                .ok_or(LibErrors::QuoteOracleNone)?,
        ))
    }

    #[wasm_bindgen]
    pub fn base_oracle_enabled(&self, index: u8) -> Result<bool, JsError> {
        Ok(self.vault_checked(index)?.oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn quote_oracle_enabled(&self, index: u8) -> Result<bool, JsError> {
        Ok(self.vault_checked(index)?.quote_oracle.is_some())
    }

    #[wasm_bindgen]
    pub fn has_lending(&mut self, index: u8) -> Result<bool, JsError> {
        Ok(self.vault_checked_mut(index)?.lend_service().is_ok())
    }

    #[wasm_bindgen]
    pub fn has_swap(&mut self, index: u8) -> Result<bool, JsError> {
        Ok(self.vault_checked_mut(index)?.swap_service().is_ok())
    }

    #[wasm_bindgen]
    pub fn has_trading(&mut self, index: u8) -> Result<bool, JsError> {
        Ok(self.vault_checked_mut(index)?.trade_service().is_ok())
    }

    #[wasm_bindgen]
    pub fn refresh(&mut self, index: u8, current_time: u32) -> Result<(), JsError> {
        Ok(self.vault_checked_mut(index)?.refresh(current_time)?)
    }

    #[wasm_bindgen]
    pub fn borrow_limit(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .lend_service_not_mut()?
            .borrow_limit
            .get())
    }

    #[wasm_bindgen]
    pub fn available_lend(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .lend_service_not_mut()?
            .available
            .get())
    }

    #[wasm_bindgen]
    pub fn utilization_lend(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .lend_service_not_mut()?
            .utilization
            .get() as u64)
    }

    #[wasm_bindgen]
    pub fn max_utilization(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .lend_service_not_mut()?
            .max_utilization
            .get() as u64)
    }

    pub fn current_fee(&self, index: u8) -> Result<u64, JsError> {
        Ok(
            if let Ok(lend) = self.vault_checked(index)?.lend_service_not_mut() {
                lend.current_fee()?.get()
            } else {
                0
            },
        )
    }

    #[wasm_bindgen]
    pub fn lending_apy(&mut self, index: u8, duration_in_secs: u32) -> Result<u64, JsError> {
        Ok(
            if let Ok(lend) = self.vault_checked_mut(index)?.lend_service() {
                lend.get_apy(duration_in_secs).get()
            } else {
                0
            },
        )
    }

    #[wasm_bindgen]
    pub fn max_leverage(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .trade_service_not_mut()?
            .max_open_leverage()
            .get())
    }

    #[wasm_bindgen]
    pub fn trading_open_fee(&self, index: u8) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .trade_service_not_mut()?
            .open_fee()
            .get())
    }

    #[wasm_bindgen]
    pub fn trading_fee(&self, index: u8, long: bool) -> Result<u64, JsError> {
        Ok(self
            .vault_checked(index)?
            .trade_service_not_mut()?
            .borrow_fee(if long { Side::Long } else { Side::Short })?
            .get())
    }

    #[wasm_bindgen]
    pub fn refresh_lend_fees(&mut self, current_time: u32) -> Result<(), JsError> {
        if let Some(mut iter) = self.arr.iter_mut() {
            while let Some(vault) = iter.next() {
                vault.refresh(current_time)?
            }
        };

        Ok(())
    }
}
