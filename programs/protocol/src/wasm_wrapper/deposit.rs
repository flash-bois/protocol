use core::panic;

use super::vault::VaultsAccount;
use crate::core_lib::{
    decimal::{Decimal, Quantity},
    Token, user::UserStatement,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AmountWithBool {
    pub base: bool,
    pub val :u64
}

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn deposit(
        &mut self,
        vault: u8,
        strategy: u8,
        amount: u64,
        deposit_base: bool,
        current_time: u32
    ) -> Result<AmountWithBool, JsError> {
        let vault = self.vault_checked_mut(vault)?;
        let mut not_used_user_statement = UserStatement::default();

        let amount = Quantity::new(amount);
        let deposit_token = if deposit_base {
            Token::Base
        } else {
            Token::Quote
        };

    
        let out = vault.deposit(&mut not_used_user_statement, deposit_token, amount, strategy, current_time)?;

    
        
        Ok(AmountWithBool { val: out.get(), base: deposit_base } )
    }
}
