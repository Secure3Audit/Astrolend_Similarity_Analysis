use crate::{prelude::*, state::astrolend_account::AstrolendAccount};
use anchor_lang::prelude::*;

pub fn set_account_transfer_authority(
    ctx: Context<AstrolendAccountSetAccountAuthority>,
) -> AstrolendResult {
    // Ensure astrolend_account is dropped out of scope to not exceed stack frame limits
    {
        let mut astrolend_account = ctx.accounts.astrolend_account.load_mut()?;
        let new_account_authority = ctx.accounts.new_authority.key();
        astrolend_account.set_new_account_authority_checked(new_account_authority)?;
    }

    // TODO: add back event (dropped for memory reasons)

    Ok(())
}

#[derive(Accounts)]
pub struct AstrolendAccountSetAccountAuthority<'info> {
    #[account(mut)]
    pub astrolend_account: AccountLoader<'info, AstrolendAccount>,

    /// CHECK: The group is confirmed by the address macro
    #[account(
        address = astrolend_account.load()?.group,
    )]
    pub astrolend_group: AccountInfo<'info>,

    #[account(
        address = astrolend_account.load()?.authority,
    )]
    pub signer: Signer<'info>,

    /// CHECK: The new account authority doesn't need explicit checks
    pub new_authority: AccountInfo<'info>,

    #[account(mut)]
    pub fee_payer: Signer<'info>,
}
