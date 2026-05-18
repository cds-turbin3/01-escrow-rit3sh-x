//! Shared scaffolding for the escrow integration tests.
//!
//! Each `.rs` file under `tests/` is a separate crate; this module is pulled
//! in via `mod common;` from each test file. Helpers that aren't used by a
//! given test file would otherwise trip `dead_code`; the blanket allow keeps
//! both call sites honest without forcing a parallel set of trimmed-down
//! variants.

#![allow(dead_code)]

use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_litesvm::{AnchorContext, Signer, TestHelpers, TransactionHelpers};
use anchor_spl::associated_token::get_associated_token_address;
use escrow::test_helpers::EscrowBundle;
use escrow::{ESCROW_SEED, ID, instruction};
use solana_keypair::Keypair;

/// Scenario constants. `DEPOSIT != RECEIVE` and the two mints differ in
/// decimals so a `take` that confused `escrow.amount`/`vault.amount` or
/// `mint_a`/`mint_b` decimals would produce visibly wrong post-balances.
pub const SEED: u64 = 42;
pub const MINT_A_DECIMALS: u8 = 6;
pub const MINT_B_DECIMALS: u8 = 9;
pub const DEPOSIT: u64 = 1_000_000;
pub const RECEIVE: u64 = 2_000_000_000;
pub const FUNDED_A: u64 = 5_000_000;
pub const FUNDED_B: u64 = 5_000_000_000;

pub struct Accounts {
    pub maker: Keypair,
    pub taker: Keypair,
    pub mint_a: Keypair,
    pub mint_b: Keypair,

    // Pubkeys lifted to the top level so assertions can write `accs.vault`
    // instead of `accs.bundle.vault` or re-deriving via
    // `get_associated_token_address`. These mirror the bundle's fields for
    // the default `SEED`; tests that build for a different seed via
    // `make_ix_with` should use the returned bundle's `escrow`/`vault`.
    pub maker_ata_a: Pubkey,
    pub maker_ata_b: Pubkey,
    pub taker_ata_a: Pubkey,
    pub taker_ata_b: Pubkey,
    pub escrow: Pubkey,
    pub vault: Pubkey,

    /// Fully populated EscrowBundle: every field that any of make/take/refund
    /// can read is set, so the same bundle is reusable across ixs.
    pub bundle: EscrowBundle,
}

/// Stand up maker + taker + both mints + funded ATAs and derive every
/// pubkey the program will read or init. Does NOT run `make` — the escrow
/// PDA and vault ATA are addresses only until the caller invokes `make`.
pub fn setup_accounts(ctx: &mut AnchorContext) -> Accounts {
    let maker = ctx
        .svm
        .create_funded_account(10_000_000_000)
        .expect("fund maker");
    let taker = ctx
        .svm
        .create_funded_account(10_000_000_000)
        .expect("fund taker");

    let mint_a = ctx
        .svm
        .create_token_mint(&maker, MINT_A_DECIMALS)
        .expect("create mint_a");
    let mint_b = ctx
        .svm
        .create_token_mint(&maker, MINT_B_DECIMALS)
        .expect("create mint_b");

    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .expect("create maker_ata_a");
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, FUNDED_A)
        .expect("fund maker_ata_a");

    let taker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &taker)
        .expect("create taker_ata_b");
    ctx.svm
        .mint_to(&mint_b.pubkey(), &taker_ata_b, &maker, FUNDED_B)
        .expect("fund taker_ata_b");

    let escrow = ctx.svm.get_pda(
        &[ESCROW_SEED, maker.pubkey().as_ref(), &SEED.to_le_bytes()],
        &ID,
    );
    let vault = get_associated_token_address(&escrow, &mint_a.pubkey());
    let taker_ata_a = get_associated_token_address(&taker.pubkey(), &mint_a.pubkey());
    let maker_ata_b = get_associated_token_address(&maker.pubkey(), &mint_b.pubkey());

    let bundle = EscrowBundle {
        maker: maker.pubkey(),
        taker: taker.pubkey(),
        mint_a: mint_a.pubkey(),
        mint_b: mint_b.pubkey(),
        maker_ata_a,
        maker_ata_b,
        taker_ata_a,
        taker_ata_b,
        escrow,
        vault,
    };

    Accounts {
        maker,
        taker,
        mint_a,
        mint_b,
        maker_ata_a,
        maker_ata_b,
        taker_ata_a,
        taker_ata_b,
        escrow,
        vault,
        bundle,
    }
}

/// Run the default `make` ix (seed=SEED, amount=RECEIVE, deposit=DEPOSIT) so
/// the vault is funded and the escrow account exists. Both take and refund
/// tests start from this state.
pub fn run_make(ctx: &mut AnchorContext, accs: &Accounts, expiry_utc: Option<i64>) {
    let (ix, _) = make_ix_with(ctx, accs, SEED, RECEIVE, DEPOSIT, expiry_utc);
    ctx.svm
        .send_instruction(ix, &[&accs.maker])
        .unwrap()
        .assert_success();
}

/// Build (but do not send) a `make` ix with arbitrary seed/amount/deposit.
/// Returns the ix and a bundle whose `escrow`/`vault` derive from the given
/// seed, so the caller can reuse it for follow-up ixs (take, refund) against
/// the resulting escrow.
pub fn make_ix_with(
    ctx: &AnchorContext,
    accs: &Accounts,
    seed: u64,
    amount: u64,
    deposit: u64,
    expiry_utc: Option<i64>,
) -> (Instruction, EscrowBundle) {
    let escrow = ctx.svm.get_pda(
        &[ESCROW_SEED, accs.maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &ID,
    );
    let vault = get_associated_token_address(&escrow, &accs.mint_a.pubkey());
    let mut bundle = accs.bundle;
    bundle.escrow = escrow;
    bundle.vault = vault;

    let ix = ctx.program().build_ix(
        bundle,
        instruction::Make {
            seed,
            amount,
            deposit,
            expiry_utc,
        },
    );
    (ix, bundle)
}

