use crate::events::{GroupEventHeader, AstrolendGroupCreateEvent};
use crate::{state::astrolend_group::AstrolendGroup, AstrolendResult};
use anchor_lang::prelude::*;

pub fn initialize_group(ctx: Context<AstrolendGroupInitialize>) -> AstrolendResult {
    let astrolend_group = &mut ctx.accounts.astrolend_group.load_init()?;

    astrolend_group.set_initial_configuration(ctx.accounts.admin.key());

    emit!(AstrolendGroupCreateEvent {
        header: GroupEventHeader {
            astrolend_group: ctx.accounts.astrolend_group.key(),
            signer: Some(*ctx.accounts.admin.key)
        },
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AstrolendGroupInitialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<AstrolendGroup>(),
    )]
    pub astrolend_group: AccountLoader<'info, AstrolendGroup>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
