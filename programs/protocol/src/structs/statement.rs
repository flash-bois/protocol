use crate::core_lib::user::UserStatement;
use anchor_lang::prelude::*;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct Statement {
    pub owner: Pubkey,
    pub bump: u8,
    pub statement: UserStatement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt() {
        println!("{}", std::mem::size_of::<Statement>())
    }
}
