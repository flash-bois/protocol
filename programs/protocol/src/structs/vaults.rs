use crate::core_lib::vault::Vault;
use anchor_lang::prelude::*;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

#[account(zero_copy)]
#[derive(Debug, SafeArray)]
pub struct Vaults {
    pub head: u8,
    pub elements: [VaultEntry; 10],
}

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
