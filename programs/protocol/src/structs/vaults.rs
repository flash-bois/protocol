use crate::core_lib::{errors::LibErrors, Vault};

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use crate::core_lib::structs::Oracle;
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

    impl VaultKeys {
        pub fn base_oracle(&self) -> std::result::Result<&Pubkey, LibErrors> {
            Ok(self.base_oracle.as_ref().ok_or(LibErrors::PubkeyMissing)?)
        }

        pub fn quote_oracle(&self) -> std::result::Result<&Pubkey, LibErrors> {
            Ok(self.quote_oracle.as_ref().ok_or(LibErrors::PubkeyMissing)?)
        }
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
        fn update_oracle_from_accs(
            oracle: &mut Oracle,
            accounts: &[AccountInfo],
            key: &Pubkey,
            current_timestamp: i64,
        ) -> std::result::Result<(), LibErrors> {
            let acc = accounts
                .iter()
                .find(|acc| *acc.key == *key)
                .ok_or(LibErrors::OracleAccountNotFound)?;

            Ok(oracle.update_from_acc(acc, current_timestamp)?)
        }

        pub fn refresh_all(
            &mut self,
            accounts: &[AccountInfo],
        ) -> std::result::Result<(), LibErrors> {
            let indexes = self.arr.indexes();
            let current_timestamp = Clock::get().map_err(|_| LibErrors::TimeGet)?.unix_timestamp;
            let current_timestamp_u32: u32 = current_timestamp
                .try_into()
                .map_err(|_| LibErrors::ParseError)?;

            for index in indexes {
                let (vault, vault_keys) = self.vault_with_keys(index as u8)?;

                vault.refresh(current_timestamp_u32)?;

                if let Some(ref mut base_oracle) = vault.oracle {
                    Self::update_oracle_from_accs(
                        base_oracle,
                        accounts,
                        vault_keys.base_oracle()?,
                        current_timestamp,
                    )?;
                }

                if let Some(ref mut quote_oracle) = vault.quote_oracle {
                    Self::update_oracle_from_accs(
                        quote_oracle,
                        accounts,
                        vault_keys.quote_oracle()?,
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
        let Self { arr, keys, .. } = self;

        Ok((
            arr.get_mut_checked(index as usize)
                .ok_or(LibErrors::NoVaultOnIndex)?,
            keys.get_checked(index as usize)
                .ok_or(LibErrors::IndexOutOfBounds)?,
        ))
    }
}
