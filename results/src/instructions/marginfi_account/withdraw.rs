use crate::{
    bank_signer, check,
    constants::{LIQUIDITY_VAULT_AUTHORITY_SEED, LIQUIDITY_VAULT_SEED},
    events::{AccountEventHeader, LendingAccountWithdrawEvent},
    prelude::*,
    state::{
        astrolend_account::{BankAccountWrapper, AstrolendAccount, RiskEngine, DISABLED_FLAG},
        astrolend_group::{Bank, BankVaultType},
    },
    utils,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use fixed::types::I80F48;
use solana_program::{clock::Clock, sysvar::Sysvar};

/// 1. Accrue interest
/// 2. Find the user's existing bank account for the asset withdrawn
/// 3. Record asset decrease in the bank account
/// 4. Transfer funds from the bank's liquidity vault to the signer's token account
/// 5. Verify that the user account is in a healthy state
///
/// Will error if there is no existing asset <=> borrowing is not allowed.
pub fn lending_account_withdraw<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, LendingAccountWithdraw<'info>>,
    amount: u64,
    withdraw_all: Option<bool>,
) -> AstrolendResult {
    let LendingAccountWithdraw {
        astrolend_account: astrolend_account_loader,
        destination_token_account,
        bank_liquidity_vault,
        token_program,
        bank_liquidity_vault_authority,
        bank: bank_loader,
        ..
    } = ctx.accounts;
    let clock = Clock::get()?;

    let withdraw_all = withdraw_all.unwrap_or(false);
    let mut astrolend_account = astrolend_account_loader.load_mut()?;

    check!(
        !astrolend_account.get_flag(DISABLED_FLAG),
        AstrolendError::AccountDisabled
    );

    let maybe_bank_mint = utils::maybe_take_bank_mint(
        &mut ctx.remaining_accounts,
        &*bank_loader.load()?,
        token_program.key,
    )?;

    bank_loader.load_mut()?.accrue_interest(
        clock.unix_timestamp,
        #[cfg(not(feature = "client"))]
        bank_loader.key(),
    )?;

    {
        let mut bank = bank_loader.load_mut()?;

        let liquidity_vault_authority_bump = bank.liquidity_vault_authority_bump;

        let mut bank_account = BankAccountWrapper::find(
            &bank_loader.key(),
            &mut bank,
            &mut astrolend_account.lending_account,
        )?;

        let amount_pre_fee = if withdraw_all {
            bank_account.withdraw_all()?
        } else {
            let amount_pre_fee = maybe_bank_mint
                .as_ref()
                .map(|mint| {
                    utils::calculate_pre_fee_spl_deposit_amount(
                        mint.to_account_info(),
                        amount,
                        clock.epoch,
                    )
                })
                .transpose()?
                .unwrap_or(amount);

            bank_account.withdraw(I80F48::from_num(amount_pre_fee))?;

            amount_pre_fee
        };

        bank_account.withdraw_spl_transfer(
            amount_pre_fee,
            bank_liquidity_vault.to_account_info(),
            destination_token_account.to_account_info(),
            bank_liquidity_vault_authority.to_account_info(),
            maybe_bank_mint.as_ref(),
            token_program.to_account_info(),
            bank_signer!(
                BankVaultType::Liquidity,
                bank_loader.key(),
                liquidity_vault_authority_bump
            ),
            ctx.remaining_accounts,
        )?;

        emit!(LendingAccountWithdrawEvent {
            header: AccountEventHeader {
                signer: Some(ctx.accounts.signer.key()),
                astrolend_account: astrolend_account_loader.key(),
                astrolend_account_authority: astrolend_account.authority,
                astrolend_group: astrolend_account.group,
            },
            bank: bank_loader.key(),
            mint: bank.mint,
            amount: amount_pre_fee,
            close_balance: withdraw_all,
        });
    }

    // Check account health, if below threshold fail transaction
    // Assuming `ctx.remaining_accounts` holds only oracle accounts
    RiskEngine::check_account_init_health(&astrolend_account, ctx.remaining_accounts)?;

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountWithdraw<'info> {
    pub astrolend_group: AccountLoader<'info, AstrolendGroup>,

    #[account(
        mut,
        constraint = astrolend_account.load()?.group == astrolend_group.key(),
    )]
    pub astrolend_account: AccountLoader<'info, AstrolendAccount>,

    #[account(
        address = astrolend_account.load()?.authority,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = bank.load()?.group == astrolend_group.key(),
    )]
    pub bank: AccountLoader<'info, Bank>,

    #[account(mut)]
    pub destination_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Seed constraint check
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_AUTHORITY_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_authority_bump,
    )]
    pub bank_liquidity_vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_bump,
    )]
    pub bank_liquidity_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
