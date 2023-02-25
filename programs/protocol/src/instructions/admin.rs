use crate::{
    core_lib::decimal::{Factories, Price},
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,
}

impl Admin<'_> {
    pub fn force_override_oracle(
        &mut self,
        index: u8,
        base: bool,
        price: u32,
        conf: u32,
        exp: i8,
        time: Option<u32>, // can be overridden as well
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Force override oracle");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.arr.get_mut(index as usize).expect("Vault not found"); // ERROR CODE
        let oracle = match base {
            true => vault.oracle_mut(),
            false => vault.quote_oracle_mut(),
        }
        .expect("Oracle not found"); // ERROR CODE

        let time = time.unwrap_or(Clock::get()?.unix_timestamp.try_into().unwrap());
        let (price, confidence) = if exp < 0 {
            (
                Price::from_scale(price, exp.abs().try_into().unwrap()),
                Price::from_scale(conf, exp.abs().try_into().unwrap()),
            )
        } else {
            (
                Price::from_integer(price) / Price::from_scale(1, exp.try_into().unwrap()),
                Price::from_integer(conf) / Price::from_scale(1, exp.try_into().unwrap()),
            )
        };

        oracle
            .update(price, confidence, time)
            .expect("Could not update oracle"); // ERROR CODE

        Ok(())
    }
}
