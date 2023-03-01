use super::vault::VaultsAccount;
use crate::core_lib::{
    decimal::{Decimal, Quantity},
    services::ServiceUpdate,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl VaultsAccount {
    #[wasm_bindgen]
    pub fn swap(
        &mut self,
        vault: u8,
        amount: u64,
        from_base: bool,
        by_amount_out: bool,
        now: u32,
    ) -> Result<i64, JsError> {
        let mut vault = self.vault_checked_mut(vault)?.clone();

        let quantity = Quantity::new(amount);

        if by_amount_out {
            unimplemented!("swaps by amount out are not yet implemented")
        }

        let quantity_out = match from_base {
            true => vault.sell(quantity)?,
            false => vault.buy(quantity)?,
        };

        Ok(quantity_out.get() as i64)
    }

    #[wasm_bindgen]
    pub fn liquidity(&self, vault: u8, base: bool) -> Result<u64, JsError> {
        let vault = self.vault_checked(vault)?;

        let available = vault.swap_service_not_mut()?.available();

        Ok(match base {
            true => available.base,
            false => available.quote,
        }
        .get())
    }
}
