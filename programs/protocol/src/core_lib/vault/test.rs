#[cfg(feature = "anchor")]
use anchor_lang::prelude::*;

#[cfg_attr(not(feature = "anchor"), derive(Clone, Copy))]
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "anchor", zero_copy)]
pub struct TestStruct {
    pub arr: [i32; 10],
}
