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
use simple_role_macros::enforce_role_auth;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env};
use stellar_utils::storage::ttl::{
    get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use super::{
    emit_set_burn_limit_per_message, emit_swap_minter_config_removed, emit_swap_minter_config_set,
    emit_token_decimal_config_added, emit_token_pair_linked, emit_token_pair_unlinked,
    SwapMinterConfig, TokenControllerError, TokenDecimalConfig, TOKEN_CONTROLLER,
};

#[contracttype]
pub enum TokenControllerStorageKey {
    BurnLimit(Address),
    RemoteTokenToLocal((u32, BytesN<32>)),
    TokenDecimalConfig(Address),
    SwapMinterConfig(Address),
}

/// Links a local token to a remote domain token pair.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `remote_domain` - The identifier of the remote domain/chain.
/// * `remote_token` - The 32-byte token identifier on the remote domain.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
/// * [`TokenControllerError::TokenPairAlreadyLinked`] – If the remote token is already linked.
///
/// # Events
///
/// * topics - `["token_pair_linked"]`
/// * data - `[local_token: Address, remote_domain: u32, remote_token: BytesN<32>]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn link_token_pair(
    e: &Env,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    let key = TokenControllerStorageKey::RemoteTokenToLocal((remote_domain, remote_token.clone()));

    if e.storage().persistent().has(&key) {
        panic_with_error!(e, TokenControllerError::TokenPairAlreadyLinked);
    }

    e.storage().persistent().set(&key, local_token);

    emit_token_pair_linked(e, local_token, remote_domain, remote_token);
}

/// Unlinks a previously linked token pair.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `remote_domain` - The identifier of the remote domain/chain.
/// * `remote_token` - The 32-byte token identifier on the remote domain.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
/// * [`TokenControllerError::TokenPairNotLinked`] – If the token pair is not currently linked.
/// * [`TokenControllerError::InvalidLocalToken`] – If the provided local token does not match the stored local token.
///
/// # Events
///
/// * topics - `["token_pair_unlinked"]`
/// * data - `[local_token: Address, remote_domain: u32, remote_token: BytesN<32>]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn unlink_token_pair(
    e: &Env,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    let key = TokenControllerStorageKey::RemoteTokenToLocal((remote_domain, remote_token.clone()));

    let stored_local_token: Address = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, TokenControllerError::TokenPairNotLinked));

    if stored_local_token != *local_token {
        panic_with_error!(e, TokenControllerError::InvalidLocalToken);
    }

    e.storage().persistent().remove(&key);

    emit_token_pair_unlinked(e, local_token, remote_domain, remote_token);
}

/// Sets the maximum burn amount per message for a specific token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `burn_limit_per_message` - The maximum amount that can be burned per message.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
/// * [`TokenControllerError::InvalidBurnLimit`] – If the burn limit is negative.
///
/// # Events
///
/// * topics - `["set_burn_limit_per_message", local_token: Address]`
/// * data - `[burn_limit_per_message: i128]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn set_max_burn_amount_per_message(
    e: &Env,
    local_token: &Address,
    burn_limit_per_message: i128,
) {
    if burn_limit_per_message < 0 {
        panic_with_error!(e, TokenControllerError::InvalidBurnLimit);
    }

    let key = TokenControllerStorageKey::BurnLimit(local_token.clone());
    e.storage().persistent().set(&key, &burn_limit_per_message);

    emit_set_burn_limit_per_message(e, local_token, burn_limit_per_message);
}

/// Retrieves the local token address for a given remote domain and token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `remote_domain` - The identifier of the remote domain/chain.
/// * `remote_token` - The 32-byte token identifier on the remote domain.
///
/// # Returns
///
/// The local token address if a mapping exists, otherwise `None`.
pub fn get_local_token(e: &Env, remote_domain: u32, remote_token: &BytesN<32>) -> Option<Address> {
    let key = TokenControllerStorageKey::RemoteTokenToLocal((remote_domain, remote_token.clone()));
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
}

/// Retrieves the maximum burn amount per message for a specific token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
///
/// # Returns
///
/// The maximum burn amount per message if set, otherwise `None`.
pub fn get_max_burn_amount_per_message(e: &Env, local_token: &Address) -> Option<i128> {
    let key = TokenControllerStorageKey::BurnLimit(local_token.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
}

/// Enforces that the burn amount is within the configured limit for the token.
///
/// This function validates that:
/// 1. A burn limit is configured for the token
/// 2. The burn limit is greater than zero
/// 3. The amount does not exceed the burn limit
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - The address of the token being burned.
/// * `amount` - The amount to burn.
///
/// # Errors
///
/// * [`TokenControllerError::BurnTokenNotSupported`] – Burn limit not set or is zero.
/// * [`TokenControllerError::BurnAmountExceedsLimit`] – Amount exceeds the configured burn limit.
pub fn enforce_within_burn_limit(e: &Env, burn_token: &Address, amount: i128) {
    let burn_limit = get_max_burn_amount_per_message(e, burn_token)
        .unwrap_or_else(|| panic_with_error!(e, TokenControllerError::BurnTokenNotSupported));

    if burn_limit <= 0 {
        panic_with_error!(e, TokenControllerError::BurnTokenNotSupported);
    }

    if amount > burn_limit {
        panic_with_error!(e, TokenControllerError::BurnAmountExceedsLimit);
    }
}

/// Gets the decimal configuration for a local token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
///
/// # Returns
///
/// The token decimal configuration if set, otherwise `None`.
pub fn get_token_decimal_config(e: &Env, local_token: &Address) -> Option<TokenDecimalConfig> {
    let key = TokenControllerStorageKey::TokenDecimalConfig(local_token.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
}

/// Sets the decimal configuration for a token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `local_decimals` - The number of decimals for the local token.
/// * `canonical_decimals` - The number of decimals for the canonical token.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
/// * [`TokenControllerError::InvalidDecimalScale`] – If `local_decimals` is less than `canonical_decimals`.
/// * [`TokenControllerError::TokenDecimalConfigAlreadySet`] – If the token decimal config is already set.
///
/// # Events
///
/// * topics - `["token_decimal_config_added", local_token: Address]`
/// * data - `[token_decimal_config: TokenDecimalConfig]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn set_token_decimal_config(
    e: &Env,
    local_token: &Address,
    local_decimals: u32,
    canonical_decimals: u32,
) {
    if local_decimals < canonical_decimals {
        panic_with_error!(e, TokenControllerError::InvalidDecimalScale);
    }

    let key = TokenControllerStorageKey::TokenDecimalConfig(local_token.clone());

    if e.storage().persistent().has(&key) {
        panic_with_error!(e, TokenControllerError::TokenDecimalConfigAlreadySet);
    }

    let config = TokenDecimalConfig {
        local_decimals,
        canonical_decimals,
    };
    e.storage().persistent().set(&key, &config);

    emit_token_decimal_config_added(e, local_token, &config);
}

/// Retrieves the swap minter configuration for a token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
///
/// # Returns
///
/// The swap minter configuration if set, otherwise `None`.
pub fn get_swap_minter_config(e: &Env, local_token: &Address) -> Option<SwapMinterConfig> {
    let key = TokenControllerStorageKey::SwapMinterConfig(local_token.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
}

/// Sets the swap minter configuration for a token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `swap_minter` - The address of the swap minter contract.
/// * `allow_asset` - The address of the allowance asset to approve.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
///
/// # Events
///
/// * topics - `["swap_minter_config_set", local_token: Address]`
/// * data - `[swap_minter_config: SwapMinterConfig]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn set_swap_minter_config(
    e: &Env,
    local_token: &Address,
    swap_minter: &Address,
    allow_asset: &Address,
) {
    let key = TokenControllerStorageKey::SwapMinterConfig(local_token.clone());
    let config = SwapMinterConfig {
        swap_minter: swap_minter.clone(),
        allow_asset: allow_asset.clone(),
    };
    e.storage().persistent().set(&key, &config);

    emit_swap_minter_config_set(e, local_token, &config);
}

/// Removes the swap minter configuration for a token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
///
/// # Errors
///
/// * [`RoleError::RoleNotSet`] – If the token controller is not set.
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the token controller fails.
/// * [`TokenControllerError::SwapMinterConfigNotSet`] – If no configuration exists for the token.
///
/// # Events
///
/// * topics - `["swap_minter_config_removed", local_token: Address]`
/// * data - `[swap_minter_config: SwapMinterConfig]`
#[enforce_role_auth(TOKEN_CONTROLLER)]
pub fn remove_swap_minter_config(e: &Env, local_token: &Address) {
    let key = TokenControllerStorageKey::SwapMinterConfig(local_token.clone());

    // Get the existing config to emit in the event
    let config: SwapMinterConfig = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, TokenControllerError::SwapMinterConfigNotSet));

    e.storage().persistent().remove(&key);

    emit_swap_minter_config_removed(e, local_token, &config);
}
