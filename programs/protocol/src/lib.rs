mod core_lib;

#[cfg(feature = "anchor")]
mod instructions;
mod structs;

#[cfg(feature = "anchor")]
pub use instructions::*;

#[cfg(feature = "anchor")]
declare_id!("9DvKMoN2Wx1jFNszJU9aGDSsvBNJ5A3UfNp1Mvv9CVDi");

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(feature = "anchor")]
#[program]
pub mod protocol {
    use super::*;

    pub fn create_state(ctx: Context<CreateState>) -> Result<()> {
        instructions::create_state::handler(ctx)
    }

    pub fn create_statement(ctx: Context<CreateStatement>) -> Result<()> {
        instructions::create_statement::handler(ctx)
    }

    pub fn init_vault(ctx: Context<InitVault>) -> Result<()> {
        ctx.accounts.handler()
    }

    pub fn enable_oracle(
        ctx: Context<EnableOracle>,
        index: u8,
        decimals: u8,
        base: bool,
        skip_init: bool,
    ) -> Result<()> {
        ctx.accounts.handler(index, decimals, base, skip_init)
    }

    pub fn force_override_oracle(
        ctx: Context<Admin>,
        index: u8,
        base: bool,
        price: u32,
        conf: u32,
        exp: i8,
        time: Option<u32>,
    ) -> Result<()> {
        ctx.accounts
            .force_override_oracle(index, base, price, conf, exp, time)
    }

    pub fn enable_lending(
        ctx: Context<Admin>,
        index: u8,
        max_utilization: u32,
        max_total_borrow: u64,
    ) -> Result<()> {
        ctx.accounts
            .enable_lending(index, max_utilization, max_total_borrow)?;
        Ok(())
    }

    pub fn enable_swapping(
        ctx: Context<Admin>,
        index: u8,
        kept_fee: u32,
        max_total_sold: u64,
    ) -> Result<()> {
        ctx.accounts
            .enable_swapping(index, kept_fee, max_total_sold)
    }

    pub fn add_strategy(
        ctx: Context<AddStrategy>,
        index: u8,
        lending: bool,
        swapping: bool,
    ) -> Result<()> {
        ctx.accounts.handler(index, lending, swapping)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        vault: u8,
        strategy: u8,
        quantity: u64,
        base: bool,
    ) -> Result<()> {
        ctx.accounts.handler(vault, strategy, quantity, base)
    }

    pub fn single_swap(
        ctx: Context<SingleSwap>,
        vault: u8,
        amount: u64,
        min_expected: u64,
        from_base: bool,
        by_amount_out: bool,
    ) -> Result<()> {
        ctx.accounts
            .handler(vault, amount, min_expected, from_base, by_amount_out)
    }

    pub fn modify_fee_curve(
        ctx: Context<Admin>,
        vault: u8,
        service: u8,
        base: bool,
        bound: u64,
        a: u64,
        b: u64,
        c: u64,
    ) -> Result<()> {
        ctx.accounts
            .modify_fee_curve(vault, service, base, bound, a, b, c)
    }
}

#[cfg(feature = "wasm")]
pub mod wasm_wrapper;

#[cfg(feature = "wasm")]
mod decoder {
    use bytemuck::{Pod, Zeroable};
    pub struct ZeroCopyDecoder;

    impl ZeroCopyDecoder {
        pub(crate) fn decode_account_info<'a, R>(d: &'a Vec<u8>) -> &'a R
        where
            R: Pod + Zeroable,
        {
            bytemuck::from_bytes::<R>(&d[..std::mem::size_of::<R>()])
        }

        pub(crate) fn mut_decode_account_info<'a, R>(d: &'a mut Vec<u8>) -> &'a R
        where
            R: Pod + Zeroable,
        {
            bytemuck::from_bytes_mut::<R>(&mut d[..std::mem::size_of::<R>()])
        }
    }
}
#[cfg(feature = "wasm")]
pub use decoder::*;
