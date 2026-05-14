use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has expired")]
    EscrowExpired,

    #[msg("Expiration date is too old")]
    ExpirationDateTooOld,
}
