use anchor_lang::{InstructionData, ToAccountMetas};
use fixtures::test::TestFixture;
use astrolend::state::astrolend_account::AstrolendAccount;
use solana_program_test::tokio;
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, system_program,
    transaction::Transaction,
};

#[tokio::test]
async fn astrolend_account_create_success() -> anyhow::Result<()> {
    let test_f = TestFixture::new(None).await;

    let astrolend_account_key = Keypair::new();
    let accounts = astrolend::accounts::AstrolendAccountInitialize {
        astrolend_group: test_f.astrolend_group.key,
        astrolend_account: astrolend_account_key.pubkey(),
        authority: test_f.payer(),
        fee_payer: test_f.payer(),
        system_program: system_program::id(),
    };
    let init_astrolend_account_ix = Instruction {
        program_id: astrolend::id(),
        accounts: accounts.to_account_metas(Some(true)),
        data: astrolend::instruction::AstrolendAccountInitialize {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_astrolend_account_ix],
        Some(&test_f.payer()),
        &[&test_f.payer_keypair(), &astrolend_account_key],
        test_f.get_latest_blockhash().await,
    );

    let res = test_f
        .context
        .borrow_mut()
        .banks_client
        .process_transaction(tx)
        .await;

    assert!(res.is_ok());

    let astrolend_account: AstrolendAccount = test_f
        .load_and_deserialize(&astrolend_account_key.pubkey())
        .await;

    assert_eq!(astrolend_account.group, test_f.astrolend_group.key);
    assert_eq!(astrolend_account.authority, test_f.payer());
    assert!(astrolend_account
        .lending_account
        .balances
        .iter()
        .all(|bank| !bank.active));

    Ok(())
}
