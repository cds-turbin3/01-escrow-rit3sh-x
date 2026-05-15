use crate::error::EscrowError;
pub use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount: u64,
    pub bump: u8,
    pub expiry_utc: Option<i64>,
}

impl Escrow {
    pub fn assert_active(&self, now: i64) -> Result<()> {
        if let Some(expiry) = self.expiry_utc {
            require!(now <= expiry, EscrowError::EscrowExpired);
        }
        Ok(())
    }
}
