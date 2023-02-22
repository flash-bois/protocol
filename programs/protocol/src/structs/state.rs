use anchor_lang::prelude::*;

use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

use crate::core_lib::vault::test::*;

#[repr(packed)]
#[zero_copy]
#[derive(Debug, Default, PartialEq)]
pub struct VaultEntry {
    pub data: TestStruct,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_reserve: Pubkey,
    pub quote_reserve: Pubkey,
    pub bump: u8,
}

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
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub vaults: Vaults,
    pub nonce: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttt() {
        println!("{}", std::mem::size_of::<State>());
    }
}
