use crate::{
    core_lib::decimal::Fraction,
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
pub struct AddStrategy<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,
}

impl AddStrategy<'_> {
    pub fn handler(
        &mut self,
        index: u8,
        lending: bool,
        swapping: bool,
        collateral_ratio: u64,
        liquidation_threshold: u64,
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Adding Strategy");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(index)?;

        msg!(
            "here {} {}",
            vault.lend_service().is_ok(),
            vault.swap_service().is_ok()
        );

        vault.add_strategy(
            lending,
            swapping,
            false,
            Fraction::new(collateral_ratio),
            Fraction::new(liquidation_threshold),
        )?;

        Ok(())
    }
}
