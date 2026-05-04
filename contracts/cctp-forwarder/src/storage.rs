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

use cctp_interfaces::ReceiverClient;
use cctp_roles::token_controller::TokenControllerClient;
use soroban_sdk::{contracttype, panic_with_error, token, Address, Bytes, BytesN, Env};

use crate::CctpForwarderError;

/// Storage keys for the CCTP Forwarder contract.
#[contracttype]
pub enum CctpForwarderStorageKey {
    /// The address of the MessageTransmitter contract
    MessageTransmitter,
    /// The address of the TokenMessengerMinter contract
    TokenMessengerMinter,
    /// The expected message version
    ExpectedMessageVersion,
    /// The expected burn message version
    ExpectedBurnMessageVersion,
}

/// Returns the message transmitter address.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The message transmitter address.
///
/// # Errors
///
/// * [`CctpForwarderError::MessageTransmitterNotSet`] – If the message transmitter has not been set.
pub fn get_message_transmitter(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&CctpForwarderStorageKey::MessageTransmitter)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::MessageTransmitterNotSet))
}

/// Returns the token messenger minter address.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The token messenger minter address.
///
/// # Errors
///
/// * [`CctpForwarderError::TokenMessengerMinterNotSet`] – If the token messenger minter has not been set.
pub fn get_token_messenger_minter(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&CctpForwarderStorageKey::TokenMessengerMinter)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::TokenMessengerMinterNotSet))
}

/// Returns the expected message version.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The expected message version.
///
/// # Errors
///
/// * [`CctpForwarderError::ExpectedMessageVersionNotSet`] – If the expected message version has not been set.
pub fn get_expected_msg_version(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&CctpForwarderStorageKey::ExpectedMessageVersion)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::ExpectedMessageVersionNotSet))
}

/// Returns the expected burn message version.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
///
/// # Returns
///
/// The expected burn message version.
///
/// # Errors
///
/// * [`CctpForwarderError::ExpectedBurnMessageVersionNotSet`] – If the expected burn message version has not been set.
pub fn get_expected_burn_msg_version(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&CctpForwarderStorageKey::ExpectedBurnMessageVersion)
        .unwrap_or_else(|| {
            panic_with_error!(e, CctpForwarderError::ExpectedBurnMessageVersionNotSet)
        })
}

/// Resolves the local token address from TokenMessengerMinter.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `source_domain` - The source domain of the CCTP message.
/// * `burn_token` - The burn token address from the source domain.
///
/// # Returns
///
/// The local token address corresponding to the burn token.
///
/// # Errors
///
/// * [`CctpForwarderError::LocalTokenNotResolved`] – If the local token could not be resolved.
pub fn get_local_token(e: &Env, source_domain: u32, burn_token: &BytesN<32>) -> Address {
    let tmm_address = get_token_messenger_minter(e);
    let tmm_client = TokenControllerClient::new(e, &tmm_address);

    tmm_client
        .get_local_token(&source_domain, burn_token)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::LocalTokenNotResolved))
}

/// Calls receive_message on the MessageTransmitter and returns the minted amount.
///
/// This function compares the contract's token balance before and after calling
/// `receive_message` to determine how many tokens were minted.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `contract_address` - The address of this contract (the mint recipient).
/// * `local_token` - The local token address that will be minted.
/// * `message` - The CCTP message bytes.
/// * `attestation` - The attestation bytes for the message.
///
/// # Returns
///
/// The amount of tokens minted.
///
/// # Errors
///
/// * [`CctpForwarderError::NoTokensMinted`] – If no tokens were minted by receive_message.
pub fn mint_through_cctp(
    e: &Env,
    contract_address: &Address,
    local_token: &Address,
    message: &Bytes,
    attestation: &Bytes,
) -> i128 {
    let token_client = token::TokenClient::new(e, local_token);
    let starting_balance = token_client.balance(contract_address);

    let message_transmitter = get_message_transmitter(e);
    let mt_client = ReceiverClient::new(e, &message_transmitter);
    mt_client.receive_message(contract_address, message, attestation);

    let ending_balance = token_client.balance(contract_address);
    let amount_minted = ending_balance
        .checked_sub(starting_balance)
        .unwrap_or_else(|| panic_with_error!(e, CctpForwarderError::NoTokensMinted));

    if amount_minted <= 0 {
        panic_with_error!(e, CctpForwarderError::NoTokensMinted);
    }

    amount_minted
}
