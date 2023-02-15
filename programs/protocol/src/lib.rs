mod errors;
mod utils;

use anchor_lang::prelude::*;
pub use checked_decimal_macro::*;

declare_id!("9DvKMoN2Wx1jFNszJU9aGDSsvBNJ5A3UfNp1Mvv9CVDi");

#[program]
pub mod protocol {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>, bump: u8) -> Result<()> {
        let mut vault = ctx.accounts.vault.load_init()?;

        *vault = Vault {
            bump,
            ..Default::default()
        };

        Ok(())
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        msg!("CLA");

        utils::close(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        )
        .unwrap();

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateVault<'info> {
    /// CHECK:
    #[account(
        init,
        seeds = [b"testv1"],
        bump,
        payer = authority,
        space = 8 + 200
    )]
    vault: AccountLoader<'info, Vault>,
    /// CHECK:
    #[account(mut)]
    authority: Signer<'info>,
    /// CHECK:
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut,
        seeds = [b"testv1"],
        bump = vault.load()?.bump
    )]
    /// CHECK:
    pub vault: AccountLoader<'info, Vault>,
    /// CHECK:
    #[account(mut)]
    authority: Signer<'info>,
    /// CHECK:
    pub system_program: Program<'info, System>,
}

#[account(zero_copy)]
#[derive(Default)]
#[repr(packed)]
pub struct Vault {
    pub id: u8,
    pub oracle: Option<Oracle>,
    pub bump: u8,
}

#[derive(Default, PartialEq, Eq)]
#[zero_copy]
pub struct Oracle {
    /// The price of the asset.
    price: Price,
    /// The confidence of the price. It is a range around the price.
    confidence: Price,
    /// The time of the last update.
    last_update: Time,
    /// The maximum time interval between updates.
    max_update_interval: Time,
    // If true, the oracle will force use the spread instead of the spot price.
    use_spread: bool,
    // The number of decimals of the asset.
    decimals: DecimalPlaces,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DecimalPlaces {
    #[default]
    Six = 6,
    Nine = 9,
}

pub type Time = u32;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
#[decimal(9)]
#[zero_copy]
pub struct Price {
    pub v: u64,
}
