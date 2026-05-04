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
pub mod storage;
#[cfg(test)]
mod test;
#[cfg(test)]
pub mod test_utils;

use soroban_sdk::{contracterror, contractevent, Address, Bytes, BytesN, Env};

// ################## ERRORS ##################

/// Errors for the Message Transmitter contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MessageTransmitterError {
    /// Cannot send a message to the local domain
    DestinationIsLocalDomain = 6900,
    /// Message body exceeds max allowed size
    MessageBodyTooLarge = 6901,
    /// Recipient cannot be zero address
    RecipientIsZero = 6902,
    /// Address type not recognized (unable to convert to bytes32 with Address::to_payload)
    AddressTypeNotRecognized = 6903,
    /// Message format is invalid (too short or malformed)
    InvalidMessageFormat = 6904,
    /// Message destination domain does not match local domain
    InvalidDestinationDomain = 6905,
    /// Caller is not the authorized destination caller for this message
    InvalidDestinationCaller = 6906,
    /// Message version does not match expected version
    InvalidMessageVersion = 6907,
    /// Nonce has already been used
    NonceAlreadyUsed = 6908,
    /// Message handler on recipient contract returned false
    HandleReceiveMessageFailed = 6909,
    /// The local domain has not been set
    LocalDomainNotSet = 6910,
    /// The version has not been set
    VersionNotSet = 6911,
    /// The max message body size has not been set
    MaxMessageBodySizeNotSet = 6912,
    /// No attesters provided
    NoAttesters = 6913,
}

/// Holds the validated and parsed fields from a received message.
pub struct ValidatedMessage {
    pub nonce: BytesN<32>,
    pub source_domain: u32,
    pub sender: BytesN<32>,
    pub recipient: BytesN<32>,
    pub finality_threshold_executed: u32,
    pub message_body: Bytes,
}

// ################## EVENTS ##################

/// Emitted when a new message is dispatched.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSent {
    pub message: Bytes,
}

/// Emitted when a message is received.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReceived {
    /// Caller on the destination domain
    #[topic]
    pub caller: Address,
    /// The source domain this message originated from
    pub source_domain: u32,
    /// The nonce unique to this message
    #[topic]
    pub nonce: BytesN<32>,
    /// The sender of this message
    pub sender: BytesN<32>,
    /// The finality at which message was attested to
    #[topic]
    pub finality_threshold_executed: u32,
    /// The message body bytes
    pub message_body: Bytes,
}

/// Emitted when the maximum message body size is updated.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaxMessageBodySizeUpdated {
    pub new_max_message_body_size: u32,
}

/// Emits a MessageSent event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `message` - The serialized message bytes.
///
/// # Events
///
/// * topics - `["message_sent"]`
/// * data - `[message: Bytes]`
pub fn emit_message_sent(e: &Env, message: &Bytes) {
    MessageSent {
        message: message.clone(),
    }
    .publish(e);
}

/// Emits a MessageReceived event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `caller` - The caller on the destination domain.
/// * `source_domain` - The source domain this message originated from.
/// * `nonce` - The nonce unique to this message.
/// * `sender` - The sender of this message.
/// * `finality_threshold_executed` - The finality at which message was attested to.
/// * `message_body` - The message body bytes.
///
/// # Events
///
/// * topics - `["message_received", caller: Address, nonce: BytesN<32>, finality_threshold_executed: u32]`
/// * data - `[source_domain: u32, sender: BytesN<32>, message_body: Bytes]`
#[allow(clippy::too_many_arguments)]
pub fn emit_message_received(
    e: &Env,
    caller: &Address,
    source_domain: u32,
    nonce: &BytesN<32>,
    sender: &BytesN<32>,
    finality_threshold_executed: u32,
    message_body: &Bytes,
) {
    MessageReceived {
        caller: caller.clone(),
        source_domain,
        nonce: nonce.clone(),
        sender: sender.clone(),
        finality_threshold_executed,
        message_body: message_body.clone(),
    }
    .publish(e);
}

/// Emits a MaxMessageBodySizeUpdated event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `new_max_message_body_size` - The maximum allowed message body size.
///
/// # Events
///
/// * topics - `["max_message_body_size_updated"]`
/// * data - `[new_max_message_body_size: u32]`
pub fn emit_max_message_body_size_updated(e: &Env, new_max_message_body_size: u32) {
    MaxMessageBodySizeUpdated {
        new_max_message_body_size,
    }
    .publish(e);
}
