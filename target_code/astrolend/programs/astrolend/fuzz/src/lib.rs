use std::{
    collections::HashMap,
    mem::size_of,
    ops::AddAssign,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use account_state::{AccountInfoCache, AccountsState};
use anchor_lang::{
    accounts::{interface::Interface, interface_account::InterfaceAccount},
    prelude::{AccountInfo, AccountLoader, Context, Program, Pubkey, Rent, Signer, Sysvar},
    Discriminator, Key,
};
use anchor_spl::token_2022::spl_token_2022;
use arbitrary_helpers::{
    AccountIdx, AssetAmount, BankAndOracleConfig, BankIdx, PriceChange, TokenType,
};
use bank_accounts::{get_bank_map, BankAccounts};
use fixed_macro::types::I80F48;
use astrolend::{
    errors::AstrolendError,
    instructions::LendingPoolAddBankBumps,
    prelude::AstrolendGroup,
    state::{
        astrolend_account::AstrolendAccount,
        astrolend_group::{Bank, BankConfig, BankVaultType, InterestRateConfig},
    },
};
use metrics::{MetricAction, Metrics};
use solana_program::system_program;
use stubs::test_syscall_stubs;
use user_accounts::UserAccount;
use utils::{
    account_info_lifetime_shortener as ails, account_info_ref_lifetime_shortener as airls,
    account_info_slice_lifetime_shortener as aisls,
};

pub mod account_state;
pub mod arbitrary_helpers;
pub mod bank_accounts;
pub mod metrics;
pub mod stubs;
pub mod user_accounts;
pub mod utils;

pub struct AstrolendFuzzContext<'info> {
    pub astrolend_group: AccountInfo<'info>,
    pub banks: Vec<BankAccounts<'info>>,
    pub astrolend_accounts: Vec<UserAccount<'info>>,
    pub owner: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_sysvar: AccountInfo<'info>,
    pub last_sysvar_current_timestamp: RwLock<u64>,
    pub metrics: Arc<RwLock<Metrics>>,
    pub state: &'info AccountsState,
}

impl<'state> AstrolendFuzzContext<'state> {
    pub fn setup(
        state: &'state AccountsState,
        bank_configs: &[BankAndOracleConfig],
        n_users: u8,
    ) -> Self {
        let system_program = state.new_program(system_program::id());
        let admin = state.new_sol_account(1_000_000);
        let rent_sysvar = state.new_rent_sysvar_account(Rent::free());
        let astrolend_group =
            initialize_astrolend_group(state, admin.clone(), system_program.clone());

        let mut astrolend_state = AstrolendFuzzContext {
            astrolend_group,
            banks: vec![],
            owner: admin,
            system_program,
            rent_sysvar,
            astrolend_accounts: vec![],
            last_sysvar_current_timestamp: RwLock::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            metrics: Arc::new(RwLock::new(Metrics::default())),
            state,
        };
        astrolend_state.advance_time(0);

        bank_configs
            .iter()
            .for_each(|config| astrolend_state.setup_bank(state, Rent::free(), config));

        let token_vec = astrolend_state
            .banks
            .iter()
            .map(|b| b.mint.clone())
            .collect();

        (0..n_users).into_iter().for_each(|_| {
            astrolend_state
                .create_astrolend_account(state, Rent::free(), &token_vec)
                .unwrap()
        });

        // Create an extra account for seeding the banks
        astrolend_state
            .create_astrolend_account(state, Rent::free(), &token_vec)
            .unwrap();

        // Seed the banks
        for bank_idx in 0..astrolend_state.banks.len() {
            astrolend_state
                .process_action_deposit(
                    &AccountIdx(astrolend_state.astrolend_accounts.len() as u8 - 1),
                    &BankIdx(bank_idx as u8),
                    &AssetAmount(
                        1_000
                            * 10_u64
                                .pow(astrolend_state.banks[bank_idx as usize].mint_decimals.into()),
                    ),
                )
                .unwrap();
        }

        astrolend_state
    }

    fn get_bank_map<'a>(&'a self) -> HashMap<Pubkey, &'a BankAccounts<'state>> {
        get_bank_map(&self.banks)
    }

    fn refresh_oracle_accounts(&self) {
        self.banks.iter().for_each(|bank| {
            bank.refresh_oracle(
                self.last_sysvar_current_timestamp
                    .read()
                    .unwrap()
                    .to_owned() as i64,
            )
            .unwrap()
        });
    }

    pub fn advance_time(&self, time: u64) {
        self.last_sysvar_current_timestamp
            .write()
            .unwrap()
            .add_assign(time);

        test_syscall_stubs(Some(
            *self.last_sysvar_current_timestamp.read().unwrap() as i64
        ));
    }

    pub fn setup_bank<'a>(
        &'a mut self,
        state: &'state AccountsState,
        rent: Rent,
        initial_bank_config: &BankAndOracleConfig,
    ) {
        log!("Setting up bank with config {:#?}", initial_bank_config);
        let bank = state.new_owned_account(size_of::<Bank>(), astrolend::id(), rent.clone());

        let mint = state.new_token_mint(
            rent.clone(),
            initial_bank_config.mint_decimals,
            initial_bank_config.token_type,
        );
        let (liquidity_vault_authority, liquidity_vault_authority_bump) =
            state.new_vault_authority(BankVaultType::Liquidity, bank.key);
        let (liquidity_vault, liquidity_vault_bump) = state.new_vault_account(
            BankVaultType::Liquidity,
            mint.clone(),
            liquidity_vault_authority.key,
            bank.key,
        );

        let (insurance_vault_authority, insurance_vault_authority_bump) =
            state.new_vault_authority(BankVaultType::Insurance, bank.key);
        let (insurance_vault, insurance_vault_bump) = state.new_vault_account(
            BankVaultType::Insurance,
            mint.clone(),
            insurance_vault_authority.key,
            bank.key,
        );

        let (fee_vault_authority, fee_vault_authority_bump) =
            state.new_vault_authority(BankVaultType::Fee, bank.key);
        let (fee_vault, fee_vault_bump) = state.new_vault_account(
            BankVaultType::Fee,
            mint.clone(),
            fee_vault_authority.key,
            bank.key,
        );

        let oracle = state.new_oracle_account(
            rent.clone(),
            initial_bank_config.oracle_native_price as i64,
            *mint.key,
            initial_bank_config.mint_decimals as i32,
        );

        let add_bank_bumps = LendingPoolAddBankBumps {
            liquidity_vault_authority: liquidity_vault_authority_bump,
            liquidity_vault: liquidity_vault_bump,
            insurance_vault_authority: insurance_vault_authority_bump,
            insurance_vault: insurance_vault_bump,
            fee_vault_authority: fee_vault_authority_bump,
            fee_vault: fee_vault_bump,
        };

        let token_program = match initial_bank_config.token_type {
            TokenType::Tokenkeg => state.new_program(spl_token::id()),
            TokenType::Token22 | TokenType::Token22WithFee { .. } => {
                state.new_program(spl_token_2022::id())
            }
        };

        {
            astrolend::instructions::astrolend_group::lending_pool_add_bank(
                Context::new(
                    &astrolend::ID,
                    &mut astrolend::instructions::LendingPoolAddBank {
                        astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))
                            .unwrap(),
                        admin: Signer::try_from(airls(&self.owner)).unwrap(),
                        fee_payer: Signer::try_from(airls(&self.owner)).unwrap(),
                        bank_mint: Box::new(InterfaceAccount::try_from(airls(&mint)).unwrap()),
                        bank: AccountLoader::try_from_unchecked(&astrolend::ID, airls(&bank))
                            .unwrap(),
                        liquidity_vault_authority: ails(liquidity_vault_authority.clone()),
                        liquidity_vault: Box::new(
                            InterfaceAccount::try_from(airls(&liquidity_vault)).unwrap(),
                        ),
                        insurance_vault_authority: ails(insurance_vault_authority.clone()),
                        insurance_vault: Box::new(
                            InterfaceAccount::try_from(airls(&insurance_vault)).unwrap(),
                        ),
                        fee_vault_authority: ails(fee_vault_authority.clone()),
                        fee_vault: Box::new(InterfaceAccount::try_from(airls(&fee_vault)).unwrap()),
                        rent: Sysvar::from_account_info(airls(&self.rent_sysvar)).unwrap(),
                        token_program: Interface::try_from(airls(&token_program)).unwrap(),
                        system_program: Program::try_from(airls(&self.system_program)).unwrap(),
                    },
                    &[ails(oracle.clone())],
                    add_bank_bumps,
                ),
                BankConfig {
                    asset_weight_init: initial_bank_config.asset_weight_init,
                    asset_weight_maint: initial_bank_config.asset_weight_maint,
                    liability_weight_init: initial_bank_config.liability_weight_init,
                    liability_weight_maint: initial_bank_config.liability_weight_maint,
                    deposit_limit: initial_bank_config.deposit_limit,
                    borrow_limit: initial_bank_config.borrow_limit,
                    interest_rate_config: InterestRateConfig {
                        optimal_utilization_rate: I80F48!(0.5).into(),
                        plateau_interest_rate: I80F48!(0.5).into(),
                        max_interest_rate: I80F48!(4).into(),
                        insurance_fee_fixed_apr: I80F48!(0.01).into(),
                        insurance_ir_fee: I80F48!(0.05).into(),
                        protocol_fixed_fee_apr: I80F48!(0.01).into(),
                        protocol_ir_fee: I80F48!(0.1).into(),
                        ..Default::default()
                    },
                    oracle_setup: astrolend::state::price::OracleSetup::PythLegacy,
                    oracle_keys: [
                        oracle.key(),
                        Pubkey::default(),
                        Pubkey::default(),
                        Pubkey::default(),
                        Pubkey::default(),
                    ],
                    operational_state:
                        astrolend::state::astrolend_group::BankOperationalState::Operational,
                    risk_tier: if !initial_bank_config.risk_tier_isolated {
                        astrolend::state::astrolend_group::RiskTier::Collateral
                    } else {
                        astrolend::state::astrolend_group::RiskTier::Isolated
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        }

        set_discriminator::<Bank>(bank.clone());

        self.banks.push(BankAccounts {
            bank,
            oracle,
            liquidity_vault,
            insurance_vault,
            fee_vault,
            mint,
            liquidity_vault_authority,
            insurance_vault_authority,
            fee_vault_authority,
            mint_decimals: initial_bank_config.mint_decimals,
            token_program,
        });
    }

    fn create_astrolend_account<'a>(
        &'a mut self,
        state: &'state AccountsState,
        rent: Rent,
        token_mints: &Vec<AccountInfo<'state>>,
    ) -> anyhow::Result<()> {
        let astrolend_account =
            state.new_owned_account(size_of::<AstrolendAccount>(), astrolend::id(), rent.clone());

        astrolend::instructions::astrolend_account::initialize_account(Context::new(
            &astrolend::id(),
            &mut astrolend::instructions::astrolend_account::AstrolendAccountInitialize {
                astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                astrolend_account: AccountLoader::try_from_unchecked(
                    &astrolend::ID,
                    airls(&astrolend_account),
                )?,
                authority: Signer::try_from(airls(&self.owner))?,
                fee_payer: Signer::try_from(airls(&self.owner))?,
                system_program: Program::try_from(airls(&self.system_program))?,
            },
            &[],
            Default::default(),
        ))?;

        let token_accounts = token_mints
            .iter()
            .map(|token| {
                state.new_token_account(
                    token.clone(),
                    self.owner.key,
                    100_000_000_000_000_000,
                    rent.clone(),
                )
            })
            .collect();

        set_discriminator::<AstrolendAccount>(astrolend_account.clone());

        self.astrolend_accounts
            .push(UserAccount::new(astrolend_account, token_accounts));

        Ok(())
    }

    pub fn process_action_deposit(
        &self,
        account_idx: &AccountIdx,
        bank_idx: &BankIdx,
        asset_amount: &AssetAmount,
    ) -> anyhow::Result<()> {
        let astrolend_account = &self.astrolend_accounts[account_idx.0 as usize];
        sort_balances(airls(&astrolend_account.astrol_account));

        let bank = &self.banks[bank_idx.0 as usize];

        let cache = AccountInfoCache::new(&[
            astrolend_account.astrol_account.clone(),
            bank.bank.clone(),
            astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
            bank.liquidity_vault.clone(),
        ]);

        let mut remaining_accounts: Vec<AccountInfo> = vec![];
        if bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(bank.mint.clone()));
        }

        let res = astrolend::instructions::astrolend_account::lending_account_deposit(
            Context::new(
                &astrolend::ID,
                &mut astrolend::instructions::LendingAccountDeposit {
                    astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                    astrolend_account: AccountLoader::try_from(airls(
                        &astrolend_account.astrol_account,
                    ))?,
                    signer: Signer::try_from(airls(&self.owner))?,
                    bank: AccountLoader::try_from(airls(&bank.bank))?,
                    signer_token_account: ails(
                        astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
                    ),
                    bank_liquidity_vault: ails(bank.liquidity_vault.clone()),
                    token_program: Interface::try_from(airls(&bank.token_program))?,
                },
                &remaining_accounts,
                Default::default(),
            ),
            asset_amount.0,
        );

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                [AstrolendError::AccountDisabled.into(),].contains(&error),
                "Unexpected deposit error: {:?}",
                error
            );

            cache.revert();

            false
        } else {
            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Deposit, success);

        Ok(())
    }

    pub fn process_action_repay(
        &self,
        account_idx: &AccountIdx,
        bank_idx: &BankIdx,
        asset_amount: &AssetAmount,
        repay_all: bool,
    ) -> anyhow::Result<()> {
        let astrolend_account = &self.astrolend_accounts[account_idx.0 as usize];
        let bank = &self.banks[bank_idx.0 as usize];
        sort_balances(airls(&astrolend_account.astrol_account));

        let cache = AccountInfoCache::new(&[
            astrolend_account.astrol_account.clone(),
            bank.bank.clone(),
            astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
            bank.liquidity_vault.clone(),
        ]);

        let mut remaining_accounts = vec![];
        if bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(bank.mint.clone()));
        }

        let res = astrolend::instructions::astrolend_account::lending_account_repay(
            Context::new(
                &astrolend::ID,
                &mut astrolend::instructions::LendingAccountRepay {
                    astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                    astrolend_account: AccountLoader::try_from(airls(
                        &astrolend_account.astrol_account,
                    ))?,
                    signer: Signer::try_from(airls(&self.owner))?,
                    bank: AccountLoader::try_from(airls(&bank.bank))?,
                    signer_token_account: ails(
                        astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
                    ),
                    bank_liquidity_vault: ails(bank.liquidity_vault.clone()),
                    token_program: Interface::try_from(airls(&bank.token_program))?,
                },
                &remaining_accounts,
                Default::default(),
            ),
            asset_amount.0,
            Some(repay_all),
        );

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                vec![
                    AstrolendError::NoLiabilityFound.into(),
                    AstrolendError::OperationRepayOnly.into(),
                    // TODO: maybe change
                    AstrolendError::BankAccountNotFound.into(),
                    AstrolendError::AccountDisabled.into(),
                ]
                .contains(&error),
                "Unexpected repay error: {:?}",
                error
            );

            cache.revert();

            false
        } else {
            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Repay, success);

        Ok(())
    }

    pub fn process_action_withdraw(
        &'state self,
        account_idx: &AccountIdx,
        bank_idx: &BankIdx,
        asset_amount: &AssetAmount,
        withdraw_all: Option<bool>,
    ) -> anyhow::Result<()> {
        self.refresh_oracle_accounts();
        let astrolend_account = &self.astrolend_accounts[account_idx.0 as usize];
        sort_balances(airls(&astrolend_account.astrol_account));

        let bank = &self.banks[bank_idx.0 as usize];

        let cache = AccountInfoCache::new(&[
            astrolend_account.astrol_account.clone(),
            bank.bank.clone(),
            astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
            bank.liquidity_vault.clone(),
        ]);

        let remove_all_bank = if let Some(withdraw_all) = withdraw_all {
            if withdraw_all {
                vec![bank.bank.key()]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let mut remaining_accounts = vec![];
        if bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(bank.mint.clone()));
        }
        remaining_accounts.extend(astrolend_account.get_remaining_accounts(
            &self.get_bank_map(),
            vec![],
            remove_all_bank,
        ));
        let res = astrolend::instructions::astrolend_account::lending_account_withdraw(
            Context::new(
                &astrolend::ID,
                &mut astrolend::instructions::LendingAccountWithdraw {
                    astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                    astrolend_account: AccountLoader::try_from(airls(
                        &astrolend_account.astrol_account,
                    ))?,
                    signer: Signer::try_from(airls(&self.owner))?,
                    bank: AccountLoader::try_from(airls(&bank.bank))?,
                    token_program: Interface::try_from(airls(&bank.token_program))?,
                    destination_token_account: InterfaceAccount::try_from(airls(
                        &astrolend_account.token_accounts[bank_idx.0 as usize],
                    ))?,
                    bank_liquidity_vault_authority: ails(bank.liquidity_vault_authority.clone()),
                    bank_liquidity_vault: InterfaceAccount::try_from(airls(&bank.liquidity_vault))?,
                },
                aisls(&remaining_accounts),
                Default::default(),
            ),
            asset_amount.0,
            withdraw_all,
        );

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                [
                    AstrolendError::OperationWithdrawOnly.into(),
                    AstrolendError::IllegalUtilizationRatio.into(),
                    AstrolendError::RiskEngineInitRejected.into(),
                    AstrolendError::NoAssetFound.into(),
                    AstrolendError::BankAccountNotFound.into(),
                    AstrolendError::AccountDisabled.into(),
                ]
                .contains(&error),
                "Unexpected withdraw error: {:?}",
                error
            );

            cache.revert();

            false
        } else {
            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Withdraw, success);

        Ok(())
    }

    pub fn process_action_borrow(
        &'state self,
        account_idx: &AccountIdx,
        bank_idx: &BankIdx,
        asset_amount: &AssetAmount,
    ) -> anyhow::Result<()> {
        self.refresh_oracle_accounts();

        let astrolend_account = &self.astrolend_accounts[account_idx.0 as usize];
        let bank = &self.banks[bank_idx.0 as usize];
        let cache = AccountInfoCache::new(&[
            astrolend_account.astrol_account.clone(),
            bank.bank.clone(),
            astrolend_account.token_accounts[bank_idx.0 as usize].clone(),
            bank.liquidity_vault.clone(),
        ]);
        sort_balances(airls(&astrolend_account.astrol_account));

        let mut remaining_accounts = vec![];
        if bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(bank.mint.clone()));
        }
        remaining_accounts.extend(astrolend_account.get_remaining_accounts(
            &self.get_bank_map(),
            vec![bank.bank.key()],
            vec![],
        ));
        let res = astrolend::instructions::astrolend_account::lending_account_borrow(
            Context::new(
                &astrolend::ID,
                &mut astrolend::instructions::LendingAccountBorrow {
                    astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                    astrolend_account: AccountLoader::try_from(airls(
                        &astrolend_account.astrol_account,
                    ))?,
                    signer: Signer::try_from(airls(&self.owner))?,
                    bank: AccountLoader::try_from(airls(&bank.bank))?,
                    token_program: Interface::try_from(airls(&bank.token_program))?,
                    destination_token_account: InterfaceAccount::try_from(airls(
                        &astrolend_account.token_accounts[bank_idx.0 as usize],
                    ))?,
                    bank_liquidity_vault_authority: ails(bank.liquidity_vault_authority.clone()),
                    bank_liquidity_vault: InterfaceAccount::try_from(airls(&bank.liquidity_vault))?,
                },
                aisls(&remaining_accounts),
                Default::default(),
            ),
            asset_amount.0,
        );

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                vec![
                    AstrolendError::RiskEngineInitRejected.into(),
                    AstrolendError::IsolatedAccountIllegalState.into(),
                    AstrolendError::IllegalUtilizationRatio.into(),
                    AstrolendError::AccountDisabled.into(),
                ]
                .contains(&error),
                "Unexpected borrow error: {:?}",
                error
            );

            cache.revert();

            false
        } else {
            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Borrow, success);

        Ok(())
    }

    pub fn process_liquidate_account(
        &'state self,
        liquidator_idx: &AccountIdx,
        liquidatee_idx: &AccountIdx,
        asset_amount: &AssetAmount,
    ) -> anyhow::Result<()> {
        self.refresh_oracle_accounts();
        let liquidator_account = &self.astrolend_accounts[liquidator_idx.0 as usize];
        let liquidatee_account = &self.astrolend_accounts[liquidatee_idx.0 as usize];
        sort_balances(airls(&liquidator_account.astrol_account));
        sort_balances(airls(&liquidatee_account.astrol_account));

        if liquidator_account.astrol_account.key() == liquidatee_account.astrol_account.key() {
            self.metrics
                .write()
                .unwrap()
                .update_metric(MetricAction::Liquidate, false);

            return Ok(());
        }

        let (asset_bank_idx, liab_bank_idx) =
            if let Some(a) = liquidatee_account.get_liquidation_banks(&self.banks) {
                if a.0 == a.1 {
                    self.metrics
                        .write()
                        .unwrap()
                        .update_metric(MetricAction::Liquidate, false);

                    return Ok(());
                } else {
                    a
                }
            } else {
                self.metrics
                    .write()
                    .unwrap()
                    .update_metric(MetricAction::Liquidate, false);

                return Ok(());
            };

        let asset_bank = &self.banks[asset_bank_idx.0 as usize];
        let liab_bank = &self.banks[liab_bank_idx.0 as usize];

        let account_cache = AccountInfoCache::new(&[
            liquidator_account.astrol_account.clone(),
            liquidatee_account.astrol_account.clone(),
            asset_bank.bank.clone(),
            asset_bank.liquidity_vault.clone(),
            liab_bank.bank.clone(),
            liab_bank.liquidity_vault.clone(),
            liab_bank.insurance_vault.clone(),
        ]);

        let mut remaining_accounts = vec![];
        if liab_bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(liab_bank.mint.clone()));
        }
        remaining_accounts.extend(vec![asset_bank.oracle.clone(), liab_bank.oracle.clone()]);

        let mut liquidator_remaining_accounts = liquidator_account.get_remaining_accounts(
            &self.get_bank_map(),
            vec![liab_bank.bank.key(), asset_bank.bank.key()],
            vec![],
        );
        let mut liquidatee_remaining_accounts =
            liquidatee_account.get_remaining_accounts(&self.get_bank_map(), vec![], vec![]);

        remaining_accounts.append(&mut liquidator_remaining_accounts);
        remaining_accounts.append(&mut liquidatee_remaining_accounts);

        let res = astrolend::instructions::lending_account_liquidate(
            Context::new(
                &astrolend::id(),
                &mut astrolend::instructions::LendingAccountLiquidate {
                    astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                    asset_bank: AccountLoader::try_from(airls(&asset_bank.bank))?,
                    liab_bank: AccountLoader::try_from(airls(&liab_bank.bank))?,
                    liquidator_astrolend_account: AccountLoader::try_from(airls(
                        &liquidator_account.astrol_account,
                    ))?,
                    signer: Signer::try_from(airls(&self.owner))?,
                    liquidatee_astrolend_account: AccountLoader::try_from(airls(
                        &liquidatee_account.astrol_account,
                    ))?,
                    bank_liquidity_vault_authority: ails(
                        liab_bank.liquidity_vault_authority.clone(),
                    ),
                    bank_liquidity_vault: Box::new(InterfaceAccount::try_from(airls(
                        &liab_bank.liquidity_vault,
                    ))?),
                    bank_insurance_vault: ails(liab_bank.insurance_vault.clone()),
                    token_program: Interface::try_from(airls(&liab_bank.token_program))?,
                },
                aisls(&remaining_accounts),
                Default::default(),
            ),
            asset_amount.0,
        );

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                vec![
                    AstrolendError::RiskEngineInitRejected.into(),
                    AstrolendError::IsolatedAccountIllegalState.into(),
                    AstrolendError::IllegalUtilizationRatio.into(),
                    AstrolendError::IllegalLiquidation.into(),
                    AstrolendError::AccountDisabled.into(),
                    AstrolendError::MathError.into(), // TODO: would be best to avoid this one
                ]
                .contains(&error),
                "Unexpected liquidate error: {:?}",
                error
            );

            account_cache.revert();

            false
        } else {
            self.process_handle_bankruptcy(liquidatee_idx, &liab_bank_idx)?;

            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Liquidate, success);

        Ok(())
    }

    pub fn process_handle_bankruptcy(
        &'state self,
        account_idx: &AccountIdx,
        bank_idx: &BankIdx,
    ) -> anyhow::Result<()> {
        log!("Action: Handle Bankruptcy");

        let astrolend_account = &self.astrolend_accounts[account_idx.0 as usize];
        let bank = &self.banks[bank_idx.0 as usize];

        let cache = AccountInfoCache::new(&[
            bank.bank.clone(),
            astrolend_account.astrol_account.clone(),
            bank.liquidity_vault.clone(),
            bank.insurance_vault.clone(),
        ]);

        let mut remaining_accounts = vec![];
        if bank.token_program.key() == spl_token_2022::ID {
            remaining_accounts.push(ails(bank.mint.clone()));
        }
        remaining_accounts.extend(astrolend_account.get_remaining_accounts(
            &self.get_bank_map(),
            vec![],
            vec![],
        ));
        let res = astrolend::instructions::lending_pool_handle_bankruptcy(Context::new(
            &astrolend::ID,
            &mut astrolend::instructions::LendingPoolHandleBankruptcy {
                astrolend_group: AccountLoader::try_from(airls(&self.astrolend_group))?,
                signer: Signer::try_from(airls(&self.owner))?,
                bank: AccountLoader::try_from(airls(&bank.bank))?,
                astrolend_account: AccountLoader::try_from(airls(&astrolend_account.astrol_account))?,
                liquidity_vault: ails(bank.liquidity_vault.clone()),
                insurance_vault: Box::new(InterfaceAccount::try_from(airls(
                    &bank.insurance_vault,
                ))?),
                insurance_vault_authority: ails(bank.insurance_vault_authority.clone()),
                token_program: Interface::try_from(airls(&bank.token_program))?,
            },
            aisls(&remaining_accounts),
            Default::default(),
        ));

        let success = if res.is_err() {
            let error = res.unwrap_err();

            self.metrics.write().unwrap().update_error(&error);

            assert!(
                vec![
                    AstrolendError::AccountDisabled.into(),
                    AstrolendError::AccountNotBankrupt.into(),
                ]
                .contains(&error),
                "Unexpected handle bankruptcy error: {:?}",
                error
            );

            cache.revert();

            false
        } else {
            true
        };

        self.metrics
            .write()
            .unwrap()
            .update_metric(MetricAction::Bankruptcy, success);

        Ok(())
    }

    pub fn process_update_oracle(
        &self,
        bank_idx: &BankIdx,
        price_change: &PriceChange,
    ) -> anyhow::Result<()> {
        log!("Action: Update Oracle");
        let bank = &self.banks[bank_idx.0 as usize];

        bank.update_oracle(price_change.0)?;

        self.metrics.write().unwrap().price_update += 1;

        Ok(())
    }
}

fn sort_balances<'a>(astrolend_account_ai: &'a AccountInfo<'a>) {
    let astrolend_account_loader =
        AccountLoader::<AstrolendAccount>::try_from(astrolend_account_ai).unwrap();
    let mut astrolend_account = astrolend_account_loader.load_mut().unwrap();
    astrolend_account
        .lending_account
        .balances
        .sort_by_key(|a| !a.active);
}

pub fn set_discriminator<T: Discriminator>(ai: AccountInfo) {
    let mut data = ai.try_borrow_mut_data().unwrap();

    if data[..8].ne(&[0u8; 8]) {
        panic!("Account discriminator is already set");
    }

    data[..8].copy_from_slice(&T::DISCRIMINATOR);
}

fn initialize_astrolend_group<'a>(
    state: &'a AccountsState,
    admin: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
) -> AccountInfo<'a> {
    let program_id = astrolend::id();
    let astrolend_group =
        state.new_owned_account(size_of::<AstrolendGroup>(), program_id, Rent::free());

    astrolend::instructions::astrolend_group::initialize_group(Context::new(
        &astrolend::id(),
        &mut astrolend::instructions::AstrolendGroupInitialize {
            // Unchecked because we are initializing the account.
            astrolend_group: AccountLoader::try_from_unchecked(&program_id, airls(&astrolend_group))
                .unwrap(),
            admin: Signer::try_from(airls(&admin)).unwrap(),
            system_program: Program::try_from(airls(&system_program)).unwrap(),
        },
        &[],
        Default::default(),
    ))
    .unwrap();

    set_discriminator::<AstrolendGroup>(astrolend_group.clone());

    astrolend_group
}

#[cfg(test)]
mod tests {
    use fixed::types::I80F48;
    use astrolend::state::astrolend_account::RiskEngine;
    use pyth_sdk_solana::state::PriceAccount;

    use super::*;
    #[test]
    fn deposit_test() {
        let account_state = AccountsState::new();

        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 2);

        let al =
            AccountLoader::<AstrolendGroup>::try_from_unchecked(&astrolend::id(), &a.astrolend_group)
                .unwrap();

        assert_eq!(al.load().unwrap().admin, a.owner.key());

        a.process_action_deposit(&AccountIdx(0), &BankIdx(0), &AssetAmount(1000))
            .unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();
        let astrolend_account = astrolend_account_ai.load().unwrap();

        assert_eq!(
            I80F48::from(astrolend_account.lending_account.balances[0].asset_shares),
            I80F48!(1000)
        );
    }

    #[test]
    fn borrow_test() {
        let account_state = AccountsState::new();
        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 2);

        a.process_action_deposit(&AccountIdx(1), &BankIdx(1), &AssetAmount(1000))
            .unwrap();
        a.process_action_deposit(&AccountIdx(0), &BankIdx(0), &AssetAmount(1000))
            .unwrap();
        a.process_action_borrow(&AccountIdx(0), &BankIdx(1), &AssetAmount(100))
            .unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();

        {
            let astrolend_account = astrolend_account_ai.load().unwrap();

            assert_eq!(
                I80F48::from(astrolend_account.lending_account.balances[0].asset_shares),
                I80F48!(1000)
            );
            assert_eq!(
                I80F48::from(astrolend_account.lending_account.balances[1].liability_shares),
                I80F48!(100)
            );
        }

        a.process_action_repay(&AccountIdx(0), &BankIdx(1), &AssetAmount(100), false)
            .unwrap();

        let astrolend_account = astrolend_account_ai.load().unwrap();

        assert_eq!(
            I80F48::from(astrolend_account.lending_account.balances[1].liability_shares),
            I80F48!(0)
        );
    }

    #[test]
    fn liquidation_test() {
        let account_state = AccountsState::new();
        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 3);

        a.process_action_deposit(&AccountIdx(1), &BankIdx(1), &AssetAmount(1000))
            .unwrap();
        a.process_action_deposit(&AccountIdx(0), &BankIdx(0), &AssetAmount(1000))
            .unwrap();
        a.process_action_borrow(&AccountIdx(0), &BankIdx(1), &AssetAmount(500))
            .unwrap();

        a.banks[1].log_oracle_price().unwrap();

        a.process_update_oracle(&BankIdx(1), &PriceChange(10000000000000))
            .unwrap();

        a.banks[1].log_oracle_price().unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();

        {
            let astrolend_account = astrolend_account_ai.load().unwrap();
            let astrol_account = &a.astrolend_accounts[0];
            let bank_map = a.get_bank_map();
            let remaining_accounts =
                astrol_account.get_remaining_accounts(&bank_map, vec![], vec![]);

            let re = RiskEngine::new(&astrolend_account, aisls(&remaining_accounts)).unwrap();

            let health = re
                .get_account_health(
                    astrolend::state::astrolend_account::RiskRequirementType::Maintenance,
                )
                .unwrap();

            println!("Health {health}");
        }

        a.process_action_deposit(&AccountIdx(2), &BankIdx(1), &AssetAmount(1000))
            .unwrap();

        a.process_liquidate_account(&AccountIdx(2), &AccountIdx(0), &AssetAmount(50))
            .unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();

        let astrolend_account = astrolend_account_ai.load().unwrap();

        assert_eq!(
            I80F48::from(astrolend_account.lending_account.balances[0].asset_shares),
            I80F48!(950)
        );
    }

    #[test]
    fn liquidation_and_bankruptcy() {
        let account_state = AccountsState::new();

        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 3);

        a.process_action_deposit(&AccountIdx(1), &BankIdx(1), &AssetAmount(1000))
            .unwrap();
        a.process_action_deposit(&AccountIdx(0), &BankIdx(0), &AssetAmount(1000))
            .unwrap();
        a.process_action_borrow(&AccountIdx(0), &BankIdx(1), &AssetAmount(500))
            .unwrap();

        a.process_update_oracle(&BankIdx(1), &PriceChange(1000000000000))
            .unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();

        {
            let astrolend_account = astrolend_account_ai.load().unwrap();
            let astrol_account = &a.astrolend_accounts[0];
            let bank_map = a.get_bank_map();
            let remaining_accounts =
                astrol_account.get_remaining_accounts(&bank_map, vec![], vec![]);

            let re = RiskEngine::new(&astrolend_account, aisls(&remaining_accounts)).unwrap();

            let health = re
                .get_account_health(
                    astrolend::state::astrolend_account::RiskRequirementType::Maintenance,
                )
                .unwrap();

            println!("Health {health}");
        }

        a.process_action_deposit(&AccountIdx(2), &BankIdx(1), &AssetAmount(1000))
            .unwrap();

        a.process_liquidate_account(&AccountIdx(2), &AccountIdx(0), &AssetAmount(1000))
            .unwrap();

        let astrolend_account_ai = AccountLoader::<AstrolendAccount>::try_from_unchecked(
            &astrolend::id(),
            &a.astrolend_accounts[0].astrol_account,
        )
        .unwrap();

        let astrolend_account = astrolend_account_ai.load().unwrap();

        assert_eq!(
            I80F48::from(astrolend_account.lending_account.balances[0].asset_shares),
            I80F48!(0)
        );
        assert_eq!(
            I80F48::from(astrolend_account.lending_account.balances[0].liability_shares),
            I80F48!(0)
        );
    }

    #[test]
    fn price_update() {
        let account_state = AccountsState::new();

        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 3);

        a.process_update_oracle(&BankIdx(0), &PriceChange(1100))
            .unwrap();

        let new_price = {
            let data = a.banks[0].oracle.try_borrow_data().unwrap();
            let data = bytemuck::from_bytes::<PriceAccount>(&data);
            data.ema_price.val
        };

        assert_eq!(new_price, 1100);
    }

    #[test]
    fn pyth_timestamp_update() {
        let account_state = AccountsState::new();

        let a = AstrolendFuzzContext::setup(&account_state, &[BankAndOracleConfig::dummy(); 2], 3);

        let initial_timestamp = {
            let data = a.banks[0].oracle.try_borrow_data().unwrap();
            let data = bytemuck::from_bytes::<PriceAccount>(&data);
            data.timestamp
        };
        assert_eq!(initial_timestamp, 0);

        a.banks[0].refresh_oracle(123_456).unwrap();

        let updated_timestamp_via_0_10 = {
            let pf =
                pyth_sdk_solana::load_price_feed_from_account_info(&a.banks[0].oracle).unwrap();

            pf.get_ema_price_unchecked().publish_time
        };
        assert_eq!(updated_timestamp_via_0_10, 123_456);
    }
}
