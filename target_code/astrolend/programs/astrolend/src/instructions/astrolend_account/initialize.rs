use crate::{
    events::{AccountEventHeader, AstrolendAccountCreateEvent},
    prelude::*,
    state::astrolend_account::AstrolendAccount,
};
use anchor_lang::prelude::*;
use solana_program::sysvar::Sysvar;

pub fn initialize_account(ctx: Context<AstrolendAccountInitialize>) -> AstrolendResult {
    let AstrolendAccountInitialize {
        authority,
        astrolend_group,
        astrolend_account: astrolend_account_loader,
        ..
    } = ctx.accounts;

    let mut astrolend_account = astrolend_account_loader.load_init()?;

    astrolend_account.initialize(astrolend_group.key(), authority.key());

    emit!(AstrolendAccountCreateEvent {
        header: AccountEventHeader {
            signer: Some(authority.key()),
            astrolend_account: astrolend_account_loader.key(),
            astrolend_account_authority: astrolend_account.authority,
            astrolend_group: astrolend_account.group,
        }
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AstrolendAccountInitialize<'info> {
    pub astrolend_group: AccountLoader<'info, AstrolendGroup>,

    #[account(
        init,
        payer = fee_payer,
        space = 8 + std::mem::size_of::<AstrolendAccount>()
    )]
    pub astrolend_account: AccountLoader<'info, AstrolendAccount>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub fee_payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
