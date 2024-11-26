pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod macros;
pub mod prelude;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;
use prelude::*;
use state::astrolend_group::{BankConfigCompact, BankConfigOpt};

cfg_if::cfg_if! {
    if #[cfg(feature = "mainnet-beta")] {
        declare_id!("Astro1oWvtB7cBTwi3efLMFB47WXx7DJDQeoxi235kA");
    } else if #[cfg(feature = "devnet")] {
        declare_id!("neetcne3Ctrrud7vLdt2ypMm21gZHGN2mCmqWaMVcBQ");
    } else if #[cfg(feature = "staging")] {
        declare_id!("stag8sTKds2h4KzjUw3zKTsxbqvT4XKHdaR9X9E6Rct");
    } else {
        declare_id!("2jGhuVUuy3umdzByFx8sNWUAaf5vaeuDm78RDPEnhrMr");
    }
}

#[program]
pub mod astrolend {
    use super::*;

    pub fn astrolend_group_initialize(ctx: Context<AstrolendGroupInitialize>) -> AstrolendResult {
        astrolend_group::initialize_group(ctx)
    }

    pub fn astrolend_group_configure(
        ctx: Context<AstrolendGroupConfigure>,
        config: GroupConfig,
    ) -> AstrolendResult {
        astrolend_group::configure(ctx, config)
    }

    pub fn lending_pool_add_bank(
        ctx: Context<LendingPoolAddBank>,
        bank_config: BankConfigCompact,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_add_bank(ctx, bank_config.into())
    }

    /// A copy of lending_pool_add_bank with an additional bank seed.
    /// This seed is used to create a PDA for the bank's signature.
    /// lending_pool_add_bank is preserved for backwards compatibility.
    pub fn lending_pool_add_bank_with_seed(
        ctx: Context<LendingPoolAddBankWithSeed>,
        bank_config: BankConfigCompact,
        bank_seed: u64,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_add_bank_with_seed(ctx, bank_config.into(), bank_seed)
    }

    pub fn lending_pool_configure_bank(
        ctx: Context<LendingPoolConfigureBank>,
        bank_config_opt: BankConfigOpt,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_configure_bank(ctx, bank_config_opt)
    }

    pub fn lending_pool_setup_emissions(
        ctx: Context<LendingPoolSetupEmissions>,
        flags: u64,
        rate: u64,
        total_emissions: u64,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_setup_emissions(ctx, flags, rate, total_emissions)
    }

    pub fn lending_pool_update_emissions_parameters(
        ctx: Context<LendingPoolUpdateEmissionsParameters>,
        emissions_flags: Option<u64>,
        emissions_rate: Option<u64>,
        additional_emissions: Option<u64>,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_update_emissions_parameters(
            ctx,
            emissions_flags,
            emissions_rate,
            additional_emissions,
        )
    }

    /// Handle bad debt of a bankrupt astrolend account for a given bank.
    pub fn lending_pool_handle_bankruptcy<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolHandleBankruptcy<'info>>,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_handle_bankruptcy(ctx)
    }

    // User instructions

    /// Initialize a astrolend account for a given group
    pub fn astrolend_account_initialize(
        ctx: Context<AstrolendAccountInitialize>,
    ) -> AstrolendResult {
        astrolend_account::initialize_account(ctx)
    }

    pub fn lending_account_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountDeposit<'info>>,
        amount: u64,
    ) -> AstrolendResult {
        astrolend_account::lending_account_deposit(ctx, amount)
    }

    pub fn lending_account_repay<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountRepay<'info>>,
        amount: u64,
        repay_all: Option<bool>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_repay(ctx, amount, repay_all)
    }

    pub fn lending_account_withdraw<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountWithdraw<'info>>,
        amount: u64,
        withdraw_all: Option<bool>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_withdraw(ctx, amount, withdraw_all)
    }

    pub fn lending_account_borrow<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountBorrow<'info>>,
        amount: u64,
    ) -> AstrolendResult {
        astrolend_account::lending_account_borrow(ctx, amount)
    }

    pub fn lending_account_close_balance(
        ctx: Context<LendingAccountCloseBalance>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_close_balance(ctx)
    }

    pub fn lending_account_withdraw_emissions<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountWithdrawEmissions<'info>>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_withdraw_emissions(ctx)
    }

    pub fn lending_account_settle_emissions(
        ctx: Context<LendingAccountSettleEmissions>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_settle_emissions(ctx)
    }

    /// Liquidate a lending account balance of an unhealthy astrolend account
    pub fn lending_account_liquidate<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountLiquidate<'info>>,
        asset_amount: u64,
    ) -> AstrolendResult {
        astrolend_account::lending_account_liquidate(ctx, asset_amount)
    }

    pub fn lending_account_start_flashloan(
        ctx: Context<LendingAccountStartFlashloan>,
        end_index: u64,
    ) -> AstrolendResult {
        astrolend_account::lending_account_start_flashloan(ctx, end_index)
    }

    pub fn lending_account_end_flashloan<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingAccountEndFlashloan<'info>>,
    ) -> AstrolendResult {
        astrolend_account::lending_account_end_flashloan(ctx)
    }

    // Operational instructions
    pub fn lending_pool_accrue_bank_interest(
        ctx: Context<LendingPoolAccrueBankInterest>,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_accrue_bank_interest(ctx)
    }

    pub fn lending_pool_collect_bank_fees<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolCollectBankFees<'info>>,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_collect_bank_fees(ctx)
    }

    pub fn lending_pool_withdraw_fees<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolWithdrawFees<'info>>,
        amount: u64,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_withdraw_fees(ctx, amount)
    }

    pub fn lending_pool_withdraw_insurance<'info>(
        ctx: Context<'_, '_, 'info, 'info, LendingPoolWithdrawInsurance<'info>>,
        amount: u64,
    ) -> AstrolendResult {
        astrolend_group::lending_pool_withdraw_insurance(ctx, amount)
    }

    pub fn set_account_flag(ctx: Context<SetAccountFlag>, flag: u64) -> AstrolendResult {
        astrolend_group::set_account_flag(ctx, flag)
    }

    pub fn unset_account_flag(ctx: Context<UnsetAccountFlag>, flag: u64) -> AstrolendResult {
        astrolend_group::unset_account_flag(ctx, flag)
    }

    pub fn set_new_account_authority(
        ctx: Context<AstrolendAccountSetAccountAuthority>,
    ) -> AstrolendResult {
        astrolend_account::set_account_transfer_authority(ctx)
    }

    pub fn astrolend_account_close(ctx: Context<AstrolendAccountClose>) -> AstrolendResult {
        astrolend_account::close_account(ctx)
    }
}
