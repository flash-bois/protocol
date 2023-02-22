#[cfg(feature = "anchor")]
mod zero {
    use anchor_lang::prelude::*;

    #[zero_copy]
    #[derive(Debug, Default, PartialEq)]
    pub struct TestStruct {
        pub arr: [i32; 10],
    }
}

#[cfg(not(feature = "anchor"))]
mod non_zero {
    #[derive(Debug, Default, PartialEq, Clone, Copy)]
    pub struct TestStruct {
        pub arr: [i32; 10],
    }
}

#[cfg(feature = "anchor")]
pub use zero::TestStruct;

#[cfg(not(feature = "anchor"))]
pub use mon_zero::TestStruct;
