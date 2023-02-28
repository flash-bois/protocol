#[cfg(feature = "anchor")]
mod zero {
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
}

#[cfg(not(feature = "anchor"))]
pub use non_zero::State;
#[cfg(feature = "anchor")]
pub use zero::State;
