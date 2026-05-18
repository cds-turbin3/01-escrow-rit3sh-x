pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

#[cfg(not(target_os = "solana"))]
pub mod test_helpers;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5YuYrfNC8emUaLBbHcu7AvyxNRgbvp9B5TaDehFz9g9K");

#[program]
pub mod escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        amount: u64,
        deposit: u64,
        expiry_utc: Option<i64>,
    ) -> Result<()> {
        ctx.accounts
            .init_escrow(seed, amount, &ctx.bumps, expiry_utc)?;
        ctx.accounts.deposit(deposit)
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        ctx.accounts.escrow.assert_active(now)?;
        ctx.accounts.pay_maker()?;
        ctx.accounts.release_to_taker()
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
