mod core_lib;

#[cfg(feature = "anchor")]
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

    pub fn create_state(ctx: Context<CreateState>) -> anchor_lang::Result<()> {
        instructions::create_state::handler(ctx)
    }

    pub fn create_statement(ctx: Context<CreateStatement>) -> anchor_lang::Result<()> {
        instructions::create_statement::handler(ctx)
    }
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// #[cfg_attr(feature = "wasm", wasm_bindgen)]
// #[wasm_bindgen]
pub fn nothing() {}
