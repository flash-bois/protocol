#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[account(zero_copy)]
    #[repr(packed)]
    #[derive(Debug, Default)]
    pub struct State {
        pub admin: Pubkey,
        pub vaults_acc: Pubkey,
        pub bump: u8,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {

    #[repr(packed)]
    #[derive(Debug, Default)]
    pub struct State {
        pub admin: [u8; 32],
        pub vaults_acc: [u8; 32],
        pub bump: u8,
    }
}

#[cfg(not(feature = "anchor"))]
pub use non_zero::State;
#[cfg(feature = "anchor")]
pub use zero::State;
