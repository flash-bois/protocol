#[cfg(feature = "anchor")]
mod zero {
    use crate::core_lib::user::UserStatement;
    use anchor_lang::prelude::*;

    #[account(zero_copy)]
    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct Statement {
        pub statement: UserStatement,
        pub owner: Pubkey,
        pub bump: u8,
    }
}

#[cfg(feature = "wasm")]
mod non_zero {
    use crate::core_lib::user::UserStatement;
    use crate::ZeroCopyDecoder;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::*;

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Statement {
        pub padding: [u8; 8],
        pub statement: UserStatement,
        pub owner: [u8; 32],
        pub bump: u8,
    }
    #[automatically_derived]
    unsafe impl bytemuck::Pod for Statement {}
    #[automatically_derived]
    unsafe impl bytemuck::Zeroable for Statement {}

    #[wasm_bindgen]
    pub struct StatementAccount {
        account: Statement,
    }

    #[wasm_bindgen]
    impl StatementAccount {
        #[wasm_bindgen]
        pub fn load(account_info: &Uint8Array) -> Self {
            let v = account_info.to_vec();
            let account = *ZeroCopyDecoder::decode_account_info::<Statement>(&v);
            Self { account }
        }

        #[wasm_bindgen]
        pub fn get_bump(&self) -> u8 {
            self.account.bump
        }
    }
}

#[cfg(feature = "wasm")]
pub use non_zero::{Statement, StatementAccount};
#[cfg(feature = "anchor")]
pub use zero::Statement;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt() {
        println!("{}", std::mem::size_of::<Statement>())
    }
}
