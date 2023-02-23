use anchor_lang::prelude::*;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct State {
    pub admin: Pubkey,
    pub vaults_acc: Pubkey,
    pub bump: u8,
}
