use super::swapping::Swap;

#[derive(Clone, Debug, Default)]
pub struct Services {
    pub swap: Option<Swap>,
}

pub enum ServiceType {
    /// Lending allows borrowing of tokens
    Lend,
    /// Swapping allows swapping tokens between each other
    Swap,
    /// Trading allows creating leveraged LONG and SHORT positions
    Trade,
}

impl Services {
    pub fn swap_service(&mut self) -> Result<&mut Swap, ()> {
        self.swap.as_mut().ok_or(())
    }
}
