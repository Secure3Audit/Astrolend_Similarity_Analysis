use anchor_lang::prelude::*;

use crate::{
    check,
    prelude::*,
    state::{
        astrolend_account::{BankAccountWrapper, AstrolendAccount, DISABLED_FLAG},
        astrolend_group::Bank,
    },
};

pub fn lending_account_close_balance(ctx: Context<LendingAccountCloseBalance>) -> AstrolendResult {
    let LendingAccountCloseBalance {
        astrolend_account,
        bank: bank_loader,
        ..
    } = ctx.accounts;

    let mut astrolend_account = astrolend_account.load_mut()?;
    let mut bank = bank_loader.load_mut()?;

    check!(
        !astrolend_account.get_flag(DISABLED_FLAG),
        AstrolendError::AccountDisabled
    );

    bank.accrue_interest(
        Clock::get()?.unix_timestamp,
        #[cfg(not(feature = "client"))]
        bank_loader.key(),
    )?;

    let mut bank_account = BankAccountWrapper::find(
        &bank_loader.key(),
        &mut bank,
        &mut astrolend_account.lending_account,
    )?;

    bank_account.close_balance()?;

    Ok(())
}

#[derive(Accounts)]
pub struct LendingAccountCloseBalance<'info> {
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
}
