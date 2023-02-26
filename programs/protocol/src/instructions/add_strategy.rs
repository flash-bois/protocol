use crate::{
    core_lib::{
        decimal::{Factories, Fraction, Price, Quantity, Utilization},
        structs::FeeCurve,
    },
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use checked_decimal_macro::{BetweenDecimals, Decimal};

#[derive(Accounts)]
pub struct AddStrategy<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,
}

impl AddStrategy<'_> {
    pub fn handler(&mut self, index: u8, lending: bool, swapping: bool) -> anchor_lang::Result<()> {
        msg!("DotWave: Adding Strategy");
        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.arr.get_mut(index as usize).expect("Vault not found"); // ERROR CODE

        vault
            .add_strategy(lending, swapping, false)
            .expect("couldn't add strategy");

        Ok(())
    }
}
