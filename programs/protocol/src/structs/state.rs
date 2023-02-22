use anchor_lang::prelude::*;

#[account(zero_copy)]
#[derive(Debug, Default)]
pub struct State {
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub vaults: Pubkey,
    pub bump: u8,
    pub nonce: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttt() {
        println!("{}", std::mem::size_of::<State>());
    }
}
