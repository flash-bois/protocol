use super::vault::VaultsAccount;
use crate::core_lib::{
    decimal::{Decimal, Quantity},
    user::UserStatement,
    Token,
};
use crate::wasm_wrapper::StatementAccount;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WithdrawAmounts {
    pub base: u64,
    pub quote: u64,
}

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn withdraw(
        &self,
        vault: u8,
        strategy: u8,
        amount: u64,
        deposit_base: bool,
        statement: &Uint8Array,
    ) -> Result<WithdrawAmounts, JsError> {
        let mut vault = self.vault_checked(vault)?.clone();
        let user_statement = &mut StatementAccount::load(statement).statement;

        let amount = Quantity::new(amount);
        let deposit_token = if deposit_base {
            Token::Base
        } else {
            Token::Quote
        };

        let (base, quote) = vault.withdraw(user_statement, deposit_token, amount, strategy)?;

        Ok(WithdrawAmounts {
            base: base.get(),
            quote: quote.get(),
        })
    }

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

        vault.refresh(current_time)?;

        Ok(vault
            .deposit(&mut temp_user_statement, deposit_token, amount, strategy)?
            .get())
    }
}
