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
    use checked_decimal_macro::num_traits::ToPrimitive;
    use js_sys::Uint8Array;
    use std::{
        ops::Range,
        slice::{Iter, IterMut},
    };
    use vec_macro::SafeArray;
    use wasm_bindgen::prelude::*;

    use crate::{core_lib::vault::Vault, to_buffer};

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
    pub struct VaultsAccount {
        account: Vaults,
    }

    #[wasm_bindgen]
    impl VaultsAccount {
        #[wasm_bindgen]
        pub fn load(account_info: &Uint8Array) -> Self {
            console_error_panic_hook::set_once();
            // panic!("not a panic");

            let v = account_info.to_vec();
            let account = *bytemuck::from_bytes::<Vaults>(&v);
            Self { account }
        }

        #[wasm_bindgen]
        pub fn vaults_len(&self) -> u8 {
            self.account.arr.head
        }

        #[wasm_bindgen]
        pub fn size() -> usize {
            std::mem::size_of::<Vaults>()
        }

        #[wasm_bindgen]
        pub fn base_token(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .base_token,
            )
        }

        #[wasm_bindgen]
        pub fn quote_token(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .quote_token,
            )
        }

        #[wasm_bindgen]
        pub fn base_reserve(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .base_reserve,
            )
        }

        #[wasm_bindgen]
        pub fn quote_reserve(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .quote_reserve,
            )
        }

        #[wasm_bindgen]
        pub fn oracle_base(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .base_oracle
                    .expect("base oracle not initialized"), // ERROR CODE
            )
        }

        #[wasm_bindgen]
        pub fn oracle_quote(&self, index: u8) -> Uint8Array {
            to_buffer(
                &self
                    .account
                    .keys
                    .get_checked(index as usize)
                    .expect("index out of bounds")
                    .quote_oracle
                    .expect("quote oracle not initialized"), // ERROR CODE
            )
        }

        #[wasm_bindgen]
        pub fn base_oracle_enabled(&self, index: u8) -> bool {
            self.account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .base_oracle
                .is_some()
        }

        #[wasm_bindgen]
        pub fn quote_oracle_enabled(&self, index: u8) -> bool {
            self.account
                .keys
                .get_checked(index as usize)
                .expect("index out of bounds")
                .quote_oracle
                .is_some()
        }
    }
}

#[cfg(feature = "wasm")]
pub use non_zero::*;

#[cfg(feature = "anchor")]
pub use zero::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn size() {
        println!("{}", size_of::<VaultsArray>());
        println!("{}", size_of::<VaultsKeysArray>());
        println!("{}", size_of::<Vaults>());
    }
}
// 16168
// 1281
// 17456

// 16168
// 1281
// 17464

//packed
// 13221
// 1281
// 14502

// 13431
// 1281
// 14720
