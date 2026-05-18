use anchor_litesvm::{AnchorLiteSVM, TestHelpers, TransactionHelpers};
use escrow::{ID, instruction};

mod common;
use common::{
    DEPOSIT, MINT_A_DECIMALS, MINT_B_DECIMALS, RECEIVE, SEED, make_ix_with, run_make,
    setup_accounts,
};

#[test]
fn take_settles_swap_and_closes_escrow() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    run_make(&mut ctx, &accs, None);

    let ix = ctx.program().build_ix(accs.bundle, instruction::Take {});
    ctx.svm.send_ok(ix, &[&accs.taker]).print_logs_structured();

    assert_eq!(ctx.svm.token_balance(&accs.taker_ata_a), Some(DEPOSIT), "taker should hold DEPOSIT mint_a");
    assert_eq!(ctx.svm.token_balance(&accs.maker_ata_b), Some(RECEIVE), "maker should hold RECEIVE mint_b");
    assert!(ctx.svm.get_account(&accs.vault).is_none(), "vault should be closed");
    assert!(ctx.svm.get_account(&accs.escrow).is_none(), "escrow should be closed");
}

#[test]
fn take_succeeds_before_expiry() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    let expiry = ctx.svm.get_unix_timestamp() + 60 * 60; // 1h window
    run_make(&mut ctx, &accs, Some(expiry));

    // Stay inside the expiry window; take should still succeed.
    let ix = ctx.program().build_ix(accs.bundle, instruction::Take {});
    ctx.svm.send_ok(ix, &[&accs.taker]);

    assert!(ctx.svm.get_account(&accs.escrow).is_none());
    assert!(ctx.svm.get_account(&accs.vault).is_none());
}

#[test]
fn take_after_expiry_fails_with_escrow_expired() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    let expiry = ctx.svm.get_unix_timestamp() + 60 * 60; // 1h window
    run_make(&mut ctx, &accs, Some(expiry));

    ctx.svm.advance_seconds(2 * 60 * 60);

    let ix = ctx.program().build_ix(accs.bundle, instruction::Take {});
    ctx.svm.send_anchor_err(ix, &[&accs.taker], "EscrowExpired");

    assert!(ctx.svm.get_account(&accs.escrow).is_some());
    assert!(ctx.svm.get_account(&accs.vault).is_some());
}

/// Pin the asymmetric-decimals invariant the rest of the suite relies on.
/// `MINT_A_DECIMALS != MINT_B_DECIMALS` means that any future regression in
/// the Anchor constraints which left a mint pubkey free to be swapped at
/// the call site would still surface at `transfer_checked`, because the
/// SPL token CPI rejects when the passed decimals don't match the on-chain
/// mint. If someone flattens these to the same value, this test trips so
/// the README's claim about that defense doesn't quietly go stale.
#[test]
fn asymmetric_mint_decimals_are_pinned() {
    assert_ne!(
        MINT_A_DECIMALS, MINT_B_DECIMALS,
        "tests rely on asymmetric mint decimals as a second line of defense \
         against mint-swap bugs at the transfer_checked CPI layer"
    );
}

/// Spoof: hand `take` the bundle with `mint_a` and `mint_b` swapped. The
/// escrow on disk still records the real (mint_a, mint_b) pair, so Anchor's
/// account-validation constraints (the `associated_token::mint = mint_a`
/// check on `taker_ata_a` / `vault`, or `has_one = mint_a` / `has_one =
/// mint_b` on the escrow) must reject the swap before any token transfer
/// runs.
///
/// Belt-and-suspenders: even if those constraints ever regressed, the
/// asymmetric decimals (see `asymmetric_mint_decimals_are_pinned`) would
/// still trip `transfer_checked` at the SPL token CPI because the passed
/// decimals wouldn't match the on-chain mint. This test exercises the
/// first line; the constants pin the second.
#[test]
fn take_rejects_swapped_mints() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    run_make(&mut ctx, &accs, None);

    let mut spoofed = accs.bundle;
    std::mem::swap(&mut spoofed.mint_a, &mut spoofed.mint_b);

    let ix = ctx.program().build_ix(spoofed, instruction::Take {});
    ctx.svm
        .send_instruction(ix, &[&accs.taker])
        .unwrap()
        .assert_failure();

    assert!(
        ctx.svm.get_account(&accs.escrow).is_some(),
        "escrow must survive a rejected take",
    );
    assert_eq!(
        ctx.svm.token_balance(&accs.vault),
        Some(DEPOSIT),
        "vault balance must be untouched after a rejected take",
    );
}

#[test]
fn take_drains_vault_when_deposit_differs_from_amount() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);

    // Asymmetric scenario: the maker put up more mint_a than they're asking
    // for in mint_b. Take must move `deposit` mint_a out of the vault and
    // `amount` mint_b from taker to maker — confusing the two would surface
    // here as a wrong post-balance.
    let deposit = 2_500_000;
    let amount = 750_000_000;
    let (make_ix, bundle) = make_ix_with(&ctx, &accs, SEED, amount, deposit, None);
    ctx.svm.send_ok(make_ix, &[&accs.maker]);
    assert_eq!(ctx.svm.token_balance(&accs.vault), Some(deposit));

    let taker_b_before = ctx.svm.token_balance(&accs.taker_ata_b).expect("taker_ata_b");

    let take_ix = ctx.program().build_ix(bundle, instruction::Take {});
    ctx.svm.send_ok(take_ix, &[&accs.taker]);

    assert!(ctx.svm.get_account(&accs.escrow).is_none());
    assert!(ctx.svm.get_account(&accs.vault).is_none());

    assert_eq!(
        taker_b_before - ctx.svm.token_balance(&accs.taker_ata_b).expect("taker_ata_b"),
        amount,
        "taker B drop must match escrow.amount, not vault.amount",
    );
    assert_eq!(
        ctx.svm.token_balance(&accs.taker_ata_a),
        Some(deposit),
        "taker A gain must match vault.amount (= deposit)",
    );
    assert_eq!(
        ctx.svm.token_balance(&accs.maker_ata_b),
        Some(amount),
        "maker B gain must match escrow.amount",
    );
}
