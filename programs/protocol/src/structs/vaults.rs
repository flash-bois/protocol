use crate::core_lib::vault::Vault;
use anchor_lang::prelude::*;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

#[zero_copy]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct Inner {
    pub head: u8,
    pub elements: [VaultEntry; 10],
}

// impl Inner {
//     pub fn new() -> Self {
//         Self {
//             head: 0,
//             elements: [(); 10].map(|_| VaultEntry::default()),
//         }
//     }
// }

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct Vaults {
    pub arr: Inner,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t() {
        println!("{}", std::mem::size_of::<Inner>())
    }
}

#[zero_copy]
#[repr(packed)]
#[derive(Debug, Default, PartialEq)]
pub struct VaultEntry {
    pub data: Vault,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_reserve: Pubkey,
    pub quote_reserve: Pubkey,
    pub bump: u8,
}
