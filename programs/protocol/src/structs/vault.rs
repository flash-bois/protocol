use anchor_lang::prelude::*;
use core_lib::vault::Vault;

#[repr(packed)]
#[zero_copy]
#[derive(Debug, Default, PartialEq)]
pub struct VaultEntry {
    pub data: Vault,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_reserve: Pubkey,
    pub quote_reserve: Pubkey,
    pub bump: u8,
}
