use anchor_lang::error_code;

#[error_code]
pub enum NoLibErrors {
    #[msg("There is no defined vault on provided index")]
    NoVaultOnIndex,
    #[msg("Parsing between types was not successful")]
    ParseError,
    #[msg("Bump for given name not found in BSTree")]
    BumpNotFound,
    #[msg("Given decimal places are not expected")]
    InvalidDecimalPlaces,
    #[msg("Failed to add vault to vaults array")]
    AddVault,
    #[msg("Failed to add vault keys to vaults keys array")]
    AddKeys,
    #[msg("Amount out did not reached passed minimum")]
    NoMinAmountOut,
}
