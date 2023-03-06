use crate::{
    core_lib::{
        decimal::{Quantity, Shares, Value},
        user::Position,
    },
    structs::Statement,
    wasm_wrapper::to_buffer,
    ZeroCopyDecoder,
};
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
pub struct BorrowPosition {
    pub vault_id: u8,
    pub owed: u64,
}

#[wasm_bindgen]
impl VaultsAccount {
    pub fn max_borrow_for(&self, id: u8, value: u64) -> Result<u64, JsError> {
        let vault = self.vault_checked(id)?;
        let value = Value::new(value as u128);

        Ok(vault.oracle()?.calculate_quantity(value).get())
    }

    #[wasm_bindgen]
    pub fn get_borrow_position(
        &self,
        vault_index: u8,
        statement: &Uint8Array,
    ) -> Result<BorrowPosition, JsError> {
        let vault = self.vault_checked(vault_index)?;
        let statement_account = StatementAccount::load(statement);

        // Search by vault index (PartialEq depended implementation)
        let position_search = Position::Borrow {
            vault_index,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };

        let found_position = statement_account.statement.search(&position_search)?;
        let owed_amount = found_position.get_owed(found_position.shares(), vault)?;

        Ok(BorrowPosition {
            vault_id: vault_index,
            owed: owed_amount.get(),
        })
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

    pub fn buffer(&self) -> Uint8Array {
        to_buffer(ZeroCopyDecoder::encode(&self.account))
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
    pub fn positions_len(&self) -> u8 {
        self.statement.positions.head
    }

    #[wasm_bindgen]
    pub fn owner(&self) -> Result<Uint8Array, JsError> {
        Ok(to_buffer(&self.owner))
    }

    #[wasm_bindgen]
    pub fn remaining_permitted_debt(&self) -> u64 {
        self.statement.permitted_debt().get() as u64
    }
}
