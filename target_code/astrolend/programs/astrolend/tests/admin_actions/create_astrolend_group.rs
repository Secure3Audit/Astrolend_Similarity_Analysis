use anchor_lang::{InstructionData, ToAccountMetas};
use fixtures::prelude::*;
use astrolend::prelude::AstrolendGroup;
use pretty_assertions::assert_eq;
use solana_program::{instruction::Instruction, system_program};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn astrolend_group_create_success() -> anyhow::Result<()> {
    let test_f = TestFixture::new(None).await;

    // Create & initialize astrolend group
    let astrolend_group_key = Keypair::new();

    let accounts = astrolend::accounts::AstrolendGroupInitialize {
        astrolend_group: astrolend_group_key.pubkey(),
        admin: test_f.payer(),
        system_program: system_program::id(),
    };
    let init_astrolend_group_ix = Instruction {
        program_id: astrolend::id(),
        accounts: accounts.to_account_metas(Some(true)),
        data: astrolend::instruction::AstrolendGroupInitialize {}.data(),
    };
    let tx = Transaction::new_signed_with_payer(
        &[init_astrolend_group_ix],
        Some(&test_f.payer().clone()),
        &[&test_f.payer_keypair(), &astrolend_group_key],
        test_f.get_latest_blockhash().await,
    );
    let res = test_f
        .context
        .borrow_mut()
        .banks_client
        .process_transaction(tx)
        .await;

    assert!(res.is_ok());

    // Fetch & deserialize astrolend group account
    let astrolend_group: AstrolendGroup = test_f
        .load_and_deserialize(&astrolend_group_key.pubkey())
        .await;

    // Check basic properties
    assert_eq!(astrolend_group.admin, test_f.payer());

    Ok(())
}
