use crate::{
    state::astrolend_group::{Bank, AstrolendGroup},
    AstrolendResult,
};
use anchor_lang::prelude::*;

pub fn lending_pool_accrue_bank_interest(
    ctx: Context<LendingPoolAccrueBankInterest>,
) -> AstrolendResult {
    let clock = Clock::get()?;
    let mut bank = ctx.accounts.bank.load_mut()?;

    bank.accrue_interest(
        clock.unix_timestamp,
        #[cfg(not(feature = "client"))]
        ctx.accounts.bank.key(),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct LendingPoolAccrueBankInterest<'info> {
    pub astrolend_group: AccountLoader<'info, AstrolendGroup>,

    #[account(
        mut,
        constraint = bank.load()?.group == astrolend_group.key(),
    )]
    pub bank: AccountLoader<'info, Bank>,
}
