#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[account(zero_copy)]
    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct State {
        pub bump: u8,
        pub admin: Pubkey,
        pub vaults_acc: Pubkey,
    }
}

#[cfg(feature = "wasm")]
mod non_zero {

    use crate::ZeroCopyDecoder;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::*;

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct State {
        pub padding: [u8; 8],
        pub bump: u8,
        pub admin: [u8; 32],
        pub vaults_acc: [u8; 32],
    }
    #[automatically_derived]
    unsafe impl bytemuck::Pod for State {}
    #[automatically_derived]
    unsafe impl bytemuck::Zeroable for State {}

    #[wasm_bindgen]
    pub struct StateAccount {
        account: State,
    }

    #[wasm_bindgen]
    impl StateAccount {
        #[wasm_bindgen]
        pub fn load(account_info: &Uint8Array) -> Self {
            let v = account_info.to_vec();
            let account = *ZeroCopyDecoder::decode_account_info::<State>(&v);
            Self { account }
        }

        #[wasm_bindgen]
        pub fn get_bump(&self) -> u8 {
            self.account.bump
        }
    }
}

#[cfg(not(feature = "anchor"))]
pub use non_zero::State;
#[cfg(feature = "anchor")]
pub use zero::State;
