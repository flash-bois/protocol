use super::VaultEntry;
use anchor_lang::prelude::*;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

#[repr(packed)]
#[zero_copy]
#[derive(Debug, SafeArray)]
pub struct Vaults {
    head: u8,
    elements: [VaultEntry; 10],
}

#[repr(packed)]
#[account(zero_copy)]
#[derive(Debug, Default)]
pub struct State {
    pub owner: Pubkey,
    pub admin: Pubkey,
    pub nonce: u8,
    pub bump: u8,
    pub vaults: Vaults,
}
