use anchor_lang::prelude::*;

declare_id!("Astro1oWvtB7cBTwi3efLMFB47WXx7DJDQeoxi235kA");

#[program]
pub mod brick {
    use super::*;

    pub fn fallback(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> Result<()> {
        Err(ErrorCode::ProgramDisabled.into())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("This program is temporarily disabled.")]
    ProgramDisabled,
}
