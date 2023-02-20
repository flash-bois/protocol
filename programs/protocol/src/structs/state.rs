use super::VaultEntry;
use anchor_lang::prelude::*;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

use core_lib::vault::*;

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
