use crate::{
    core_lib::{decimal::Quantity, structs::Side},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct OpenPosition<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, seeds = [b"statement".as_ref(), signer.key.as_ref()], bump=statement.load()?.bump, constraint = statement.load()?.owner == signer.key())]
    pub statement: AccountLoader<'info, Statement>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

impl<'info> OpenPosition<'info> {
    pub fn handler(
        ctx: Context<OpenPosition>,
        vault: u8,
        amount: u64,
        long: bool,
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Open position");
        let current_timestamp = Clock::get()?.unix_timestamp;
        let user_statement = &mut ctx.accounts.statement.load_mut()?.statement;
        let vaults = &mut ctx.accounts.vaults.load_mut()?;

        let vaults_indexes = user_statement.get_vaults_indexes(&vault);
        vaults.refresh(&vaults_indexes, ctx.remaining_accounts, current_timestamp)?;

        user_statement.refresh(&vaults.arr.elements)?;

        let vault = vaults.vault_checked_mut(vault)?;
        let quantity = Quantity::new(amount);
        let side = if long { Side::Long } else { Side::Short };
        vault.open_position(user_statement, quantity, side)?;

        Ok(())
    }
}
