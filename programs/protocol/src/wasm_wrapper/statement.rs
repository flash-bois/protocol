use crate::{
    core_lib::{
        decimal::{BalanceChange, Quantity, Shares, Value},
        structs::{Receipt, Side},
        user::{Position, ValueChange},
    },
    structs::Statement,
    wasm_wrapper::to_buffer,
    ZeroCopyDecoder,
};
use checked_decimal_macro::{num_traits::ToPrimitive, Decimal};
use js_sys::{Array, Uint8Array};
use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};
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
pub struct BorrowPositionInfo {
    pub vault_id: u8,
    pub borrowed_quantity: u64,
    pub owed_quantity: u64,
}

#[wasm_bindgen]
pub struct LpPositionInfo {
    pub vault_id: u8,
    pub strategy_id: u8,
    pub position_value: u64,
    pub deposited_base_quantity: u64,
    pub deposited_quote_quantity: u64,
    pub earned_base_quantity: u64,
    pub earned_quote_quantity: u64,
    pub max_withdraw_quote: u64,
    pub max_withdraw_base: u64,
}

#[wasm_bindgen]
pub struct TradingPositionInfo {
    pub vault_id: u8,
    pub long: bool,
    pub size: u64,
    pub size_value: u64,
    pub locked: u64,
    pub open_price: u64,
    pub open_value: u64,
    pub pnl: i64,
    pub pnl_value: i64,
    pub fees: u64,
    pub fees_value: u64,
}

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn get_borrow_position_info(
        &mut self,
        vault_index: u8,
        statement: &Uint8Array,
        current_time: u32,
    ) -> Result<Option<BorrowPositionInfo>, JsError> {
        let vault = self.vault_checked_mut(vault_index)?;
        vault.refresh(current_time)?;

        let statement_account = StatementAccount::load(statement);

        // Search by vault index (PartialEq depended implementation)
        let position_search = Position::Borrow {
            vault_index,
            shares: Shares::new(0),
            amount: Quantity::new(0),
        };
        let found_position = match statement_account.statement.search(&position_search) {
            Ok(position) => position,
            Err(_) => return Ok(None),
        };

        let owed_amount = found_position.get_owed_single(found_position.shares(), vault)?;

        Ok(Some(BorrowPositionInfo {
            vault_id: vault_index,
            borrowed_quantity: found_position.amount().get(),
            owed_quantity: owed_amount.get(),
        }))
    }

    #[wasm_bindgen]
    pub fn get_lp_position_info(
        &mut self,
        vault_index: u8,
        strategy_index: u8,
        statement: &Uint8Array,
        current_time: u32,
    ) -> Result<Option<LpPositionInfo>, JsError> {
        let vault = self.vault_checked_mut(vault_index)?;
        vault.refresh(current_time)?;

        let statement_account = StatementAccount::load(statement);

        // Search by vault index (PartialEq depended implementation)
        let position_search = Position::LiquidityProvide {
            vault_index,
            strategy_index,
            shares: Shares::new(0),
            amount: Quantity::new(0),
            quote_amount: Quantity::new(0),
        };

        let found_position = match statement_account.statement.search(&position_search) {
            Ok(position) => position,
            Err(_) => return Ok(None),
        };

        let strategy = vault.strategy(strategy_index)?;
        let available = strategy.available();
        let balance = strategy.balance();
        let balance_quote = strategy.balance_quote();
        let available_quote = strategy.available_quote();
        let total_shares = strategy.total_shares();
        let (base_quantity, quote_quantity) = strategy.get_earned_double(found_position.shares());
        let oracle = vault.oracle()?;
        let quote_oracle = vault.quote_oracle()?;
        let value =
            oracle.calculate_value(base_quantity) + quote_oracle.calculate_value(quote_quantity);

        let (max_withdraw_base, max_withdraw_quote) = match (
            base_quantity.cmp(&strategy.available()),
            quote_quantity.cmp(&strategy.available_quote()),
        ) {
            (Ordering::Less | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
                (base_quantity, quote_quantity)
            }
            (Ordering::Less | Ordering::Equal, Ordering::Greater) => {
                let max_quote = std::cmp::min(quote_quantity, available_quote);
                let shares = total_shares.get_change_down(max_quote, balance_quote);
                let max_base = total_shares.calculate_earned(shares, balance);

                (max_base, max_quote)
            }
            (Ordering::Greater, Ordering::Less | Ordering::Equal) => {
                let max_base = std::cmp::min(base_quantity, available);
                let shares = total_shares.get_change_down(max_base, balance);
                let max_quote = total_shares.calculate_earned(shares, balance_quote);

                (max_base, max_quote)
            }
            (Ordering::Greater, Ordering::Greater) => {
                let shares_base = total_shares.get_change_down(available, balance);
                let shares_quote = total_shares.get_change_down(available_quote, balance_quote);
                let shares = std::cmp::min(shares_quote, shares_base);
                let max_base = total_shares.calculate_earned(shares, balance);
                let max_quote = total_shares.calculate_earned(shares, balance_quote);

                (max_base, max_quote)
            }
        };

        Ok(Some(LpPositionInfo {
            vault_id: vault_index,
            strategy_id: strategy_index,
            position_value: value.get() as u64,
            earned_base_quantity: base_quantity.get(),
            earned_quote_quantity: quote_quantity.get(),
            deposited_base_quantity: found_position.amount().get(),
            deposited_quote_quantity: found_position.quote_amount().get(),
            max_withdraw_quote: max_withdraw_quote.get(),
            max_withdraw_base: max_withdraw_base.get(),
        }))
    }

    pub fn get_trading_position_info(
        &mut self,
        vault_index: u8,
        statement: &Uint8Array,
        current_time: u32,
    ) -> Result<Option<TradingPositionInfo>, JsError> {
        let vault = self.vault_checked_mut(vault_index)?;
        let user_statement = StatementAccount::load(statement);

        vault.refresh(current_time)?;

        let (trade, oracle, quote_oracle) = vault.trade_mut_and_oracles()?;

        let position_search = Position::Trading {
            vault_index,
            receipt: Receipt::default(),
        };

        let found_position = match user_statement.statement.search(&position_search) {
            Ok(position) => position,
            Err(_) => return Ok(None),
        };

        let receipt = found_position.receipt_not_mut();

        let (pnl, pnl_value) = match trade.calculate_position(receipt, oracle, quote_oracle, false)
        {
            (BalanceChange::Profit(profit), ValueChange::Profitable(value)) => (
                profit.get().to_i64().unwrap(),
                value.get().to_i64().unwrap(),
            ),
            (BalanceChange::Loss(loss), ValueChange::Loss(value)) => (
                -loss.get().to_i64().unwrap(),
                -value.get().to_i64().unwrap(),
            ),
            _ => unreachable!("pnl cannot be none"),
        };

        let (long, fees, fees_value) = match receipt.side {
            Side::Long => {
                let fees = trade.long_fees(receipt).quantity();
                let value = oracle.calculate_value(fees);

                (true, fees.get(), value.get() as u64)
            }
            Side::Short => {
                let fees = trade.short_fees(receipt, oracle, quote_oracle).quantity();
                let value = quote_oracle.calculate_value(fees);

                (false, fees.get(), value.get() as u64)
            }
        };

        let size_value = oracle.calculate_value(receipt.size).get() as u64;

        Ok(Some(TradingPositionInfo {
            vault_id: vault_index,
            long,
            pnl,
            pnl_value,
            fees,
            fees_value,
            size: receipt.size.get(),
            size_value,
            locked: receipt.locked.get(),
            open_price: receipt.open_price.get(),
            open_value: receipt.open_value.get() as u64,
        }))
    }

    #[wasm_bindgen]
    pub fn max_borrow_for(&self, id: u8, value: u64) -> Result<u64, JsError> {
        let vault = self.vault_checked(id)?;
        let value = Value::new(value as u128);

        Ok(vault.oracle()?.calculate_quantity(value).get())
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

        self.account = account
    }

    pub fn buffer(&self) -> Uint8Array {
        to_buffer(ZeroCopyDecoder::encode(&self.account))
    }

    #[wasm_bindgen]
    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    #[wasm_bindgen]
    pub fn vaults_to_refresh(&self, current: u8) -> Result<Array, JsError> {
        Ok(self
            .statement
            .get_vaults_indexes(&current)
            .iter()
            .map(|x| JsValue::from(*x))
            .collect::<Array>())
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
