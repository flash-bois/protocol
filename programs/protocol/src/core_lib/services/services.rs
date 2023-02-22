use super::{lending::Lend, swapping::Swap};

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Debug, Default, PartialEq)]
    pub struct Services {
        pub swap: Option<Swap>,
        pub lend: Option<Lend>,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(packed)]
    pub struct Services {
        pub swap: Option<Swap>,
        pub lend: Option<Lend>,
    }
}

#[cfg(feature = "anchor")]
pub use zero::Services;

#[cfg(not(feature = "anchor"))]
pub use mon_zero::Services;

#[derive(Clone, Copy)]
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
        self.swap.as_mut().ok_or(())
    }

    pub fn lend_mut(&mut self) -> Result<&mut Lend, ()> {
        self.lend.as_mut().ok_or(())
    }
}
