use crate::{
    core_lib::{decimal::Quantity, Token},
    structs::{State, Statement, Vaults},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use checked_decimal_macro::Decimal;

#[derive(Accounts)]
#[instruction(vault: u8)]
pub struct SingleSwap<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,
        constraint = account_base.mint == reserve_base.mint,
        constraint = account_base.owner == signer.key(),
    )]
    account_base: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = account_quote.mint == reserve_quote.mint,
        constraint = account_quote.owner == signer.key(),
    )]
    account_quote: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = reserve_base.mint == vaults.load()?.keys.get(vault as usize).unwrap().base_token,
        constraint = reserve_base.key() == vaults.load()?.keys.get(vault as usize).unwrap().base_reserve,
        constraint = reserve_base.owner == state.key(),
    )]
    pub reserve_base: Account<'info, TokenAccount>,
    #[account(mut,
        constraint = reserve_quote.mint == vaults.load()?.keys.get(vault as usize).unwrap().quote_token,
        constraint = reserve_quote.key() == vaults.load()?.keys.get(vault as usize).unwrap().quote_reserve,
        constraint = reserve_quote.owner == state.key(),
    )]
    pub reserve_quote: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

impl SingleSwap<'_> {
    pub fn handler(
        &mut self,
        vault: u8,
        amount: u64,
        min_expected: u64,
        from_base: bool,
        by_amount_out: bool,
    ) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp as u32;
        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults
            .arr
            .get_mut(vault as usize)
            .expect("invalid vault index");

        let quantity = Quantity::new(amount);

        if by_amount_out {
            unimplemented!("swaps by amount out are not yet implemented")
        }

        let quantity_out = match from_base {
            true => vault.sell(quantity, now).expect("sell failed"), // ERROR CODE

            false => vault.buy(quantity, now).expect("buy failed"), // ERROR CODE
        };

        msg!("quantity out: {}", quantity_out);

        if quantity_out < Quantity::new(min_expected) {
            panic!("quantity out is less than min expected") // ERROR CODE
        }

        // TODO: token transfers

        Ok(())
    }
}
