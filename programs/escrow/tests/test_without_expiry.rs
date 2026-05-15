mod utils;

use {
    utils::{
        escrow_state, send_instruction, setup, token_balance, try_send_instruction, EscrowAccounts,
        DEFAULT_OFFER_AMOUNT, MINT_AMOUNT,
    },
};

#[test]
fn make_locks_tokens_in_vault_and_initialises_escrow() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 1);

    let maker_a_before = token_balance(&svm, &accounts.maker_ata_a);

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );

    let maker_a_after = token_balance(&svm, &accounts.maker_ata_a);
    assert_eq!(maker_a_before - maker_a_after, DEFAULT_OFFER_AMOUNT);
    assert_eq!(token_balance(&svm, &accounts.vault), DEFAULT_OFFER_AMOUNT);

    let escrow = escrow_state(&svm, &accounts.escrow);
    assert_eq!(escrow.seed, accounts.seed);
    assert_eq!(escrow.maker, accounts.maker);
    assert_eq!(escrow.mint_a, accounts.mint_a);
    assert_eq!(escrow.mint_b, accounts.mint_b);
    assert_eq!(escrow.amount, DEFAULT_OFFER_AMOUNT);
    assert!(escrow.expiry_utc.is_none());
}

#[test]
fn deposit_and_amount_can_differ() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 2);

    let deposit = 7_000_000;
    let amount = 3_000_000;

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(amount, deposit, None),
    );

    assert_eq!(token_balance(&svm, &accounts.vault), deposit);
    assert_eq!(escrow_state(&svm, &accounts.escrow).amount, amount);
}

#[test]
fn take_swaps_balances_and_closes_state() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 3);

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );

    let taker_b_before = token_balance(&svm, &accounts.taker_ata_b);

    send_instruction(&mut svm, &taker_authority, accounts.take_ix());

    assert_eq!(
        taker_b_before - token_balance(&svm, &accounts.taker_ata_b),
        DEFAULT_OFFER_AMOUNT
    );
    assert_eq!(
        token_balance(&svm, &accounts.maker_ata_b),
        DEFAULT_OFFER_AMOUNT
    );

    assert_eq!(
        token_balance(&svm, &accounts.taker_ata_a),
        DEFAULT_OFFER_AMOUNT
    );

    assert!(svm.get_account(&accounts.escrow).is_none());
    assert!(svm.get_account(&accounts.vault).is_none());
}

#[test]
fn refund_returns_vault_and_closes_state() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 4);

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );

    send_instruction(&mut svm, &maker_authority, accounts.refund_ix());

    assert!(svm.get_account(&accounts.escrow).is_none());
    assert!(svm.get_account(&accounts.vault).is_none());

    assert_eq!(token_balance(&svm, &accounts.maker_ata_a), MINT_AMOUNT);
}

#[test]
fn take_drains_vault_when_deposit_differs_from_amount() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 200);

    let deposit = 10_000_000;
    let amount = 4_000_000;

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(amount, deposit, None),
    );
    assert_eq!(token_balance(&svm, &accounts.vault), deposit);

    let taker_b_before = token_balance(&svm, &accounts.taker_ata_b);

    send_instruction(&mut svm, &taker_authority, accounts.take_ix());

    assert!(svm.get_account(&accounts.escrow).is_none());
    assert!(svm.get_account(&accounts.vault).is_none());

    assert_eq!(
        taker_b_before - token_balance(&svm, &accounts.taker_ata_b),
        amount
    );
    assert_eq!(token_balance(&svm, &accounts.taker_ata_a), deposit);
    assert_eq!(token_balance(&svm, &accounts.maker_ata_b), amount);
}

#[test]
fn two_concurrent_escrows_use_distinct_seeds() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let a = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 100);
    let b = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 101);

    assert_ne!(a.escrow, b.escrow);
    assert_ne!(a.vault, b.vault);

    send_instruction(
        &mut svm,
        &maker_authority,
        a.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );
    send_instruction(
        &mut svm,
        &maker_authority,
        b.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );

    send_instruction(&mut svm, &maker_authority, a.refund_ix());
    assert!(svm.get_account(&a.escrow).is_none());
    assert!(svm.get_account(&b.escrow).is_some());
}

#[test]
fn refund_signer_must_match_escrow_maker() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 5);

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, None),
    );

    let refund_ix = accounts.refund_ix_with_maker(accounts.taker, accounts.taker_ata_a);
    let result = try_send_instruction(&mut svm, &taker_authority, refund_ix);
    assert!(result.is_err(), "taker should not be able to refund");
    assert!(svm.get_account(&accounts.escrow).is_some());
}
