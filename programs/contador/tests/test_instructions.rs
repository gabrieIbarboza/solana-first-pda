use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

// ── constants ─────────────────────────────────────────────────────────────────

const AIRDROP_AMOUNT: u64 = 1_000_000_000; // 1 SOL in lamports
const INITIAL_COUNT: u64 = 0;
const COUNT_AFTER_ONE_INCREMENT: u64 = 1;

// ── helpers ───────────────────────────────────────────────────────────────────
fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey) {
    let payer    = Keypair::new();
    let stranger = Keypair::new();
    let program_id = contador::id();

    let counter = Pubkey::find_program_address(
        &[contador::constants::COUNTER_SEED],
        &program_id,
    ).0;

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/contador.so"));
    svm.add_program(program_id, bytes).unwrap();

    svm.airdrop(&payer.pubkey(),    AIRDROP_AMOUNT).unwrap();
    svm.airdrop(&stranger.pubkey(), AIRDROP_AMOUNT).unwrap();

    (svm, payer, stranger, counter)
}

/// Reads and deserializes the on-chain counter account.
fn get_counter_state(svm: &LiteSVM, counter: &Pubkey) -> contador::state::counter::Counter {
    let account = svm.get_account(counter).unwrap();
    let mut data: &[u8] = &account.data;
    contador::state::counter::Counter::try_deserialize(&mut data).unwrap()
}

/// Builds the Initialize instruction.
fn initialize_ix(payer: &Keypair, counter: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        contador::id(),
        &contador::instruction::Initialize {}.data(),
        contador::accounts::Initialize {
            payer: payer.pubkey(),
            counter,
            system_program: system_program::ID,
        }.to_account_metas(None),
    )
}

/// Builds the Increment instruction for a given authority.
fn increment_ix(authority: &Keypair, counter: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        contador::id(),
        &contador::instruction::Increment {}.data(),
        contador::accounts::Increment {
            counter,
            authority: authority.pubkey(),
        }.to_account_metas(None),
    )
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_success() {
    let (mut svm, payer, _stranger, counter) = setup();

    let ix = initialize_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Initialize should succeed");

    let state = get_counter_state(&svm, &counter);
    assert_eq!(state.count,     INITIAL_COUNT,  "Count should start at 0");
    assert_eq!(state.authority, payer.pubkey(), "Authority should be the payer");
}

#[test]
fn test_initialize_twice_fails() {
    let (mut svm, payer, _stranger, counter) = setup();

    // First initialize — must succeed
    let ix = initialize_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Second initialize — must fail (account already exists)
    let ix = initialize_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Initializing the same counter twice should fail");
}

#[test]
fn test_increment_success() {
    let (mut svm, payer, _stranger, counter) = setup();

    // Initialize first
    let ix = initialize_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Now increment
    let ix = increment_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Increment by authority should succeed");

    let state = get_counter_state(&svm, &counter);
    assert_eq!(state.count, COUNT_AFTER_ONE_INCREMENT, "Count should be 1 after one increment");
}

#[test]
fn test_increment_unauthorized_fails() {
    let (mut svm, payer, stranger, counter) = setup();

    // Payer initializes (and becomes the authority)
    let ix = initialize_ix(&payer, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Stranger tries to increment — should be rejected
    let ix = increment_ix(&stranger, counter);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&stranger.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&stranger]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Increment by a stranger should fail");

    // Counter must remain untouched
    let state = get_counter_state(&svm, &counter);
    assert_eq!(state.count, INITIAL_COUNT, "Count should still be 0 after failed increment");
}