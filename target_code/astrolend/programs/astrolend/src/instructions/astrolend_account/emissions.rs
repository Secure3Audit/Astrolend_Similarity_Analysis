use anchor_lang::{prelude::*, Accounts, ToAccountInfo};
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    check,
    constants::{EMISSIONS_AUTH_SEED, EMISSIONS_TOKEN_ACCOUNT_SEED},
    debug,
    prelude::{AstrolendError, AstrolendResult},
    state::{
        astrolend_account::{BankAccountWrapper, AstrolendAccount, DISABLED_FLAG},
        astrolend_group::{Bank, AstrolendGroup},
    },
};

pub fn lending_account_withdraw_emissions<'info>(
    ctx: Context<'_, '_, 'info, 'info, LendingAccountWithdrawEmissions<'info>>,
) -> AstrolendResult {
    let mut astrolend_account = ctx.accounts.astrolend_account.load_mut()?;

    check!(
        !astrolend_account.get_flag(DISABLED_FLAG),
        AstrolendError::AccountDisabled
    );

    let mut bank = ctx.accounts.bank.load_mut()?;

    let mut balance = BankAccountWrapper::find(
        ctx.accounts.bank.to_account_info().key,
        &mut bank,
        &mut astrolend_account.lending_account,
    )?;

    // Settle emissions
    let emissions_settle_amount = balance.settle_emissions_and_get_transfer_amount()?;

    if emissions_settle_amount > 0 {
        debug!("Transferring {} emissions to user", emissions_settle_amount);

        let signer_seeds: &[&[&[u8]]] = &[&[
            EMISSIONS_AUTH_SEED.as_bytes(),
            &ctx.accounts.bank.key().to_bytes(),
            &ctx.accounts.emissions_mint.key().to_bytes(),
            &[ctx.bumps.emissions_auth],
        ]];

        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.emissions_vault.to_account_info(),
                    to: ctx.accounts.destination_account.to_account_info(),
                    authority: ctx.accounts.emissions_auth.to_account_info(),
                    mint: ctx.accounts.emissions_mint.to_account_info(),
                },
                signer_seeds,
            ),
            emissions_settle_amount,
            ctx.accounts.emissions_mint.decimals,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountWithdrawEmissions<'info> {
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

    #[account(
        address = bank.load()?.emissions_mint
    )]
    pub emissions_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [
            EMISSIONS_AUTH_SEED.as_bytes(),
            bank.key().as_ref(),
            emissions_mint.key().as_ref(),
        ],
        bump
    )]
    /// CHECK: Asserted by PDA
    pub emissions_auth: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            EMISSIONS_TOKEN_ACCOUNT_SEED.as_bytes(),
            bank.key().as_ref(),
            emissions_mint.key().as_ref(),
        ],
        bump,
    )]
    pub emissions_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub destination_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// Permissionlessly settle unclaimed emissions to a users account.
pub fn lending_account_settle_emissions(
    ctx: Context<LendingAccountSettleEmissions>,
) -> AstrolendResult {
    let mut astrolend_account = ctx.accounts.astrolend_account.load_mut()?;
    let mut bank = ctx.accounts.bank.load_mut()?;

    let mut balance = BankAccountWrapper::find(
        ctx.accounts.bank.to_account_info().key,
        &mut bank,
        &mut astrolend_account.lending_account,
    )?;

    balance.claim_emissions(Clock::get()?.unix_timestamp.try_into().unwrap())?;

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountSettleEmissions<'info> {
    #[account(
        mut,
        constraint = astrolend_account.load()?.group == bank.load()?.group,
    )]
    pub astrolend_account: AccountLoader<'info, AstrolendAccount>,

    #[account(mut)]
    pub bank: AccountLoader<'info, Bank>,
}
