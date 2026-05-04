/*
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Test utilities for TokenMessengerMinter contract testing.
//!
//! This module provides:
//! - Test context structs and setup helpers
//! - Authorization mock helpers for contract calls

use crate::contract::{
    TokenMessengerMinterV2Contract, TokenMessengerMinterV2ContractClient,
    TokenMessengerMinterV2ContractInitParams,
};
use cctp_roles::test_utils::token_controller::{
    mock_link_token_pair_auth, mock_set_max_burn_amount_per_message_auth,
    mock_set_swap_minter_config_auth, mock_set_token_decimal_config_auth,
};
pub use cctp_utils::{BurnMessageV2, MessageV2};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    vec, Address, Bytes, BytesN, Env, IntoVal, U256,
};

// Re-export mock contracts from external packages
pub use failing_mock_message_transmitter::FailingMockMessageTransmitterContract as FailingMockMessageTransmitter;
pub use mock_message_transmitter::MockMessageTransmitterContract as MockMessageTransmitter;
pub use mock_swap_minter::MockSwapMinterContract;

// =============================================================================
// Storage Test Stub
// =============================================================================

/// Minimal stub contract for testing storage error conditions.
/// This contract creates instance storage but does not set the storage keys
/// that TokenMessengerMinter expects, allowing us to test "not set" errors.
mod storage_test_stub {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct StorageTestStub;

    #[contractimpl]
    impl StorageTestStub {
        pub fn __constructor(_env: Env) {
            // Empty constructor - creates instance storage but sets nothing
        }

        /// Calls get_local_message_transmitter which should panic if not set
        pub fn get_local_msg_transmitter(e: &Env) -> Address {
            crate::storage::get_local_message_transmitter(e)
        }

        /// Calls get_message_body_version which should panic if not set
        pub fn get_msg_body_version(e: &Env) -> u32 {
            crate::storage::get_message_body_version(e)
        }
    }
}

pub use storage_test_stub::{StorageTestStub, StorageTestStubClient};

// Re-export MessageSent event for test assertions
pub use mock_message_transmitter::MessageSent;

// =============================================================================
// Constants
// =============================================================================

/// Reuse existing test WASM from the roles package for upgrade tests
pub const UPGRADE_V2_WASM: &[u8] = include_bytes!("../../../testdata/upgrade_v2.wasm");

/// Default message body version for tests
pub const MESSAGE_BODY_VERSION: u32 = 1;

// =============================================================================
// Test Contexts
// =============================================================================

/// Basic test context for general contract testing.
pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub owner: Address,
    pub pauser: Address,
    pub rescuer: Address,
    pub token_controller: Address,
    pub admin: Address,
    pub fee_recipient: Address,
    pub min_fee_controller: Address,
    pub denylister: Address,
    pub message_transmitter: Address,
}

impl TestContext {
    pub fn client(&self) -> TokenMessengerMinterV2ContractClient<'_> {
        TokenMessengerMinterV2ContractClient::new(&self.env, &self.contract_id)
    }
}

/// Test context for deposit_for_burn tests.
pub struct DepositTestContext {
    pub env: Env,
    pub contract_id: Address,
    pub owner: Address,
    pub denylister: Address,
    pub token_controller: Address,
    pub burn_token: Address,
    pub caller: Address,
    pub destination_domain: u32,
    pub remote_token_messenger: BytesN<32>,
    pub message_transmitter: Address,
}

impl DepositTestContext {
    pub fn client(&self) -> TokenMessengerMinterV2ContractClient<'_> {
        TokenMessengerMinterV2ContractClient::new(&self.env, &self.contract_id)
    }
}

/// Test context for handle_receive_message tests.
pub struct ReceiveTestContext {
    pub env: Env,
    pub contract_id: Address,
    pub owner: Address,
    pub token_controller: Address,
    pub fee_recipient: Address,
    pub denylister: Address,
    pub message_transmitter: Address,
    pub remote_domain: u32,
    pub remote_token_messenger: BytesN<32>,
    pub local_token: Address,
    pub remote_token: BytesN<32>,
    pub swap_minter: Address,
    pub allow_asset: Address,
}

impl ReceiveTestContext {
    pub fn client(&self) -> TokenMessengerMinterV2ContractClient<'_> {
        TokenMessengerMinterV2ContractClient::new(&self.env, &self.contract_id)
    }
}

// =============================================================================
// Setup Helpers
// =============================================================================

/// Helper to create a Stellar Asset Contract (SAC) token for testing.
pub fn create_test_token(env: &Env, admin: &Address) -> Address {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    token_address.address()
}

/// Sets up a basic contract for general testing.
pub fn setup_contract() -> TestContext {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let message_transmitter = Address::generate(&env);

    let contract_id = env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer: rescuer.clone(),
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env],
            remote_token_messengers: vec![&env],
        },),
    );

    TestContext {
        env,
        contract_id,
        owner,
        pauser,
        rescuer,
        token_controller,
        admin,
        fee_recipient,
        min_fee_controller,
        denylister,
        message_transmitter,
    }
}

/// Sets up a contract with a registered remote token messenger and burn token configured.
/// Used for deposit_for_burn tests.
pub fn setup_deposit_test_with_decimals(
    local_decimals: u32,
    canonical_decimals: u32,
) -> DepositTestContext {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let caller = Address::generate(&env);

    // Register a mock message transmitter
    let message_transmitter = env.register(MockMessageTransmitter, ());

    // Setup remote token messenger for destination domain 1
    let destination_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);

    let contract_id = env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer: rescuer.clone(),
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, destination_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );

    // Create a test token (owner is the SAC admin)
    let burn_token = create_test_token(&env, &owner);

    // Configure burn limit for the token
    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &token_controller,
        &burn_token,
        1_000_000_i128,
    );
    client.set_max_burn_amount_per_message(&burn_token, &1_000_000_i128);

    // Configure decimal config (required for all deposit_for_burn operations)
    mock_set_token_decimal_config_auth(
        &env,
        &contract_id,
        &token_controller,
        &burn_token,
        local_decimals,
        canonical_decimals,
    );
    client.set_token_decimal_config(&burn_token, &local_decimals, &canonical_decimals);

    // Mint tokens to the caller
    let token_admin_client = StellarAssetClient::new(&env, &burn_token);
    mock_sac_mint_auth(&env, &burn_token, &owner, &caller, 10_000_000_i128);
    token_admin_client.mint(&caller, &10_000_000_i128);

    // Approve the contract to spend caller's tokens (required for transfer_from)
    // Use a large allowance and expiration relative to current ledger
    let expiration_ledger = env.ledger().sequence() + 1000;
    mock_approve_auth(
        &env,
        &burn_token,
        &caller,
        &contract_id,
        i128::MAX,
        expiration_ledger,
    );
    let token_client = soroban_sdk::token::TokenClient::new(&env, &burn_token);
    token_client.approve(&caller, &contract_id, &i128::MAX, &expiration_ledger);

    DepositTestContext {
        env,
        contract_id,
        owner,
        denylister,
        token_controller,
        burn_token,
        caller,
        destination_domain,
        remote_token_messenger,
        message_transmitter,
    }
}

/// Convenience wrapper that sets up a deposit test with equal decimals (6,6 = no conversion).
pub fn setup_deposit_test() -> DepositTestContext {
    setup_deposit_test_with_decimals(6, 6)
}

/// Sets up a contract configured for receiving cross-chain messages.
/// Used for handle_recv_finalized_message and handle_recv_unfinalized_message tests.
pub fn setup_receive_test_with_decimals(
    local_decimals: u32,
    canonical_decimals: u32,
) -> ReceiveTestContext {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let denylister = Address::generate(&env);
    let contract_id = Address::generate(&env);

    // Register a mock message transmitter
    let message_transmitter = env.register(MockMessageTransmitter, ());

    // Setup remote token messenger for remote domain 1
    let remote_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);
    let min_fee_controller = Address::generate(&env);

    env.register_at(
        &contract_id,
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer: rescuer.clone(),
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, remote_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );
    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);
    // Create tokens first with contract_id as temporary admin
    let local_token = create_test_token(&env, &contract_id);
    let allow_asset = create_test_token(&env, &contract_id);

    let allow_asset_client = StellarAssetClient::new(&env, &allow_asset);
    // Mint a large amount of allow_asset to the TokenMessengerMinter contract
    // This provides balance for the swap_minter to burn during swap_mint
    env.mock_all_auths();
    allow_asset_client.mint(&contract_id, &i128::MAX);
    env.mock_auths(&[]);

    // Register MockSwapMinterContract with the token addresses
    let swap_minter = env.register(
        MockSwapMinterContract,
        (local_token.clone(), allow_asset.clone()),
    );

    // Transfer local_token admin to swap_minter so it can mint
    env.mock_all_auths();
    StellarAssetClient::new(&env, &local_token).set_admin(&swap_minter);
    env.mock_auths(&[]);

    // Create a remote token identifier
    let remote_token = BytesN::<32>::random(&env);

    // Link the token pair

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &token_controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    // Configure decimal config (required for all receive/mint operations)
    mock_set_token_decimal_config_auth(
        &env,
        &contract_id,
        &token_controller,
        &local_token,
        local_decimals,
        canonical_decimals,
    );
    client.set_token_decimal_config(&local_token, &local_decimals, &canonical_decimals);

    // Configure swap minter config for the local token
    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &token_controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    ReceiveTestContext {
        env,
        contract_id,
        owner,
        token_controller,
        fee_recipient,
        denylister,
        message_transmitter,
        remote_domain,
        remote_token_messenger,
        local_token,
        remote_token,
        swap_minter,
        allow_asset,
    }
}

/// Convenience wrapper that sets up a receive test with equal decimals (6,6 = no conversion).
pub fn setup_receive_test() -> ReceiveTestContext {
    setup_receive_test_with_decimals(6, 6)
}

/// Helper to create a valid burn message for testing.
pub fn create_test_burn_message(
    env: &Env,
    burn_token: BytesN<32>,
    mint_recipient: BytesN<32>,
    amount: U256,
    max_fee: i128,
    fee_executed: i128,
) -> Bytes {
    create_test_burn_message_with_expiration(
        env,
        burn_token,
        mint_recipient,
        amount,
        max_fee,
        fee_executed,
        U256::from_u32(env, 0), // no expiration by default
    )
}

/// Helper to create a burn message with a specific expiration block.
#[allow(clippy::too_many_arguments)]
pub fn create_test_burn_message_with_expiration(
    env: &Env,
    burn_token: BytesN<32>,
    mint_recipient: BytesN<32>,
    amount: U256,
    max_fee: i128,
    fee_executed: i128,
    expiration_block: U256,
) -> Bytes {
    let message_sender = BytesN::<32>::random(env);
    let hook_data = Bytes::new(env);

    // Create a BurnMessage struct and serialize it
    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token,
        mint_recipient,
        amount,
        message_sender,
        max_fee: U256::from_u128(env, max_fee as u128),
        fee_executed: U256::from_u128(env, fee_executed as u128),
        expiration_block,
        hook_data,
    };

    burn_message.serialize(env)
}

// =============================================================================
// Authorization Mock Helpers
// =============================================================================

/// Mocks authorization for `deposit_for_burn` from the caller.
///
/// Note: The caller must have previously approved the contract to spend their tokens
/// via `token.approve()` before calling `deposit_for_burn`.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The TokenMessengerMinter contract address
/// * `caller` - The caller address that will be authorized
/// * `amount` - The amount of tokens to burn
/// * `destination_domain` - The destination domain identifier
/// * `mint_recipient` - The mint recipient on the destination domain
/// * `burn_token` - The token to burn
/// * `destination_caller` - The authorized caller on the destination domain
/// * `max_fee` - The maximum fee
/// * `min_finality_threshold` - The minimum finality threshold
#[allow(clippy::too_many_arguments)]
pub fn mock_deposit_for_burn_auth(
    env: &Env,
    contract_id: &Address,
    caller: &Address,
    amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    burn_token: &Address,
    destination_caller: &BytesN<32>,
    max_fee: i128,
    min_finality_threshold: u32,
) {
    env.mock_auths(&[MockAuth {
        address: caller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "deposit_for_burn",
            args: (
                caller.clone(),
                amount,
                destination_domain,
                mint_recipient.clone(),
                burn_token.clone(),
                destination_caller.clone(),
                max_fee,
                min_finality_threshold,
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for `deposit_for_burn_with_hook` from the caller.
///
/// Note: The caller must have previously approved the contract to spend their tokens
/// via `token.approve()` before calling `deposit_for_burn_with_hook`.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The TokenMessengerMinter contract address
/// * `caller` - The caller address that will be authorized
/// * `amount` - The amount of tokens to burn
/// * `destination_domain` - The destination domain identifier
/// * `mint_recipient` - The mint recipient on the destination domain
/// * `burn_token` - The token to burn
/// * `destination_caller` - The authorized caller on the destination domain
/// * `max_fee` - The maximum fee
/// * `min_finality_threshold` - The minimum finality threshold
/// * `hook_data` - The hook data for the destination domain
#[allow(clippy::too_many_arguments)]
pub fn mock_deposit_for_burn_with_hook_auth(
    env: &Env,
    contract_id: &Address,
    caller: &Address,
    amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    burn_token: &Address,
    destination_caller: &BytesN<32>,
    max_fee: i128,
    min_finality_threshold: u32,
    hook_data: &Bytes,
) {
    env.mock_auths(&[MockAuth {
        address: caller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "deposit_for_burn_with_hook",
            args: (
                caller.clone(),
                amount,
                destination_domain,
                mint_recipient.clone(),
                burn_token.clone(),
                destination_caller.clone(),
                max_fee,
                min_finality_threshold,
                hook_data.clone(),
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for `deposit_for_burn` with decimal conversion.
///
/// Note: The caller must have previously approved the contract to spend their tokens
/// via `token.approve()` before calling `deposit_for_burn`.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The TokenMessengerMinter contract address
/// * `caller` - The caller address that will be authorized
/// * `amount` - The amount passed to deposit_for_burn (user's original amount)
/// * `destination_domain` - The destination domain identifier
/// * `mint_recipient` - The mint recipient on the destination domain
/// * `burn_token` - The token to burn
/// * `destination_caller` - The authorized caller on the destination domain
/// * `max_fee` - The max_fee passed to deposit_for_burn (user's original max_fee)
/// * `min_finality_threshold` - The minimum finality threshold
#[allow(clippy::too_many_arguments)]
pub fn mock_deposit_for_burn_with_decimal_conversion_auth(
    env: &Env,
    contract_id: &Address,
    caller: &Address,
    amount: i128,
    destination_domain: u32,
    mint_recipient: &BytesN<32>,
    burn_token: &Address,
    destination_caller: &BytesN<32>,
    max_fee: i128,
    min_finality_threshold: u32,
) {
    env.mock_auths(&[MockAuth {
        address: caller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "deposit_for_burn",
            args: (
                caller.clone(),
                amount,
                destination_domain,
                mint_recipient.clone(),
                burn_token.clone(),
                destination_caller.clone(),
                max_fee,
                min_finality_threshold,
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for minting tokens from a Stellar Asset Contract.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `token` - The SAC token address
/// * `admin` - The SAC admin address that will be authorized
/// * `to` - The recipient address
/// * `amount` - The amount to mint
pub fn mock_sac_mint_auth(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
    env.mock_auths(&[MockAuth {
        address: admin,
        invoke: &MockAuthInvoke {
            contract: token,
            fn_name: "mint",
            args: (to.clone(), amount).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for approving a spender on a token.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `token` - The token address
/// * `from` - The token owner address that will be authorized
/// * `spender` - The spender address being approved
/// * `amount` - The amount to approve
/// * `expiration_ledger` - The ledger number when the approval expires
pub fn mock_approve_auth(
    env: &Env,
    token: &Address,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    env.mock_auths(&[MockAuth {
        address: from,
        invoke: &MockAuthInvoke {
            contract: token,
            fn_name: "approve",
            args: (from.clone(), spender.clone(), amount, expiration_ledger).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for `handle_recv_finalized_message` from the MessageTransmitter.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The TokenMessengerMinter contract address
/// * `message_transmitter` - The MessageTransmitter address that will be authorized
/// * `remote_domain` - The remote domain identifier
/// * `sender` - The sender address (remote token messenger) as bytes32
/// * `finality_threshold_executed` - The finality threshold at which the message was attested
/// * `message_body` - The burn message body bytes
#[allow(clippy::too_many_arguments)]
pub fn mock_handle_recv_finalized_message_auth(
    env: &Env,
    contract_id: &Address,
    message_transmitter: &Address,
    remote_domain: u32,
    sender: &BytesN<32>,
    finality_threshold_executed: u32,
    message_body: &Bytes,
) {
    env.mock_auths(&[MockAuth {
        address: message_transmitter,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "handle_recv_finalized_message",
            args: (
                remote_domain,
                sender.clone(),
                finality_threshold_executed,
                message_body.clone(),
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for `handle_recv_unfinalized_message` from the MessageTransmitter.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The TokenMessengerMinter contract address
/// * `message_transmitter` - The MessageTransmitter address that will be authorized
/// * `remote_domain` - The remote domain identifier
/// * `sender` - The sender address (remote token messenger) as bytes32
/// * `finality_threshold_executed` - The finality threshold at which the message was attested
/// * `message_body` - The burn message body bytes
#[allow(clippy::too_many_arguments)]
pub fn mock_handle_recv_unfinalized_message_auth(
    env: &Env,
    contract_id: &Address,
    message_transmitter: &Address,
    remote_domain: u32,
    sender: &BytesN<32>,
    finality_threshold_executed: u32,
    message_body: &Bytes,
) {
    env.mock_auths(&[MockAuth {
        address: message_transmitter,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "handle_recv_unfinalized_message",
            args: (
                remote_domain,
                sender.clone(),
                finality_threshold_executed,
                message_body.clone(),
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}
