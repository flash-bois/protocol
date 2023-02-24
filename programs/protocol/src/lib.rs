mod core_lib;

#[cfg(feature = "anchor")]
mod errors;

#[cfg(feature = "anchor")]
mod instructions;
mod structs;

#[cfg(feature = "anchor")]
use instructions::*;

#[cfg(feature = "anchor")]
declare_id!("9DvKMoN2Wx1jFNszJU9aGDSsvBNJ5A3UfNp1Mvv9CVDi");

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg(feature = "anchor")]
#[program]
pub mod protocol {
    use super::*;

    pub fn create_state(ctx: Context<CreateState>) -> anchor_lang::Result<()> {
        instructions::create_state::handler(ctx)
    }

    pub fn create_statement(ctx: Context<CreateStatement>) -> anchor_lang::Result<()> {
        instructions::create_statement::handler(ctx)
    }
}

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
pub use decoder::ZeroCopyDecoder;
