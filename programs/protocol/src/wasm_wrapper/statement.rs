use crate::structs::Statement;
use crate::ZeroCopyDecoder;
use js_sys::Uint8Array;
use std::ops::{Deref, DerefMut};
use wasm_bindgen::prelude::*;
use checked_decimal_macro::Decimal;
use crate::wasm_wrapper::to_buffer;
use serde::{Serialize};

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

#[cfg(test)]
mod statement_test {
    use super::*;
    use crate::core_lib::{user::Position::LiquidityProvide, decimal::{Shares, Quantity}};

    unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        ::core::slice::from_raw_parts(
            (p as *const T) as *const u8,
            ::core::mem::size_of::<T>(),
        )
    }

    #[test]
    fn test_decode_zc() {
        let mut statement = Statement::default();
        statement.bump = 12;
        let position = LiquidityProvide { vault_index: 1, strategy_index: 1, shares: Shares::new(10), amount: Quantity::new(10), quote_amount: Quantity::new(10) };
        statement.statement.add_position(position).unwrap();


        let statement_in_bytes = unsafe { any_as_u8_slice(&statement) }.to_vec();

        let decoded = ZeroCopyDecoder::decode_account_info::<Statement>(&statement_in_bytes);


        let buff = to_buffer(&statement_in_bytes);
    
    

        println!("{:?} {:?}", decoded.statement.positions.get_checked(0), buff);
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
        self.bump
    }

    #[wasm_bindgen]
    pub fn refresh(&mut self, vaults: &VaultsAccount) -> Result<(), JsError>{
        let vaults = &vaults.arr.elements;
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