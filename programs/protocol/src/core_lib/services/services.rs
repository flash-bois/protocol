use crate::core_lib::errors::LibErrors;

use super::{lending::Lend, swapping::Swap, trading::Trade};

#[cfg(feature = "anchor")]
mod zero {
    use super::*;
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[repr(C)]
    #[derive(Debug, Default, PartialEq)]
    pub struct Services {
        pub swap: Option<Swap>,
        pub lend: Option<Lend>,
        pub trade: Option<Trade>,
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {

    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    #[repr(C)]
    pub struct Services {
        pub swap: Option<Swap>,
        pub lend: Option<Lend>,
        pub trade: Option<Trade>,
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
    pub fn trade(&self) -> Result<&Trade, LibErrors> {
        self.trade.as_ref().ok_or(LibErrors::TradeServiceNone)
    }

    pub fn swap(&self) -> Result<&Swap, LibErrors> {
        self.swap.as_ref().ok_or(LibErrors::SwapServiceNone)
    }

    pub fn lend(&self) -> Result<&Lend, LibErrors> {
        self.lend.as_ref().ok_or(LibErrors::LendServiceNone)
    }

    pub fn swap_mut(&mut self) -> Result<&mut Swap, LibErrors> {
        self.swap.as_mut().ok_or(LibErrors::SwapServiceNone)
    }

    pub fn lend_mut(&mut self) -> Result<&mut Lend, LibErrors> {
        self.lend.as_mut().ok_or(LibErrors::LendServiceNone)
    }

    pub fn trade_mut(&mut self) -> Result<&mut Trade, LibErrors> {
        self.trade.as_mut().ok_or(LibErrors::TradeServiceNone)
    }
}
