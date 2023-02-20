mod errors;
mod instructions;
mod structs;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("9DvKMoN2Wx1jFNszJU9aGDSsvBNJ5A3UfNp1Mvv9CVDi");

#[program]
pub mod protocol {
    use super::*;

    pub fn create_state(ctx: Context<CreateState>, nonce: u8) -> anchor_lang::Result<()> {
        instructions::create_state::handler(ctx, nonce)
    }
}
