use std::ops::{Deref, DerefMut};

use crate::structs::State;
use crate::ZeroCopyDecoder;
use crate::wasm_wrapper::to_buffer;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct StateAccount {
    account: State,
}

impl Deref for StateAccount {
    type Target = State;
    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl DerefMut for StateAccount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

#[wasm_bindgen]
impl StateAccount {
    #[wasm_bindgen]
    pub fn load(account_info: &Uint8Array) -> Self {
        let v = account_info.to_vec();
        let account = *ZeroCopyDecoder::decode_account_info::<State>(&v);
        Self { account }
    }

    #[wasm_bindgen]
    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    #[wasm_bindgen]
    pub fn get_vaults_account(&self) -> Result<Uint8Array, JsValue> {
        Ok(to_buffer(&self.account.vaults_acc))
    }
}
