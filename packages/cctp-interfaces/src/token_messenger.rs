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
//! TokenMessenger module for CCTP defining an interface for initiating cross-chain token transfers.

// Allowing more than 7 arguments so we can adhere to the CCTP deposit_for_burn and deposit_for_burn_with_hook specifications.
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env};

/// Interface for initiating cross-chain token transfers.
///
/// Implemented by TokenMessengerMinterContract. Allows external contracts
/// to deposit tokens for burning and relay to a destination domain.
#[contractclient(name = "TokenMessengerClient")]
pub trait TokenMessenger {
    /// Deposits and burns tokens from sender to be minted on destination domain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller.
    /// * `amount` - Amount of tokens to burn.
    /// * `destination_domain` - Destination domain to receive message on.
    /// * `mint_recipient` - Address of mint recipient on destination domain, as BytesN<32>.
    /// * `burn_token` - Token to burn `amount` of, on local domain.
    /// * `destination_caller` - Authorized caller on the destination domain, as BytesN<32>.
    ///   If BytesN<32>(0), any address can broadcast the message.
    /// * `max_fee` - Maximum fee to pay on the destination domain, specified in units of burn_token.
    /// * `min_finality_threshold` - The minimum finality at which a burn message will be attested to.
    fn deposit_for_burn(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
    );

    /// Deposits and burns tokens from sender to be minted on destination domain,
    /// with hook data to append to burn message for interpretation on destination domain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller.
    /// * `amount` - Amount of tokens to burn.
    /// * `destination_domain` - Destination domain to receive message on.
    /// * `mint_recipient` - Address of mint recipient on destination domain, as BytesN<32>.
    /// * `burn_token` - Token to burn `amount` of, on local domain.
    /// * `destination_caller` - Authorized caller on the destination domain, as BytesN<32>.
    ///   If BytesN<32>(0), any address can broadcast the message.
    /// * `max_fee` - Maximum fee to pay on the destination domain, specified in units of burn_token.
    /// * `min_finality_threshold` - The minimum finality at which a burn message will be attested to.
    /// * `hook_data` - Hook data to append to burn message for interpretation on destination domain.
    fn deposit_for_burn_with_hook(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
        hook_data: Bytes,
    );
}
