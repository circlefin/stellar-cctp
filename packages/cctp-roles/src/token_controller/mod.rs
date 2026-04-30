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
//! # Token Controller Contract Module
//!
//! This module provides token control functionality for cross-chain token operations.
//! It allows managing token pairs between local and remote domains, setting burn limits
//! per message, and configuring decimal conversions for tokens.
//!
//! The `TokenController` trait exposes methods for:
//! - Getting and setting the token controller address
//! - Linking/unlinking token pairs across domains
//! - Setting burn limits per message for tokens
//! - Managing token decimal configurations
//! - Querying local token mappings
//!
//! Access control is enforced through the `token_controller` role, which must be set by
//! the contract owner. The `token_controller` has exclusive rights to manage token
//! configurations and mappings.

use soroban_sdk::{
    contractclient, contracterror, contractevent, contracttype, Address, BytesN, Env,
};

mod storage;
#[cfg(test)]
mod test;

pub use storage::{
    enforce_within_burn_limit, get_local_token, get_max_burn_amount_per_message,
    get_swap_minter_config, get_token_decimal_config, link_token_pair, remove_swap_minter_config,
    set_max_burn_amount_per_message, set_swap_minter_config, set_token_decimal_config,
    unlink_token_pair,
};

/// Role identifier for the token controller.
pub const TOKEN_CONTROLLER: &str = "token_controller";

/// Represents a pair of decimal configurations for local and canonical tokens.
///
/// This configuration is used to handle decimal precision differences between
/// tokens on different chains (e.g., Stellar USDC with 7 decimals vs CCTP with 6 decimals).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDecimalConfig {
    /// Number of decimals for the local token (e.g., 7 for Stellar USDC)
    pub local_decimals: u32,
    /// Number of decimals for the canonical token (e.g., 6 for standard CCTP)
    pub canonical_decimals: u32,
}

/// Represents a configuration for a local token needed to perform a swap mint with a SwapMinter.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapMinterConfig {
    pub swap_minter: Address,
    pub allow_asset: Address,
}

/// A trait for managing token controller functionality.
///
/// Provides functions to manage token pairs, burn limits, and decimal configurations
/// for cross-chain token operations.
#[contractclient(name = "TokenControllerClient")]
pub trait TokenController {
    /// Returns the current token controller address if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_token_controller(e: &Env) -> Option<Address>;

    /// Sets the token controller address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_token_controller` - The new token controller address.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["set_token_controller", token_controller: Address]`
    /// * data - `[]`
    fn set_token_controller(e: &Env, new_token_controller: Address);

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
    fn link_token_pair(e: &Env, local_token: Address, remote_domain: u32, remote_token: BytesN<32>);

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
    fn unlink_token_pair(
        e: &Env,
        local_token: Address,
        remote_domain: u32,
        remote_token: BytesN<32>,
    );

    /// Sets the maximum burn amount per message for a specific token.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `local_token` - The address of the local token.
    /// * `burn_limit_per_message` - The maximum amount that can be burned per message (must be non-negative).
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
    fn set_max_burn_amount_per_message(e: &Env, local_token: Address, burn_limit_per_message: i128);

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
    fn get_max_burn_amount_per_message(e: &Env, local_token: Address) -> Option<i128>;

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
    fn get_local_token(e: &Env, remote_domain: u32, remote_token: BytesN<32>) -> Option<Address>;

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
    fn get_token_decimal_config(e: &Env, local_token: Address) -> Option<TokenDecimalConfig>;

    /// Sets the decimal configuration for a local token.
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
    ///
    /// # Events
    ///
    /// * topics - `["token_decimal_config_added", local_token: Address]`
    /// * data - `[token_decimal_config: TokenDecimalConfig]`
    fn set_token_decimal_config(
        e: &Env,
        local_token: Address,
        local_decimals: u32,
        canonical_decimals: u32,
    );

    /// Gets the swap minter configuration for a token.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `local_token` - The address of the local token.
    ///
    /// # Returns
    ///
    /// The swap minter configuration if set, otherwise `None`.
    fn get_swap_minter_config(e: &Env, local_token: Address) -> Option<SwapMinterConfig>;

    /// Sets the swap minter configuration for a local token.
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
    fn set_swap_minter_config(
        e: &Env,
        local_token: Address,
        swap_minter: Address,
        allow_asset: Address,
    );

    /// Removes the swap minter configuration for a local token.
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
    fn remove_swap_minter_config(e: &Env, local_token: Address);
}

// ################## ERRORS ##################

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenControllerError {
    /// If a token pair is already linked.
    TokenPairAlreadyLinked = 6300,
    /// If a token pair is not linked.
    TokenPairNotLinked = 6301,
    /// If the token decimal config is not set.
    TokenDecimalConfigNotSet = 6302,
    /// If the burn token is not supported (no burn limit set or limit is zero).
    BurnTokenNotSupported = 6303,
    /// If the burn amount exceeds the configured limit per message.
    BurnAmountExceedsLimit = 6304,
    /// If the swap minter config is not set for the token.
    SwapMinterConfigNotSet = 6305,
    /// If the burn limit per message is invalid (negative).
    InvalidBurnLimit = 6306,
    /// If local_decimals is less than canonical_decimals.
    InvalidDecimalScale = 6307,
    /// If the token decimal config is already set.
    TokenDecimalConfigAlreadySet = 6308,
    /// If the provided local token does not match the stored local token.
    InvalidLocalToken = 6309,
}

// ################## EVENTS ##################

/// Emitted when a token pair is linked between domains.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenPairLinked {
    pub local_token: Address,
    pub remote_domain: u32,
    pub remote_token: BytesN<32>,
}

/// Emitted when a token pair is unlinked between domains.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenPairUnlinked {
    pub local_token: Address,
    pub remote_domain: u32,
    pub remote_token: BytesN<32>,
}

/// Emitted when a burn limit per message is set for a local token.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetBurnLimitPerMessage {
    #[topic]
    pub local_token: Address,
    pub burn_limit_per_message: i128,
}

/// Emitted when a new token controller is set.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetTokenController {
    pub token_controller: Address,
}

/// Emitted when a local token decimal config is added for a local token.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDecimalConfigAdded {
    #[topic]
    pub local_token: Address,
    pub token_decimal_config: TokenDecimalConfig,
}

/// Emitted when a swap minter config is set for a token.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapMinterConfigSet {
    #[topic]
    pub local_token: Address,
    pub swap_minter_config: SwapMinterConfig,
}

/// Emitted when a swap minter config is removed for a local token.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapMinterConfigRemoved {
    #[topic]
    pub local_token: Address,
    pub swap_minter_config: SwapMinterConfig,
}

/// Emits an event when the token controller is set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `token_controller` - The address of the new token controller.
pub fn emit_set_token_controller(e: &Env, token_controller: &Address) {
    SetTokenController {
        token_controller: token_controller.clone(),
    }
    .publish(e);
}

/// Emits an event when a token pair is linked.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `remote_domain` - The identifier of the remote domain.
/// * `remote_token` - The 32-byte token identifier on the remote domain.
pub fn emit_token_pair_linked(
    e: &Env,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    TokenPairLinked {
        local_token: local_token.clone(),
        remote_domain,
        remote_token: remote_token.clone(),
    }
    .publish(e);
}

/// Emits an event when a token pair is unlinked.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `remote_domain` - The identifier of the remote domain.
/// * `remote_token` - The 32-byte token identifier on the remote domain.
pub fn emit_token_pair_unlinked(
    e: &Env,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    TokenPairUnlinked {
        local_token: local_token.clone(),
        remote_domain,
        remote_token: remote_token.clone(),
    }
    .publish(e);
}

/// Emits an event when a burn limit per message is set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `burn_limit_per_message` - The maximum burn amount per message.
pub fn emit_set_burn_limit_per_message(
    e: &Env,
    local_token: &Address,
    burn_limit_per_message: i128,
) {
    SetBurnLimitPerMessage {
        local_token: local_token.clone(),
        burn_limit_per_message,
    }
    .publish(e);
}

/// Emits an event when a token decimal config is added.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `config` - The token decimal configuration.
pub fn emit_token_decimal_config_added(
    e: &Env,
    local_token: &Address,
    config: &TokenDecimalConfig,
) {
    TokenDecimalConfigAdded {
        local_token: local_token.clone(),
        token_decimal_config: config.clone(),
    }
    .publish(e);
}

/// Emits an event when a swap minter config is set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `config` - The swap minter configuration.
pub fn emit_swap_minter_config_set(e: &Env, local_token: &Address, config: &SwapMinterConfig) {
    SwapMinterConfigSet {
        local_token: local_token.clone(),
        swap_minter_config: config.clone(),
    }
    .publish(e);
}

/// Emits an event when a swap minter config is removed.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `local_token` - The address of the local token.
/// * `config` - The swap minter configuration that was removed.
pub fn emit_swap_minter_config_removed(e: &Env, local_token: &Address, config: &SwapMinterConfig) {
    SwapMinterConfigRemoved {
        local_token: local_token.clone(),
        swap_minter_config: config.clone(),
    }
    .publish(e);
}
