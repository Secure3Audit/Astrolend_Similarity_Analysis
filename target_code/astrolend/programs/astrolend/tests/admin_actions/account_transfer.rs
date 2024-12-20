use fixtures::{assert_custom_error, test::TestFixture};
use astrolend::{errors::AstrolendError, state::astrolend_account::TRANSFER_AUTHORITY_ALLOWED_FLAG};
use solana_program_test::tokio;
use solana_sdk::{signature::Keypair, signer::Signer};

// Test transfer account authority.
// No transfer flag set -- tx should fail.
// Set the flag and try again -- tx should succeed.
// RUST_BACKTRACE=1 cargo test-bpf astrolend_account_authority_transfer_no_flag_set -- --exact
#[tokio::test]
async fn astrolend_account_authority_transfer_no_flag_set() -> anyhow::Result<()> {
    let test_f = TestFixture::new(None).await;
    // Default account with no flags set
    let astrolend_account = test_f.create_astrolend_account().await;
    let new_authority = Keypair::new().pubkey();

    let res = astrolend_account
        .try_transfer_account_authority(new_authority, None)
        .await;

    // Check transfer authority is unchanged
    let account = astrolend_account.load().await;
    assert_eq!(account.authority, test_f.payer());

    // Assert the response is an error due to the lack of the correct flag
    assert!(res.is_err());
    assert_custom_error!(
        res.unwrap_err(),
        AstrolendError::IllegalAccountAuthorityTransfer
    );

    // set the flag on the account
    astrolend_account
        .try_set_flag(TRANSFER_AUTHORITY_ALLOWED_FLAG)
        .await
        .unwrap();

    // Check transfer authority flag
    let account = astrolend_account.load().await;
    assert!(account.get_flag(TRANSFER_AUTHORITY_ALLOWED_FLAG));

    let new_authority_2 = Keypair::new().pubkey();
    let res = astrolend_account
        .try_transfer_account_authority(new_authority_2, None)
        .await;

    assert!(res.is_ok());

    // Check transfer authority
    let account = astrolend_account.load().await;
    assert_eq!(account.authority, new_authority_2);

    Ok(())
}

#[tokio::test]
async fn astrolend_account_authority_transfer_not_account_owner() -> anyhow::Result<()> {
    let test_f = TestFixture::new(None).await;
    let astrolend_account = test_f.create_astrolend_account().await;
    let new_authority = Keypair::new().pubkey();
    let signer = Keypair::new();

    let res = astrolend_account
        .try_transfer_account_authority(new_authority, Some(signer))
        .await;

    // Assert the response is an error due to fact that a non-owner of the
    // acount attempted to initialize this account transfer
    assert!(res.is_err());

    Ok(())
}
