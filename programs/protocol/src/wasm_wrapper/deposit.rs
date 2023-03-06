use super::vault::VaultsAccount;
use crate::core_lib::{
    decimal::{Decimal, Quantity},
    user::UserStatement,
    Token,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn deposit(
        &self,
        vault: u8,
        strategy: u8,
        amount: u64,
        deposit_base: bool,
        current_time: u32,
    ) -> Result<u64, JsError> {
        let mut vault = self.vault_checked(vault)?.clone();
        let mut temp_user_statement = UserStatement::default();

        let amount = Quantity::new(amount);
        let deposit_token = if deposit_base {
            Token::Base
        } else {
            Token::Quote
        };

        let out = vault.deposit(
            &mut temp_user_statement,
            deposit_token,
            amount,
            strategy,
            current_time,
        )?;

        Ok(out.get())
    }
}
