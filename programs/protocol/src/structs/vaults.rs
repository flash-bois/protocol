#[cfg(feature = "anchor")]
mod zero {
    use crate::core_lib::Vault;
    use anchor_lang::prelude::*;
    use checked_decimal_macro::num_traits::ToPrimitive;
    use std::ops::Range;
    use std::slice::{Iter, IterMut};
    use vec_macro::SafeArray;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq)]
    pub struct VaultEntry {
        pub data: Vault,
    }

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq)]
    pub struct VaultKeys {
        pub base_token: Pubkey,
        pub quote_token: Pubkey,
        pub base_reserve: Pubkey,
        pub quote_reserve: Pubkey,
        pub base_oracle: Option<Pubkey>,
        pub quote_oracle: Option<Pubkey>,
    }

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, SafeArray)]
    pub struct VaultsArray {
        pub head: u8,
        pub elements: [Vault; 10],
    }

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, SafeArray)]
    pub struct VaultsKeysArray {
        pub head: u8,
        pub elements: [VaultKeys; 10],
    }

    #[account(zero_copy)]
    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct Vaults {
        pub arr: VaultsArray,
        pub keys: VaultsKeysArray,
    }
}

#[cfg(feature = "wasm")]
mod non_zero {
    use crate::{core_lib::vault::Vault, wasm_wrapper::to_buffer};
    use checked_decimal_macro::num_traits::ToPrimitive;
    use std::{
        ops::Range,
        slice::{Iter, IterMut},
    };
    use vec_macro::SafeArray;

    #[repr(C)]
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    pub struct VaultEntry {
        pub data: Vault,
    }

    #[repr(C)]
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    pub struct VaultKeys {
        pub base_token: [u8; 32], // buffer
        pub quote_token: [u8; 32],
        pub base_reserve: [u8; 32],
        pub quote_reserve: [u8; 32],
        pub base_oracle: Option<[u8; 32]>,
        pub quote_oracle: Option<[u8; 32]>,
    }

    #[repr(C)]
    #[derive(Debug, SafeArray, Clone, Copy)]
    pub struct VaultsArray {
        pub head: u8,
        pub elements: [Vault; 10],
    }

    #[repr(C)]
    #[derive(Debug, SafeArray, Clone, Copy)]
    pub struct VaultsKeysArray {
        pub head: u8,
        pub elements: [VaultKeys; 10],
    }

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Vaults {
        pub padding: [u8; 8],
        pub arr: VaultsArray,
        pub keys: VaultsKeysArray,
    }

    unsafe impl bytemuck::Pod for Vaults {}
    unsafe impl bytemuck::Zeroable for Vaults {}
}

#[cfg(feature = "wasm")]
pub use non_zero::*;

#[cfg(feature = "anchor")]
pub use zero::*;

#[cfg(test)]
mod tests {
    use crate::structs::Statement;

    use super::*;
    use std::mem::size_of;

    #[test]
    fn size() {
        println!("{}", size_of::<VaultsArray>());
        println!("{}", size_of::<VaultsKeysArray>());
        println!("{}", size_of::<Vaults>());
        println!("{}", size_of::<Statement>());
    }
}
