use fixtures::{
    assert_custom_error,
    spl::TokenAccountFixture,
    test::{BankMint, TestFixture, TestSettings},
};
use astrolend::errors::AstrolendError;
use solana_program_test::tokio;

#[tokio::test]
async fn close_astrolend_account() -> anyhow::Result<()> {
    let mut test_f: TestFixture =
        TestFixture::new(Some(TestSettings::all_banks_payer_not_admin())).await;

    let astrolend_account_f = test_f.create_astrolend_account().await;

    let owner = test_f.payer();
    let token_account_f =
        TokenAccountFixture::new(test_f.context.clone(), &test_f.usdc_mint, &owner).await;
    test_f.usdc_mint.mint_to(&token_account_f.key, 1_000).await;

    let usdc_bank_f = test_f.get_bank(&BankMint::Usdc);

    astrolend_account_f
        .try_bank_deposit(token_account_f.key, usdc_bank_f, 1_000)
        .await?;

    let res = astrolend_account_f.try_close_account(0).await;

    assert!(res.is_err());
    assert_custom_error!(res.unwrap_err(), AstrolendError::IllegalAction);

    let sol_bank_f = test_f.get_bank(&BankMint::Sol);
    let sol_account = test_f.sol_mint.create_token_account_and_mint_to(100).await;
    let depositor = test_f.create_astrolend_account().await;
    depositor
        .try_bank_deposit(sol_account.key, sol_bank_f, 100)
        .await?;

    let sol_account_2 = test_f.sol_mint.create_token_account_and_mint_to(0).await;

    astrolend_account_f
        .try_bank_borrow(sol_account_2.key, sol_bank_f, 10)
        .await?;

    let res = astrolend_account_f.try_close_account(0).await;

    assert!(res.is_err());
    assert_custom_error!(res.unwrap_err(), AstrolendError::IllegalAction);

    // Repay the loan
    astrolend_account_f
        .try_bank_repay(sol_account_2.key, sol_bank_f, 10, Some(true))
        .await?;

    astrolend_account_f
        .try_bank_withdraw(token_account_f.key, usdc_bank_f, 1_000, Some(true))
        .await?;

    let res = astrolend_account_f.try_close_account(1).await;

    assert!(res.is_ok());

    Ok(())
}
