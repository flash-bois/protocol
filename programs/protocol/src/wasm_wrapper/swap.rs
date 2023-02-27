use crate::core_lib::decimal::{Decimal, Price, Quantity};
use crate::core_lib::errors::LibErrors;
use crate::core_lib::services::ServiceUpdate;
use crate::structs::VaultsAccount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn swap(
        &self,
        vault: u8,
        amount: u64,
        min_expected: u64,
        from_base: bool,
        by_amount_out: bool,
        now: u32,
    ) -> Result<i64, JsValue> {
        let mut vault = self
            .account
            .arr
            .get(vault as usize)
            .expect("invalid vault index")
            .clone();

        let quantity = Quantity::new(amount);

        if by_amount_out {
            unimplemented!("swaps by amount out are not yet implemented")
        }

        let quantity_out = match from_base {
            true => vault.sell(quantity).map_err(|err| JsValue::from(err))?,
            false => vault.buy(quantity).map_err(|err| JsValue::from(err))?,
        };

        if quantity_out < Quantity::new(min_expected) {
            return Err(JsValue::from(LibErrors::NoMinAmountOut));
        }

        Ok(quantity_out.get() as i64)
    }

    #[wasm_bindgen]
    pub fn liquidity(&self, vault: u8, base: bool) -> u64 {
        let vault = self
            .account
            .arr
            .get(vault as usize)
            .expect("invalid vault index");

        let available = vault.swap_service_not_mut().unwrap().available();
        match base {
            true => available.base,
            false => available.quote,
        }
        .get()
    }
}
