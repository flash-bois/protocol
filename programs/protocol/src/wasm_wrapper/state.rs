use std::ops::{Deref, DerefMut};

use crate::structs::State;
use crate::ZeroCopyDecoder;
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
}
