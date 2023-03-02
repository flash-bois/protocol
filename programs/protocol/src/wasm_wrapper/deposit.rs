use super::vault::VaultsAccount;
use crate::core_lib::{
    decimal::{Decimal, Quantity},
    Token,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DepositAmounts {
    pub base: u64,
    pub quote: u64,
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
    ) -> Result<DepositAmounts, JsError> {
        let vault = self.vault_checked(vault)?;

        let amount = Quantity::new(amount);
        let base_oracle = vault.oracle()?;
        let quote_oracle = vault.quote_oracle()?;

        let deposit_token = if deposit_base {
            Token::Base
        } else {
            Token::Quote
        };

        let strategy = vault.strategy(strategy)?;
        let opposite_quantity = vault.opposite_quantity(amount, deposit_token, strategy);

        let (base_quantity, quote_quantity) = match deposit_token {
            Token::Base => {
                let (base, quote) = vault.get_opposite_quantity(
                    quote_oracle,
                    opposite_quantity,
                    base_oracle,
                    amount,
                );

                (base, quote)
            }
            Token::Quote => {
                let (quote, base) = vault.get_opposite_quantity(
                    base_oracle,
                    opposite_quantity,
                    quote_oracle,
                    amount,
                );

                (base, quote)
            }
        };

        Ok(DepositAmounts {
            base: base_quantity.get(),
            quote: quote_quantity.get(),
        })
    }
}
