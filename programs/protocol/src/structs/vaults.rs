use crate::core_lib::services::lending::Lend;
use crate::core_lib::services::Services;
use crate::core_lib::strategy::Strategies;
use crate::core_lib::structs::{FeeCurve, Oracle};
use crate::core_lib::Vault;
use anchor_lang::prelude::*;
use checked_decimal_macro::num_traits::ToPrimitive;
use std::ops::Range;
use std::slice::{Iter, IterMut};
use vec_macro::SafeArray;

#[zero_copy]
#[repr(packed)]
#[derive(Debug, Default, PartialEq)]
pub struct VaultEntry {
    pub data: Vault,
}

#[zero_copy]
#[repr(packed)]
#[derive(Debug, Default, PartialEq)]
pub struct VaultKeys {
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_reserve: Pubkey,
    pub quote_reserve: Pubkey,
}

#[zero_copy]
#[repr(packed)]
#[derive(Debug, SafeArray)]
pub struct VaultsArray {
    pub head: u8,
    pub elements: [Vault; 10],
}

#[zero_copy]
#[repr(packed)]
#[derive(Debug, SafeArray)]
pub struct VaultsKeysArray {
    pub head: u8,
    pub elements: [VaultKeys; 10],
}

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct Vaults {
    pub arr: VaultsArray,
    pub keys: VaultsKeysArray,
}
