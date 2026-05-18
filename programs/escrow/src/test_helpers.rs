//! Host-side test scaffolding. Gated `#[cfg(not(target_os = "solana"))]`
//! so the BPF program build doesn't pull in anchor-litesvm.

use anchor_lang::prelude::Pubkey;
use anchor_litesvm::Bundle;

/// Union of all bundle-projected accounts across Make, Take, and Refund.
/// Each instruction's `From` impl only references the subset of fields it
/// actually has, so unused fields per-instruction are fine.
///
/// This struct is the test suite's **index page for accounts**: every account
/// any test touches is listed here, documented with its role and lifecycle.
/// Use the field names as a search dimension when navigating the suite
/// (e.g. `grep vault tests/` finds every test that touches the vault).
#[derive(Bundle, Copy, Clone, Debug)]
pub struct EscrowBundle {
    /// Maker's wallet pubkey. Signer of `make` and `refund`; SystemAccount
    /// (close-rent destination) on `take`. The escrow PDA pins this value
    /// via `has_one = maker`.
    pub maker: Pubkey,

    /// Taker's wallet pubkey. Signer of `take`. Pays init-if-needed for
    /// `taker_ata_a` and `maker_ata_b`.
    pub taker: Pubkey,

    /// Mint of the token the maker deposits. Tests use different `decimals`
    /// than `mint_b` so a take that confused the two would surface as a
    /// wrong post-balance.
    pub mint_a: Pubkey,

    /// Mint of the token the maker wants in exchange.
    pub mint_b: Pubkey,

    /// Maker's ATA for `mint_a`. Must exist before `make` (it's the source
    /// of the deposit transfer). `refund` returns the vault contents here.
    pub maker_ata_a: Pubkey,

    /// Maker's ATA for `mint_b`. Created by `take` via init-if-needed;
    /// destination for the taker's payment of `escrow.amount`.
    pub maker_ata_b: Pubkey,

    /// Taker's ATA for `mint_a`. Created by `take` via init-if-needed;
    /// destination for the vault contents (`vault.amount`) released by the
    /// program.
    pub taker_ata_a: Pubkey,

    /// Taker's ATA for `mint_b`. Must exist before `take` (it's the source
    /// of the taker's payment to the maker).
    pub taker_ata_b: Pubkey,

    /// Escrow account PDA: seeds `[b"escrow", maker, seed.to_le_bytes()]`.
    /// Inited by `make`, closed by `take` or `refund`. Tests with non-default
    /// seeds (e.g. `two_concurrent_escrows_use_distinct_seeds`) derive a
    /// different value via `common::make_ix_with(seed, ..)` and use the
    /// returned bundle's `escrow`/`vault` fields rather than the default
    /// ones mirrored on `common::Accounts`.
    pub escrow: Pubkey,

    /// Vault ATA: ATA(escrow, mint_a). Inited by `make` (holds the deposit);
    /// closed by `take` or `refund` after the contents are transferred out,
    /// with the rent returned to the maker.
    pub vault: Pubkey,
}
