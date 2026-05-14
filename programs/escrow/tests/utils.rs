use {
    anchor_lang::{
        prelude::*, solana_program::instruction::Instruction,
        system_program::ID as SYSTEM_PROGRAM_ID, InstructionData, ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
        token::ID as TOKEN_PROGRAM_ID,
    },
    escrow::{program_pack::Pack, ESCROW_SEED},
    litesvm::{types::TransactionResult, LiteSVM},
    litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo},
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

pub const INITIAL_USER_LAMPORTS: u64 = 2_000_000_000;
pub const MINT_AMOUNT: u64 = 1_000_000_000;
pub const DEFAULT_OFFER_AMOUNT: u64 = 10_000_000;
pub const MINT_DECIMALS: u8 = 6;

pub fn setup() -> (LiteSVM, Keypair, Keypair) {
    let maker_authority = Keypair::new();
    let taker_authority = Keypair::new();

    let bytes = include_bytes!("../../../target/deploy/escrow.so");

    let mut svm = LiteSVM::new();
    svm.add_program(escrow::id(), bytes).unwrap();

    svm.airdrop(&maker_authority.pubkey(), INITIAL_USER_LAMPORTS)
        .unwrap();
    svm.airdrop(&taker_authority.pubkey(), INITIAL_USER_LAMPORTS)
        .unwrap();

    (svm, maker_authority, taker_authority)
}

pub fn send_instruction(svm: &mut LiteSVM, user_authority: &Keypair, instruction: Instruction) {
    try_send_instruction(svm, user_authority, instruction).unwrap();
}

pub fn try_send_instruction(
    svm: &mut LiteSVM,
    user_authority: &Keypair,
    instruction: Instruction,
) -> TransactionResult {
    let message = Message::new(&[instruction], Some(&user_authority.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[user_authority], message, recent_blockhash);
    svm.send_transaction(transaction)
}

pub struct EscrowAccounts {
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker_ata_a: Pubkey,
    pub maker_ata_b: Pubkey,
    pub taker_ata_a: Pubkey,
    pub taker_ata_b: Pubkey,
    pub seed: u64,
    pub escrow: Pubkey,
    pub vault: Pubkey,
}

impl EscrowAccounts {
    pub fn new(
        svm: &mut LiteSVM,
        maker_authority: &Keypair,
        taker_authority: &Keypair,
        seed: u64,
    ) -> Self {
        let maker = maker_authority.pubkey();
        let taker = taker_authority.pubkey();

        let mint_a = CreateMint::new(svm, maker_authority)
            .decimals(MINT_DECIMALS)
            .authority(&maker)
            .send()
            .unwrap();

        let mint_b = CreateMint::new(svm, taker_authority)
            .decimals(MINT_DECIMALS)
            .authority(&taker)
            .send()
            .unwrap();

        let maker_ata_a = CreateAssociatedTokenAccount::new(svm, maker_authority, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(svm, maker_authority, &mint_b)
            .owner(&maker)
            .send()
            .unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(svm, taker_authority, &mint_a)
            .owner(&taker)
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(svm, taker_authority, &mint_b)
            .owner(&taker)
            .send()
            .unwrap();

        MintTo::new(svm, maker_authority, &mint_a, &maker_ata_a, MINT_AMOUNT)
            .send()
            .unwrap();
        MintTo::new(svm, taker_authority, &mint_b, &taker_ata_b, MINT_AMOUNT)
            .send()
            .unwrap();

        let (escrow, _bump) = Pubkey::find_program_address(
            &[ESCROW_SEED, maker.as_ref(), &seed.to_le_bytes()],
            &escrow::id(),
        );
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

        Self {
            maker,
            taker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            seed,
            escrow,
            vault,
        }
    }

    pub fn make_ix(&self, amount: u64, deposit: u64, expiry_utc: Option<i64>) -> Instruction {
        Instruction {
            program_id: escrow::id(),
            accounts: escrow::accounts::Make {
                escrow: self.escrow,
                maker: self.maker,
                maker_ata_a: self.maker_ata_a,
                mint_a: self.mint_a,
                mint_b: self.mint_b,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: escrow::instruction::Make {
                amount,
                deposit,
                seed: self.seed,
                expiry_utc,
            }
            .data(),
        }
    }

    pub fn take_ix(&self) -> Instruction {
        Instruction {
            program_id: escrow::id(),
            accounts: escrow::accounts::Take {
                escrow: self.escrow,
                maker: self.maker,
                mint_a: self.mint_a,
                mint_b: self.mint_b,
                vault: self.vault,
                maker_ata_b: self.maker_ata_b,
                taker: self.taker,
                taker_ata_a: self.taker_ata_a,
                taker_ata_b: self.taker_ata_b,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: escrow::instruction::Take {}.data(),
        }
    }

    pub fn refund_ix(&self) -> Instruction {
        Instruction {
            program_id: escrow::id(),
            accounts: escrow::accounts::Refund {
                escrow: self.escrow,
                maker: self.maker,
                maker_ata_a: self.maker_ata_a,
                mint_a: self.mint_a,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: escrow::instruction::Refund {}.data(),
        }
    }


    pub fn refund_ix_with_maker(&self, maker: Pubkey, maker_ata_a: Pubkey) -> Instruction {
        Instruction {
            program_id: escrow::id(),
            accounts: escrow::accounts::Refund {
                escrow: self.escrow,
                maker,
                maker_ata_a,
                mint_a: self.mint_a,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: escrow::instruction::Refund {}.data(),
        }
    }
}

pub fn token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    spl_token::state::Account::unpack(&svm.get_account(ata).unwrap().data)
        .unwrap()
        .amount
}

pub fn escrow_state(svm: &LiteSVM, escrow: &Pubkey) -> escrow::state::Escrow {
    let account = svm.get_account(escrow).unwrap();
    escrow::state::Escrow::try_deserialize(&mut account.data.as_ref()).unwrap()
}

pub fn current_unix_timestamp(svm: &LiteSVM) -> i64 {
    svm.get_sysvar::<Clock>().unix_timestamp
}

pub fn set_unix_timestamp(svm: &mut LiteSVM, new_ts: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = new_ts;
    svm.set_sysvar::<Clock>(&clock);
}
