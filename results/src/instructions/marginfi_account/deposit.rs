use crate::{
    check,
    constants::LIQUIDITY_VAULT_SEED,
    events::{AccountEventHeader, LendingAccountDepositEvent},
    prelude::*,
    state::{
        astrolend_account::{BankAccountWrapper, AstrolendAccount, DISABLED_FLAG},
        astrolend_group::Bank,
    },
    utils,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use fixed::types::I80F48;
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

/// 1. Accrue interest
/// 2. Create the user's bank account for the asset deposited if it does not exist yet
/// 3. Record asset increase in the bank account
/// 4. Transfer funds from the signer's token account to the bank's liquidity vault
///
/// Will error if there is an existing liability <=> repaying is not allowed.
pub fn lending_account_deposit<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, LendingAccountDeposit<'info>>,
    amount: u64,
) -> AstrolendResult {
    let LendingAccountDeposit {
        astrolend_account: astrolend_account_loader,
        signer,
        signer_token_account,
        bank_liquidity_vault,
        token_program,
        bank: bank_loader,
        ..
    } = ctx.accounts;
    let clock = Clock::get()?;
    let maybe_bank_mint = utils::maybe_take_bank_mint(
        &mut ctx.remaining_accounts,
        &*bank_loader.load()?,
        token_program.key,
    )?;

    let mut bank = bank_loader.load_mut()?;
    let mut astrolend_account = astrolend_account_loader.load_mut()?;

    check!(
        !astrolend_account.get_flag(DISABLED_FLAG),
        AstrolendError::AccountDisabled
    );

    bank.accrue_interest(
        clock.unix_timestamp,
        #[cfg(not(feature = "client"))]
        bank_loader.key(),
    )?;

    let mut bank_account = BankAccountWrapper::find_or_create(
        &bank_loader.key(),
        &mut bank,
        &mut astrolend_account.lending_account,
    )?;

    bank_account.deposit(I80F48::from_num(amount))?;

    let amount_pre_fee = maybe_bank_mint
        .as_ref()
        .map(|mint| {
            utils::calculate_pre_fee_spl_deposit_amount(mint.to_account_info(), amount, clock.epoch)
        })
        .transpose()?
        .unwrap_or(amount);

    bank_account.deposit_spl_transfer(
        amount_pre_fee,
        signer_token_account.to_account_info(),
        bank_liquidity_vault.to_account_info(),
        signer.to_account_info(),
        maybe_bank_mint.as_ref(),
        token_program.to_account_info(),
        ctx.remaining_accounts,
    )?;

    emit!(LendingAccountDepositEvent {
        header: AccountEventHeader {
            signer: Some(signer.key()),
            astrolend_account: astrolend_account_loader.key(),
            astrolend_account_authority: astrolend_account.authority,
            astrolend_group: astrolend_account.group,
        },
        bank: bank_loader.key(),
        mint: bank.mint,
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountDeposit<'info> {
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

    /// CHECK: Token mint/authority are checked at transfer
    #[account(mut)]
    pub signer_token_account: AccountInfo<'info>,

    /// CHECK: Seed constraint check
    #[account(
        mut,
        seeds = [
            LIQUIDITY_VAULT_SEED.as_bytes(),
            bank.key().as_ref(),
        ],
        bump = bank.load()?.liquidity_vault_bump,
    )]
    pub bank_liquidity_vault: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}
