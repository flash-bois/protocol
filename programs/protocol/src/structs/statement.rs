use crate::core_lib::user::UserStatement;

#[cfg(feature = "anchor")]
mod zero {
    use super::UserStatement;
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
    use super::UserStatement;

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Statement {
        pub padding: [u8; 8],
        pub statement: UserStatement,
        pub owner: [u8; 32],
        pub bump: u8,
    }

    unsafe impl bytemuck::Pod for Statement {}
    unsafe impl bytemuck::Zeroable for Statement {}
}

#[cfg(feature = "wasm")]
pub use non_zero::Statement;
#[cfg(feature = "anchor")]
pub use zero::Statement;
