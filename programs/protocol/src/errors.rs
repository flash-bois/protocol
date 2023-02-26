use anchor_lang::error_code;

#[error_code]
pub enum MyError {
    #[msg("Too large data")]
    DataTooLarge,
}
