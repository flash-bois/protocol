use crate::core_lib::{
    decimal::Quantity,
    errors::LibErrors,
    services::{swapping::Swap, ServiceType, ServiceUpdate, Services},
    structs::Oracle,
};

use super::Vault;

impl Vault {
    fn swap_and_oracles(&mut self) -> Result<(&mut Swap, &mut Oracle, &mut Oracle), LibErrors> {
        let Self {
            services: Services { swap, .. },
            oracle,
            quote_oracle,
            ..
        } = self;

        let swap = swap.as_mut().ok_or(LibErrors::SwapServiceNone)?;
        let oracle = oracle.as_mut().ok_or(LibErrors::OracleMissing)?;
        let quote_oracle = quote_oracle.as_mut().ok_or(LibErrors::QuoteOracleMissing)?;

        Ok((swap, oracle, quote_oracle))
    }

    pub fn sell(&mut self, quantity: Quantity) -> Result<Quantity, LibErrors> {
        let (swap, oracle, quote_oracle) = self.swap_and_oracles()?;

        let quote_quantity = swap.sell(quantity, oracle, quote_oracle)?;
        let total_available = swap.available();
        self.exchange_to_quote(
            quantity,
            quote_quantity,
            total_available.base,
            ServiceType::Swap,
        )?;

        Ok(quote_quantity)
    }

    pub fn buy(&mut self, quantity: Quantity) -> Result<Quantity, LibErrors> {
        let (swap, oracle, quote_oracle) = self.swap_and_oracles()?;

        let base_quantity = swap.buy(quantity, oracle, quote_oracle)?;
        let total_available = swap.available();
        self.exchange_to_base(
            base_quantity,
            quantity,
            total_available.quote,
            ServiceType::Swap,
        )?;

        Ok(base_quantity)
    }
}

#[cfg(test)]
mod tests {
    use crate::core_lib::decimal::Fraction;
    use checked_decimal_macro::Factories;

    use super::*;

    #[test]
    fn test_swap() -> Result<(), LibErrors> {
        let mut vault = Vault::new_vault_for_tests()?;
        vault
            .swap_service()?
            .fee_curve_sell()
            .add_constant_fee(Fraction::from_scale(3, 3), Fraction::from_scale(1, 1));

        // vault.deposit(Token::Quote, Quantity::new(10_000000), 0, 0)?;
        // TODO: finish this test after deposits are finished

        Ok(())
    }
}
