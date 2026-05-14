mod utils;

use {
    utils::{
        current_unix_timestamp, escrow_state, send_instruction, set_unix_timestamp, setup,
        token_balance, try_send_instruction, EscrowAccounts, DEFAULT_OFFER_AMOUNT, MINT_AMOUNT,
    },
};

#[test]
fn make_with_future_expiry_persists_timestamp() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 10);

    let now = current_unix_timestamp(&svm);
    let expiry = now.checked_add(3_600).unwrap();

    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, Some(expiry)),
    );

    let escrow = escrow_state(&svm, &accounts.escrow);
    assert_eq!(escrow.expiry_utc, Some(expiry));
    assert_eq!(token_balance(&svm, &accounts.vault), DEFAULT_OFFER_AMOUNT);
}

#[test]
fn make_rejects_past_expiry() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 11);

    set_unix_timestamp(&mut svm, 1_000_000);

    let past = 500_000;
    let result = try_send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, Some(past)),
    );

    assert!(result.is_err());
    assert!(svm.get_account(&accounts.escrow).is_none());
}

#[test]
fn take_succeeds_before_expiry() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 12);

    let expiry = current_unix_timestamp(&svm).checked_add(3_600).unwrap();
    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, Some(expiry)),
    );

    send_instruction(&mut svm, &taker_authority, accounts.take_ix());

    assert!(svm.get_account(&accounts.escrow).is_none());
    assert!(svm.get_account(&accounts.vault).is_none());
    assert_eq!(
        token_balance(&svm, &accounts.taker_ata_a),
        DEFAULT_OFFER_AMOUNT
    );
    assert_eq!(
        token_balance(&svm, &accounts.maker_ata_b),
        DEFAULT_OFFER_AMOUNT
    );
}

#[test]
fn take_fails_after_expiry() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 13);

    let expiry = current_unix_timestamp(&svm).checked_add(3_600).unwrap();
    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, Some(expiry)),
    );

    set_unix_timestamp(&mut svm, expiry.checked_add(1).unwrap());

    let result = try_send_instruction(&mut svm, &taker_authority, accounts.take_ix());
    assert!(result.is_err());

    assert!(svm.get_account(&accounts.escrow).is_some());
    assert_eq!(token_balance(&svm, &accounts.vault), DEFAULT_OFFER_AMOUNT);
}

#[test]
fn refund_works_after_expiry() {
    let (mut svm, maker_authority, taker_authority) = setup();
    let accounts = EscrowAccounts::new(&mut svm, &maker_authority, &taker_authority, 14);

    let expiry = current_unix_timestamp(&svm).checked_add(3_600).unwrap();
    send_instruction(
        &mut svm,
        &maker_authority,
        accounts.make_ix(DEFAULT_OFFER_AMOUNT, DEFAULT_OFFER_AMOUNT, Some(expiry)),
    );

    set_unix_timestamp(&mut svm, expiry.checked_add(1_000).unwrap());

    send_instruction(&mut svm, &maker_authority, accounts.refund_ix());

    assert!(svm.get_account(&accounts.escrow).is_none());
    assert!(svm.get_account(&accounts.vault).is_none());
    assert_eq!(token_balance(&svm, &accounts.maker_ata_a), MINT_AMOUNT);
}
