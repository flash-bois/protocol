use super::{lending::Lend, swapping::Swap};

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(packed)]
    #[derive(Debug, Default, PartialEq)]
    pub struct Services {
        pub swap: Swap,
        pub lend: Lend,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(packed)]
    pub struct Services {
        pub swap: Swap,
        pub lend: Lend,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Services;

#[cfg(not(feature = "anchor"))]
pub use non_zero::Services;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ServiceType {
    /// Lending allows borrowing of tokens
    Lend,
    /// Swapping allows swapping tokens between each other
    Swap,
    /// Trading allows creating leveraged LONG and SHORT positions
    Trade,
}

impl Services {
    pub fn swap_mut(&mut self) -> Result<&mut Swap, ()> {
        Ok(&mut self.swap)
    }

    pub fn lend_mut(&mut self) -> Result<&mut Lend, ()> {
        Ok(&mut self.lend)
    }
}
