use anchor_litesvm::{AnchorLiteSVM, TestHelpers, TransactionHelpers};
use escrow::state::Escrow;
use escrow::{ID, instruction};

mod common;
use common::{DEPOSIT, FUNDED_A, RECEIVE, SEED, make_ix_with, run_make, setup_accounts};

#[test]
fn make_works_no_expiry() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));

    let accs = setup_accounts(&mut ctx);
    let ix = ctx.program().build_ix(
        accs.bundle,
        instruction::Make {
            seed: SEED,
            amount: RECEIVE,
            deposit: DEPOSIT,
            expiry_utc: None,
        },
    );
    ctx.svm.send_ok(ix, &[&accs.maker]).print_logs_structured();
}

#[test]
fn make_works_expiry() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));

    let accs = setup_accounts(&mut ctx);
    let expiry = ctx.svm.get_unix_timestamp() + 30 * 24 * 60 * 60; // now + 30 days
    let ix = ctx.program().build_ix(
        accs.bundle,
        instruction::Make {
            seed: SEED,
            amount: RECEIVE,
            deposit: DEPOSIT,
            expiry_utc: Some(expiry),
        },
    );
    ctx.svm.send_ok(ix, &[&accs.maker]);

    // The persisted escrow should round-trip the expiry we passed in.
    let escrow: Escrow = ctx.load(&accs.escrow);
    assert_eq!(escrow.expiry_utc, Some(expiry));
}

#[test]
fn make_locks_tokens_in_vault_and_initialises_escrow() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);

    let before = ctx.svm.token_balance(&accs.maker_ata_a).expect("maker_ata_a");
    run_make(&mut ctx, &accs, None);
    let after = ctx.svm.token_balance(&accs.maker_ata_a).expect("maker_ata_a");

    assert_eq!(before - after, DEPOSIT, "maker A balance should drop by DEPOSIT");
    assert_eq!(ctx.svm.token_balance(&accs.vault), Some(DEPOSIT));

    let escrow: Escrow = ctx.load(&accs.escrow);
    assert_eq!(escrow.seed, SEED);
    assert_eq!(escrow.maker, accs.bundle.maker);
    assert_eq!(escrow.mint_a, accs.bundle.mint_a);
    assert_eq!(escrow.mint_b, accs.bundle.mint_b);
    assert_eq!(escrow.amount, RECEIVE);
    assert!(escrow.expiry_utc.is_none());
}

#[test]
fn make_rejects_past_expiry() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);

    // Anchor's check is `now < expiry`; passing `now` itself trips it.
    let past = ctx.svm.get_unix_timestamp();
    let (ix, _) = make_ix_with(&ctx, &accs, SEED, RECEIVE, DEPOSIT, Some(past));
    ctx.svm.send_anchor_err(ix, &[&accs.maker], "ExpirationDateTooOld");

    assert!(
        ctx.svm.get_account(&accs.escrow).is_none(),
        "escrow must not be created when make fails"
    );
}

#[test]
fn deposit_and_amount_can_differ() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);

    // Vault holds what the maker put in (`deposit`); the escrow records what
    // the maker wants back (`amount`). The two are independent fields.
    let deposit = 1_500_000;
    let amount = 4_242_424_242;
    let (ix, _) = make_ix_with(&ctx, &accs, SEED, amount, deposit, None);
    ctx.svm.send_ok(ix, &[&accs.maker]);

    assert_eq!(ctx.svm.token_balance(&accs.vault), Some(deposit));
    let escrow: Escrow = ctx.load(&accs.escrow);
    assert_eq!(escrow.amount, amount);
}

#[test]
fn two_concurrent_escrows_use_distinct_seeds() {
    let mut ctx =
        AnchorLiteSVM::build_with_program(ID, include_bytes!("../../../target/deploy/escrow.so"));
    let accs = setup_accounts(&mut ctx);

    let (ix_a, bundle_a) = make_ix_with(&ctx, &accs, 100, RECEIVE, DEPOSIT, None);
    let (ix_b, bundle_b) = make_ix_with(&ctx, &accs, 101, RECEIVE, DEPOSIT, None);

    assert_ne!(bundle_a.escrow, bundle_b.escrow);
    assert_ne!(bundle_a.vault, bundle_b.vault);

    ctx.svm.send_ok(ix_a, &[&accs.maker]);
    ctx.svm.send_ok(ix_b, &[&accs.maker]);

    // Both escrows live independently; refund one and the other survives.
    let refund_a = ctx.program().build_ix(bundle_a, instruction::Refund {});
    ctx.svm.send_ok(refund_a, &[&accs.maker]);

    assert!(ctx.svm.get_account(&bundle_a.escrow).is_none());
    assert!(ctx.svm.get_account(&bundle_b.escrow).is_some());

    // Sanity: total mint_a movement is `2 * DEPOSIT - DEPOSIT = DEPOSIT`,
    // so the maker is down exactly one deposit relative to FUNDED_A.
    assert_eq!(ctx.svm.token_balance(&accs.maker_ata_a), Some(FUNDED_A - DEPOSIT));
}
