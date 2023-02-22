mod core_lib;
mod errors;

#[cfg(feature = "anchor")]
mod instructions;
#[cfg(feature = "anchor")]
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

    pub fn create_state(ctx: Context<CreateState>, nonce: u8) -> anchor_lang::Result<()> {
        msg!("Initializing here");
        ctx.accounts
            .handler(nonce, *ctx.bumps.get("state").unwrap())
    }
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// #[cfg_attr(feature = "wasm", wasm_bindgen)]
// #[wasm_bindgen]
pub fn nothing() {}
