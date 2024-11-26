use anchor_lang::prelude::*;

use crate::{check, state::astrolend_account::AstrolendAccount, AstrolendError, AstrolendResult};

pub fn close_account(ctx: Context<AstrolendAccountClose>) -> AstrolendResult {
    let astrolend_account = &ctx.accounts.astrolend_account.load()?;

    check!(
        astrolend_account.can_be_closed(),
        AstrolendError::IllegalAction,
        "Account cannot be closed"
    );

    Ok(())
}

#[derive(Accounts)]
pub struct AstrolendAccountClose<'info> {
    #[account(mut, close = fee_payer)]
    pub astrolend_account: AccountLoader<'info, AstrolendAccount>,
    #[account(address = astrolend_account.load()?.authority)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,
}
