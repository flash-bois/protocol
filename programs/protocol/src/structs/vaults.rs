use crate::core_lib::errors::LibErrors;
use crate::core_lib::Vault;

#[cfg(feature = "anchor")]
mod zero {
    use crate::core_lib::structs::Oracle;
    use crate::pyth::{get_oracle_update_data, OracleUpdate};

    use super::*;

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

    impl Vaults {
        fn find_key_and_update_oracle(
            oracle: &mut Oracle,
            accounts: &[AccountInfo],
            key: &Pubkey,
            current_timestamp: i64,
        ) -> std::result::Result<(), LibErrors> {
            let acc = accounts
                .iter()
                .find(|acc| *acc.key == *key)
                .ok_or(LibErrors::OracleAccountNotFound)?;

            let OracleUpdate { price, confidence } =
                get_oracle_update_data(acc, current_timestamp)?;

            oracle.update(
                price,
                confidence,
                current_timestamp
                    .try_into()
                    .map_err(|_| LibErrors::ParseError)?,
            )
        }

        pub fn refresh_all(
            &mut self,
            accounts: &[AccountInfo],
        ) -> std::result::Result<(), LibErrors> {
            let indexes = self.arr.indexes();

            for index in indexes {
                let (vault, vault_keys) = self.vault_with_keys(index as u8)?;

                let current_timestamp =
                    Clock::get().map_err(|_| LibErrors::TimeGet)?.unix_timestamp;

                if let Some(ref mut base_oracle) = vault.oracle {
                    let key = vault_keys
                        .base_oracle
                        .as_ref()
                        .ok_or(LibErrors::PubkeyMissing)?;

                    Self::find_key_and_update_oracle(
                        base_oracle,
                        accounts,
                        key,
                        current_timestamp,
                    )?;
                }

                if let Some(ref mut quote_oracle) = vault.quote_oracle {
                    let key = vault_keys
                        .quote_oracle
                        .as_ref()
                        .ok_or(LibErrors::PubkeyMissing)?;

                    Self::find_key_and_update_oracle(
                        quote_oracle,
                        accounts,
                        key,
                        current_timestamp,
                    )?;
                }
            }
            Ok(())
        }
    }
}
#[cfg(feature = "wasm")]
mod non_zero {
    use checked_decimal_macro::num_traits::ToPrimitive;
    use std::{
        ops::Range,
        slice::{Iter, IterMut},
    };
    use vec_macro::SafeArray;
    use wasm_bindgen::prelude::*;

    use crate::core_lib::vault::Vault;

    #[repr(C)]
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    pub struct VaultEntry {
        pub data: Vault,
    }

    #[repr(C)]
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    pub struct VaultKeys {
        pub base_token: [u8; 32],
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

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct VaultsAccount {
        pub(crate) account: Vaults,
    }
}

#[cfg(feature = "wasm")]
pub use non_zero::*;

#[cfg(feature = "anchor")]
pub use zero::*;

impl Vaults {
    pub fn vault_checked(&self, index: u8) -> Result<&Vault, LibErrors> {
        Ok(self
            .arr
            .get_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }

    pub fn keys_checked(&self, index: u8) -> Result<&VaultKeys, LibErrors> {
        Ok(self
            .keys
            .get_checked(index as usize)
            .ok_or(LibErrors::IndexOutOfBounds)?)
    }

    pub fn keys_checked_mut(&mut self, index: u8) -> Result<&mut VaultKeys, LibErrors> {
        Ok(self
            .keys
            .get_mut_checked(index as usize)
            .ok_or(LibErrors::IndexOutOfBounds)?)
    }

    pub fn vault_checked_mut(&mut self, index: u8) -> Result<&mut Vault, LibErrors> {
        Ok(self
            .arr
            .get_mut_checked(index as usize)
            .ok_or(LibErrors::NoVaultOnIndex)?)
    }

    pub fn vault_with_keys(&mut self, index: u8) -> Result<(&mut Vault, &VaultKeys), LibErrors> {
        let Self { arr, keys } = self;

        Ok((
            arr.get_mut_checked(index as usize)
                .ok_or(LibErrors::NoVaultOnIndex)?,
            keys.get_checked(index as usize)
                .ok_or(LibErrors::IndexOutOfBounds)?,
        ))
    }
}
