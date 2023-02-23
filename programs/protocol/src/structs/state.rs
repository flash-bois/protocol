use anchor_lang::prelude::*;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, Default)]
pub struct State {
    pub admin: Pubkey,
    pub vaults_acc: Pubkey,
    pub bump: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttt() {
        println!("{}", std::mem::size_of::<State>());
    }
}
