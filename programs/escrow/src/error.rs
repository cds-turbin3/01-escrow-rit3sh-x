use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has expired")]
    EscrowExpired,

    #[msg("Expiration date is too old")]
    ExpirationDateTooOld,

    #[msg("Ask amount must be greater than zero")]
    AmountIsZero,

    #[msg("Deposit must be greater than zero")]
    DepositIsZero,

    #[msg("Mint A and mint B must be different")]
    MintsMustDiffer,
}
