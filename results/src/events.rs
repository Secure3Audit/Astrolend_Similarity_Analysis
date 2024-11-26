use crate::{prelude::*, state::astrolend_group::BankConfigOpt};
use anchor_lang::prelude::*;

// Event headers

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GroupEventHeader {
    pub signer: Option<Pubkey>,
    pub astrolend_group: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AccountEventHeader {
    pub signer: Option<Pubkey>,
    pub astrolend_account: Pubkey,
    pub astrolend_account_authority: Pubkey,
    pub astrolend_group: Pubkey,
}

// astrolend group events

#[event]
pub struct AstrolendGroupCreateEvent {
    pub header: GroupEventHeader,
}

#[event]
pub struct AstrolendGroupConfigureEvent {
    pub header: GroupEventHeader,
    pub config: GroupConfig,
}

#[event]
pub struct LendingPoolBankCreateEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
}

#[event]
pub struct LendingPoolBankConfigureEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub config: BankConfigOpt,
}

#[event]
pub struct LendingPoolBankAccrueInterestEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub delta: u64,
    pub fees_collected: f64,
    pub insurance_collected: f64,
}

#[event]
pub struct LendingPoolBankCollectFeesEvent {
    pub header: GroupEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub group_fees_collected: f64,
    pub group_fees_outstanding: f64,
    pub insurance_fees_collected: f64,
    pub insurance_fees_outstanding: f64,
}

#[event]
pub struct LendingPoolBankHandleBankruptcyEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub bad_debt: f64,
    pub covered_amount: f64,
    pub socialized_amount: f64,
}

// astrolend account events

#[event]
pub struct AstrolendAccountCreateEvent {
    pub header: AccountEventHeader,
}

#[event]
pub struct LendingAccountDepositEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LendingAccountRepayEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub close_balance: bool,
}

#[event]
pub struct LendingAccountBorrowEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LendingAccountWithdrawEvent {
    pub header: AccountEventHeader,
    pub bank: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub close_balance: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidationBalances {
    pub liquidatee_asset_balance: f64,
    pub liquidatee_liability_balance: f64,
    pub liquidator_asset_balance: f64,
    pub liquidator_liability_balance: f64,
}

#[event]
pub struct LendingAccountLiquidateEvent {
    pub header: AccountEventHeader,
    pub liquidatee_astrolend_account: Pubkey,
    pub liquidatee_astrolend_account_authority: Pubkey,
    pub asset_bank: Pubkey,
    pub asset_mint: Pubkey,
    pub liability_bank: Pubkey,
    pub liability_mint: Pubkey,
    pub liquidatee_pre_health: f64,
    pub liquidatee_post_health: f64,
    pub pre_balances: LiquidationBalances,
    pub post_balances: LiquidationBalances,
}

#[event]
pub struct AstrolendAccountTransferAccountAuthorityEvent {
    pub header: AccountEventHeader,
    pub old_account_authority: Pubkey,
    pub new_account_authority: Pubkey,
}
