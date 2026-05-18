use anchor_litesvm::{AnchorLiteSVM, Signer, TestHelpers, TransactionHelpers};
use escrow::{ID, instruction};

mod common;
use common::{FUNDED_A, run_make, setup_accounts};

#[test]
fn refund_returns_vault_and_closes_state() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    run_make(&mut ctx, &accs, None);

    let ix = ctx.program().build_ix(accs.bundle, instruction::Refund {});
    ctx.svm.send_ok(ix, &[&accs.maker]).print_logs_structured();

    assert!(ctx.svm.get_account(&accs.escrow).is_none(), "escrow should be closed");
    assert!(ctx.svm.get_account(&accs.vault).is_none(), "vault should be closed");
    // Deposit went out then came back, so the maker is whole again.
    assert_eq!(ctx.svm.token_balance(&accs.maker_ata_a), Some(FUNDED_A));
}

#[test]
fn refund_works_after_expiry() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    let expiry = ctx.svm.get_unix_timestamp() + 60 * 60;
    run_make(&mut ctx, &accs, Some(expiry));

    // Refund has no time guard; expiry only gates take. Confirm here.
    ctx.svm.advance_seconds(2 * 60 * 60);

    let ix = ctx.program().build_ix(accs.bundle, instruction::Refund {});
    ctx.svm.send_ok(ix, &[&accs.maker]);

    assert!(ctx.svm.get_account(&accs.escrow).is_none());
    assert!(ctx.svm.get_account(&accs.vault).is_none());
    assert_eq!(ctx.svm.token_balance(&accs.maker_ata_a), Some(FUNDED_A));
}

#[test]
fn refund_signer_must_match_escrow_maker() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);
    run_make(&mut ctx, &accs, None);

    // For the program to reach `has_one = maker`, the `maker_ata_a` slot we
    // pass must be a real, initialized ATA — otherwise Anchor rejects at
    // the earlier `associated_token` constraint with AccountNotInitialized.
    ctx.svm
        .create_associated_token_account(&accs.mint_a.pubkey(), &accs.taker)
        .expect("create taker_ata_a");

    // Swap in the taker as the would-be maker. The escrow PDA on disk still
    // records the real maker; `has_one = maker` should reject the substitution.
    let mut spoofed = accs.bundle;
    spoofed.maker = accs.taker.pubkey();
    spoofed.maker_ata_a = accs.taker_ata_a;

    let ix = ctx.program().build_ix(spoofed, instruction::Refund {});
    ctx.svm.send_anchor_err(ix, &[&accs.taker], "ConstraintHasOne");

    assert!(
        ctx.svm.get_account(&accs.escrow).is_some(),
        "escrow must survive a failed refund"
    );
}
