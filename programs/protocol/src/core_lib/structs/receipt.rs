use crate::core_lib::decimal::{FundingRate, Price, Quantity, Value};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Long,
    Short,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Receipt {
    /// side of the position: either long or short
    pub side: Side,
    /// size of the position (in base token)
    pub size: Quantity,
    /// quantity of tokens locked in the position (size for LONG)
    pub locked: Quantity,
    /// shares for borrow fee
    pub initial_funding: FundingRate,
    /// price at which the position was opened
    pub open_price: Price,
    /// value o position at the moment of creation
    pub open_value: Value,
}
