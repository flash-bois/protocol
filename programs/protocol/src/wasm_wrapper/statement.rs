use crate::structs::Statement;
use crate::wasm_wrapper::to_buffer;
use crate::ZeroCopyDecoder;
use checked_decimal_macro::Decimal;
use js_sys::Uint8Array;
use std::ops::{Deref, DerefMut};
use wasm_bindgen::prelude::*;

use super::VaultsAccount;

#[wasm_bindgen]
pub struct StatementAccount {
    account: Statement,
}

impl Deref for StatementAccount {
    type Target = Statement;
    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl DerefMut for StatementAccount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

#[wasm_bindgen]
impl StatementAccount {
    #[wasm_bindgen]
    pub fn load(account_info: &Uint8Array) -> Self {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode::<Statement>(&v);
        Self { account }
    }

    pub fn reload(&mut self, account_info: &Uint8Array) {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode::<Statement>(&v);

        self.account.clone_from(&account)
    }

    #[wasm_bindgen]
    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    #[wasm_bindgen]
    pub fn refresh(&mut self, vaults: &Uint8Array) -> Result<(), JsError> {
        let vaults_acc = VaultsAccount::load(vaults);
        let vaults = &vaults_acc.arr.elements;
        Ok(self.statement.refresh(vaults)?)
    }

    #[wasm_bindgen]
    pub fn statement_len(&self) -> u8 {
        self.statement.positions.head
    }

    #[wasm_bindgen]
    pub fn owner(&self) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.owner))
    }

    #[wasm_bindgen]
    pub fn max_allowed_borrow_value(&self) -> u64 {
        self.statement.permitted_debt().get() as u64
    }
}
