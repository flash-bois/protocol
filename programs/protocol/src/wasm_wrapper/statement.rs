use crate::structs::Statement;
use crate::ZeroCopyDecoder;
use js_sys::Uint8Array;
use std::ops::{Deref, DerefMut};
use wasm_bindgen::prelude::*;

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
        let account = *ZeroCopyDecoder::decode_account_info::<Statement>(&v);
        Self { account }
    }

    #[wasm_bindgen]
    pub fn get_bump(&self) -> u8 {
        self.account.bump
    }
}
