use super::{lending::Lend, swapping::Swap};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(packed)]
pub struct Services {
    pub swap: Option<Swap>,
    pub lend: Option<Lend>,
}

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
