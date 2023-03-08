use super::vault::VaultsAccount;
use crate::core_lib::{decimal::Utilization, errors::LibErrors, strategy::Strategy};
use checked_decimal_macro::Decimal;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct StrategyInfo {
    pub has_lend: bool,
    pub has_swap: bool,
    pub has_trade: bool,
    pub balance_base: u64,
    pub balance_quote: u64,
    pub locked_base: u64,
    pub locked_quote: u64,
    pub utilization_base: u64,
    pub utilization_quote: u64,
}

#[wasm_bindgen]
impl VaultsAccount {
    fn strategy(&self, vault: u8, strategy: u8) -> Result<&Strategy, LibErrors> {
        Ok(self
            .vault_checked(vault)?
            .strategies
            .get_strategy(strategy)?)
    }

    pub fn strategy_info(&self, vault: u8, strategy: u8) -> Result<StrategyInfo, JsError> {
        let strategy = self.strategy(vault, strategy)?;

        Ok(StrategyInfo {
            has_lend: strategy.is_lending_enabled(),
            has_swap: strategy.is_swapping_enabled(),
            has_trade: strategy.is_trading_enabled(),
            balance_base: strategy.balance().get(),
            balance_quote: strategy.balance_quote().get(),
            locked_base: strategy.locked().get(),
            locked_quote: strategy.locked_quote().get(),
            utilization_base: Utilization::get_utilization(strategy.locked(), strategy.balance())
                .get()
                .try_into()
                .map_err(|_| LibErrors::ParseError)?,
            utilization_quote: Utilization::get_utilization(
                strategy.locked_quote(),
                strategy.balance_quote(),
            )
            .get()
            .try_into()
            .map_err(|_| LibErrors::ParseError)?,
        })
    }

    #[wasm_bindgen]
    pub fn count_strategies(&self, vault: u8) -> Result<u8, JsError> {
        Ok(self.vault_checked(vault)?.strategies.head)
    }

    #[wasm_bindgen]
    pub fn does_lend(&self, vault: u8, strategy: u8) -> Result<bool, JsError> {
        Ok(self.strategy(vault, strategy)?.is_lending_enabled())
    }

    #[wasm_bindgen]
    pub fn does_swap(&self, vault: u8, strategy: u8) -> Result<bool, JsError> {
        Ok(self.strategy(vault, strategy)?.is_swapping_enabled())
    }

    #[wasm_bindgen]
    pub fn does_trade(&self, vault: u8, strategy: u8) -> Result<bool, JsError> {
        Ok(self.strategy(vault, strategy)?.is_trading_enabled())
    }

    #[wasm_bindgen]
    pub fn balance_base(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        Ok(self.strategy(vault, strategy)?.balance().get())
    }

    #[wasm_bindgen]
    pub fn balance_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        Ok(self.strategy(vault, strategy)?.balance_quote().get())
    }

    #[wasm_bindgen]
    pub fn lock_base(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        Ok(self.strategy(vault, strategy)?.locked().get())
    }

    #[wasm_bindgen]
    pub fn lock_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        Ok(self.strategy(vault, strategy)?.locked_quote().get())
    }

    #[wasm_bindgen]
    pub fn utilization_base(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        let strategy = self.strategy(vault, strategy)?;
        Ok((strategy.locked() / strategy.balance()).get())
    }

    #[wasm_bindgen]
    pub fn utilization_quote(&self, vault: u8, strategy: u8) -> Result<u64, JsError> {
        let strategy = self.strategy(vault, strategy)?;
        Ok((strategy.locked_quote() / strategy.balance_quote()).get())
    }
}
