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
#![no_std]

#[cfg(feature = "entry-points")]
mod contract;
pub mod message;
pub mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contracterror, contractevent, Address, Env, MuxedAddress};

// ################## ERRORS ##################

/// Errors for the CCTP Forwarder contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CctpForwarderError {
    /// Hook data is too short (minimum 87 bytes for shortest strkey)
    HookDataTooShort = 7300,
    /// The mintRecipient in the message is not this CctpForwarder contract
    InvalidMintRecipient = 7301,
    /// The recipient in the message is not the configured TokenMessengerMinter
    InvalidRecipient = 7302,
    /// The message format is invalid
    InvalidMessageFormat = 7303,
    /// The message version is unsupported
    UnsupportedMessageVersion = 7304,
    /// The burn message format is invalid
    InvalidBurnMessageFormat = 7305,
    /// The burn message version is unsupported
    UnsupportedBurnMessageVersion = 7306,
    /// Failed to parse the forward recipient strkey
    InvalidForwardRecipient = 7307,
    /// The message transmitter address is not set
    MessageTransmitterNotSet = 7308,
    /// The token messenger minter address is not set
    TokenMessengerMinterNotSet = 7309,
    /// The expected message version is not set
    ExpectedMessageVersionNotSet = 7310,
    /// The expected burn message version is not set
    ExpectedBurnMessageVersionNotSet = 7311,
    /// The local token could not be resolved
    LocalTokenNotResolved = 7312,
    /// The hook data version is unsupported
    InvalidHookVersion = 7313,
    /// No tokens were minted by receive_message
    NoTokensMinted = 7314,
    /// The message transmitter address has already been set
    MessageTransmitterAlreadySet = 7315,
    /// The token messenger minter address has already been set
    TokenMessengerMinterAlreadySet = 7316,
    /// The expected message version has already been set
    ExpectedMessageVersionAlreadySet = 7317,
    /// The expected burn message version has already been set
    ExpectedBurnMessageVersionAlreadySet = 7318,
}

// ################## EVENTS ##################

/// Emitted when a mint and forward operation is completed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintAndForward {
    /// The address that received the forwarded tokens
    pub forward_recipient: MuxedAddress,
    /// The token address that was forwarded
    pub token: Address,
    /// The amount of tokens forwarded
    pub amount: i128,
}

/// Emits a MintAndForward event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `forward_recipient` - The address that received the forwarded tokens.
/// * `token` - The token address that was forwarded.
/// * `amount` - The amount of tokens forwarded.
///
/// # Events
///
/// * topics - `["mint_and_forward"]`
/// * data - `[forward_recipient: MuxedAddress, token: Address, amount: i128]`
pub fn emit_mint_and_forward(
    e: &Env,
    forward_recipient: &MuxedAddress,
    token: &Address,
    amount: i128,
) {
    MintAndForward {
        forward_recipient: forward_recipient.clone(),
        token: token.clone(),
        amount,
    }
    .publish(e);
}
