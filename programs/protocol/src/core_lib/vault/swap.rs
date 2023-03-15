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
        let oracle = oracle.as_mut().ok_or(LibErrors::OracleNone)?;
        let quote_oracle = quote_oracle.as_mut().ok_or(LibErrors::QuoteOracleNone)?;

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
            quantity,
            base_quantity,
            total_available.quote,
            ServiceType::Swap,
        )?;

        Ok(base_quantity)
    }
}

#[cfg(test)]
mod tests {
    use crate::core_lib::{
        decimal::{Balances, Fraction, Shares},
        strategy::Strategy,
        user::UserStatement,
        Token,
    };
    use checked_decimal_macro::{Decimal, Factories};

    use super::*;

    #[test]
    fn test_swap() -> Result<(), LibErrors> {
        let mut vault = Vault::new_vault_for_tests()?;
        vault
            .swap_service()?
            .fee_curve_sell()
            .add_constant_fee(Fraction::from_scale(1, 3), Fraction::from_scale(1, 2))
            .add_linear_fee(
                Fraction::from_scale(5, 3),
                Fraction::new(0),
                Fraction::from_scale(7, 1),
            );
        vault
            .swap_service()?
            .fee_curve_buy()
            .add_constant_fee(Fraction::from_scale(3, 3), Fraction::from_scale(1, 1))
            .add_linear_fee(
                Fraction::from_scale(5, 3),
                Fraction::new(0),
                Fraction::from_scale(7, 1),
            );

        let selling_fee = *vault.swap_service()?.fee_curve_sell();
        let buying_fee = *vault.swap_service()?.fee_curve_buy();

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                balances: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        assert_eq!(
            *vault.strategy(0)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: None,
                available: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(0),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(1)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                available: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(0),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(2)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: None,
                traded: None,
                available: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(0),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );

        // Deposit to unrelated strategy

        vault.deposit(
            &mut UserStatement::default(),
            Token::Base,
            Quantity::new(1000),
            2,
        )?;

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                balances: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        vault.deposit(
            &mut UserStatement::default(),
            Token::Base,
            Quantity::new(1000),
            1,
        )?;

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                balances: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        vault.deposit(
            &mut UserStatement::default(),
            Token::Base,
            Quantity::new(1000),
            0,
        )?;

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(2000),
                    quote: Quantity::new(4000)
                },
                balances: Balances {
                    base: Quantity::new(2000),
                    quote: Quantity::new(4000)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        assert_eq!(
            *vault.strategy(0)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: None,
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(1)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(2)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: None,
                traded: None,
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );

        // Swapping
        let quote = vault.sell(Quantity::new(100))?;
        assert_eq!(quote, Quantity::new(200) - Quantity::new(1));

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(1900),
                    quote: Quantity::new(4199)
                },
                balances: Balances {
                    base: Quantity::new(1900),
                    quote: Quantity::new(4199)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(1)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        assert_eq!(
            *vault.strategy(0)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: None,
                available: Balances {
                    base: Quantity::new(950),
                    quote: Quantity::new(2099)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(1)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                available: Balances {
                    base: Quantity::new(950),
                    quote: Quantity::new(2100)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(2)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: None,
                traded: None,
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );

        let base_quantity = vault.buy(Quantity::new(400))?;
        assert_eq!(base_quantity, Quantity::new(200) - Quantity::new(1));

        assert_eq!(
            *vault.swap_service()?,
            Swap {
                available: Balances {
                    base: Quantity::new(2099),
                    quote: Quantity::new(3799)
                },
                balances: Balances {
                    base: Quantity::new(2099),
                    quote: Quantity::new(3799)
                },
                total_earned_fee: Balances {
                    base: Quantity::new(1),
                    quote: Quantity::new(1)
                },
                total_paid_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_kept_fee: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                selling_fee,
                buying_fee,
                kept_fee: Fraction::from_scale(1, 1),
            }
        );

        assert_eq!(
            *vault.strategy(0)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: None,
                available: Balances {
                    base: Quantity::new(1049),
                    quote: Quantity::new(1900)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(1)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                traded: Some(Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                }),
                available: Balances {
                    base: Quantity::new(1050),
                    quote: Quantity::new(1899)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );
        assert_eq!(
            *vault.strategy(2)?,
            Strategy {
                lent: Some(Quantity::new(0)),
                sold: None,
                traded: None,
                available: Balances {
                    base: Quantity::new(1000),
                    quote: Quantity::new(2000)
                },
                locked: Balances {
                    base: Quantity::new(0),
                    quote: Quantity::new(0)
                },
                total_shares: Shares::new(1000),
                collateral_ratio: Fraction::from_integer(1),
                accrued_fee: Quantity::new(0),
                liquidation_threshold: Fraction::from_integer(1),
            }
        );

        Ok(())
    }
}
