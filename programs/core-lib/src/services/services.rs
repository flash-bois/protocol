use super::{lending::Lend, swapping::Swap, trading::Trade};

#[derive(Clone, Debug, Default)]
pub struct Services {
    pub swap: Option<Swap>,
    pub lend: Option<Lend>,
    pub trade: Option<Trade>,
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

    pub fn trade_mut(&mut self) -> Result<&mut Trade, ()> {
        self.trade.as_mut().ok_or(())
    }
}
