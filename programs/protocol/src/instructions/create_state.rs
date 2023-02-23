use crate::structs::{Inner, State, Vaults};
use anchor_lang::solana_program::system_program;
use anchor_lang::{prelude::*, Discriminator};

#[derive(Accounts)]
pub struct CreateState<'info> {
    #[account(init, seeds = [b"state".as_ref()], bump, payer = admin, space = 8 + 65)]
    pub state: AccountLoader<'info, State>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(zero)]
    pub vaults: AccountLoader<'info, Vaults>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK:
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<CreateState>, bump: u8) -> Result<()> {
    let state = &mut ctx.accounts.state.load_init()?;

    msg!("Initializing state");

    **state = State {
        admin: *ctx.accounts.admin.key,
        vaults_acc: *ctx.accounts.vaults.to_account_info().key,
        bump,
    };

    drop(state);

    //msg!("{:?}", *ctx.accounts.vaults.to_account_info().data);
    let vaults = &mut ctx.accounts.vaults.load_init()?;
    // {
    //     let vaults = &mut ctx.accounts.vaults.load_init()?;
    // }

    //let val = Inner::default();
    msg!(
        "Initializing vaults {} {} ",
        std::mem::size_of::<Vaults>(),
        std::mem::size_of_val(&**vaults),
        // std::mem::size_of_val(&val),
        // std::mem::size_of_val(&Inner::default())
    );

    // **vaults = Vaults {
    //     arr: Inner::default(),
    // };

    //drop(vaults);
    Ok(())
}
