mod core_lib;
mod errors;

#[cfg(feature = "anchor")]
mod instructions;
#[cfg(feature = "anchor")]
pub mod structs;
#[cfg(feature = "anchor")]
use instructions::*;

// #[cfg(feature = "anchor")]
// pub mod anchor_program {

#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

use core_lib::vault::*;

declare_id!("9DvKMoN2Wx1jFNszJU9aGDSsvBNJ5A3UfNp1Mvv9CVDi");

#[program]
pub mod protocol {
    use super::*;

    pub fn create_state(ctx: Context<CreateState>, nonce: u8) -> anchor_lang::Result<()> {
        instructions::create_state::handler(ctx, nonce)
    }
}

// #[cfg(feature = "anchor")]
// pub use anchor_program::*;
