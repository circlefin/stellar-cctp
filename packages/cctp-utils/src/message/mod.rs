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
//! Message V2 module
//!
//! This module provides functionality for serializing and deserializing CCTP V2 messages.
//!
//! /// The message body is dynamically-sized to support custom message body
//! formats. Other fields must be fixed-size to avoid hash collisions.
//! Padding: uintNN fields are left-padded, and bytesNN fields are right-padded.
//!
//! Field                        Bytes      Type       Index
//! version                      4          uint32     0
//! sourceDomain                 4          uint32     4
//! destinationDomain            4          uint32     8
//! nonce                        32         bytes32    12
//! sender                       32         bytes32    44
//! recipient                    32         bytes32    76
//! destinationCaller            32         bytes32    108
//! minFinalityThreshold         4          uint32     140
//! finalityThresholdExecuted    4          uint32     144
//! messageBody                  dynamic    bytes      148

#[cfg(test)]
mod test;

use crate::bytes::{self, ByteReadError};
use soroban_sdk::{Bytes, BytesN, Env};

// Field indices in the serialized message
const VERSION_INDEX: u32 = 0;
const SOURCE_DOMAIN_INDEX: u32 = 4;
const DESTINATION_DOMAIN_INDEX: u32 = 8;
const NONCE_INDEX: u32 = 12;
const SENDER_INDEX: u32 = 44;
const RECIPIENT_INDEX: u32 = 76;
const DESTINATION_CALLER_INDEX: u32 = 108;
const MIN_FINALITY_THRESHOLD_INDEX: u32 = 140;
const FINALITY_THRESHOLD_EXECUTED_INDEX: u32 = 144;
const MESSAGE_BODY_INDEX: u32 = 148;

// Empty/default values
const EMPTY_NONCE: [u8; 32] = [0u8; 32];
const EMPTY_FINALITY_THRESHOLD_EXECUTED: u32 = 0;

/// Error types for message operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageV2Error {
    /// The message is too short to contain all required fields
    MessageTooShort = 6500,
    /// Failed to read a field from the message
    FieldReadError = 6501,
}

impl From<ByteReadError> for MessageV2Error {
    fn from(err: ByteReadError) -> Self {
        match err {
            ByteReadError::OutOfBounds => MessageV2Error::FieldReadError,
            ByteReadError::ValueTooLarge => MessageV2Error::FieldReadError,
        }
    }
}

/// Represents a CCTP V2 message with all its fields.
///
/// This struct can be used to construct new messages or parse existing ones.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageV2 {
    /// The version of the message format
    pub version: u32,
    /// Domain of the source chain
    pub source_domain: u32,
    /// Domain of the destination chain
    pub destination_domain: u32,
    /// Unique nonce for the message
    pub nonce: BytesN<32>,
    /// Address of sender on source chain (as bytes32)
    pub sender: BytesN<32>,
    /// Address of recipient on destination chain (as bytes32)
    pub recipient: BytesN<32>,
    /// Address of caller on destination chain (as bytes32)
    pub destination_caller: BytesN<32>,
    /// Minimum finality at which the message should be attested to
    pub min_finality_threshold: u32,
    /// Finality threshold at which the message was executed
    pub finality_threshold_executed: u32,
    /// Raw bytes of message body
    pub message_body: Bytes,
}

impl MessageV2 {
    /// Formats a V2 message for relay with an empty nonce and finality threshold executed.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment
    /// * `version` - The version of the message format
    /// * `source_domain` - Domain of the source chain
    /// * `destination_domain` - Domain of the destination chain
    /// * `sender` - Address of sender on source chain (as bytes32)
    /// * `recipient` - Address of recipient on destination chain (as bytes32)
    /// * `destination_caller` - Address of caller on destination chain (as bytes32)
    /// * `min_finality_threshold` - Minimum finality at which the message should be attested to
    /// * `message_body` - Raw bytes of message body
    ///
    /// # Returns
    ///
    /// Serialized V2 message as `Bytes`
    #[allow(clippy::too_many_arguments)]
    pub fn format_for_relay(
        env: &Env,
        version: u32,
        source_domain: u32,
        destination_domain: u32,
        sender: BytesN<32>,
        recipient: BytesN<32>,
        destination_caller: BytesN<32>,
        min_finality_threshold: u32,
        message_body: Bytes,
    ) -> Bytes {
        let message = Self {
            version,
            source_domain,
            destination_domain,
            nonce: BytesN::from_array(env, &EMPTY_NONCE),
            sender,
            recipient,
            destination_caller,
            min_finality_threshold,
            finality_threshold_executed: EMPTY_FINALITY_THRESHOLD_EXECUTED,
            message_body,
        };
        message.serialize(env)
    }

    /// Serializes the V2 message to bytes.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment
    ///
    /// # Returns
    ///
    /// The serialized V2 message as `Bytes`
    pub fn serialize(&self, env: &Env) -> Bytes {
        let mut out = Bytes::new(env);

        out.extend_from_array(&self.version.to_be_bytes());
        out.extend_from_array(&self.source_domain.to_be_bytes());
        out.extend_from_array(&self.destination_domain.to_be_bytes());
        out.append(&self.nonce.to_bytes());
        out.append(&self.sender.to_bytes());
        out.append(&self.recipient.to_bytes());
        out.append(&self.destination_caller.to_bytes());
        out.extend_from_array(&self.min_finality_threshold.to_be_bytes());
        out.extend_from_array(&self.finality_threshold_executed.to_be_bytes());
        out.append(&self.message_body);

        out
    }

    /// Validates the V2 message format.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// `Ok(())` if the V2 message is valid, or an error if it is an invalid length
    ///
    /// # Errors
    ///
    /// * [`MessageV2Error::MessageTooShort`] - If the message is shorter than the minimum required length
    pub fn validate_format(data: &Bytes) -> Result<(), MessageV2Error> {
        if data.len() < MESSAGE_BODY_INDEX {
            return Err(MessageV2Error::MessageTooShort);
        }
        Ok(())
    }

    /// Returns the version from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The version, or an error if it cannot be read
    pub fn get_version(data: &Bytes) -> Result<u32, MessageV2Error> {
        Ok(bytes::read_u32(data, VERSION_INDEX)?)
    }

    /// Returns the source domain from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The source domain, or an error if it cannot be read
    pub fn get_source_domain(data: &Bytes) -> Result<u32, MessageV2Error> {
        Ok(bytes::read_u32(data, SOURCE_DOMAIN_INDEX)?)
    }

    /// Returns the destination domain from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The destination domain, or an error if it cannot be read
    pub fn get_destination_domain(data: &Bytes) -> Result<u32, MessageV2Error> {
        Ok(bytes::read_u32(data, DESTINATION_DOMAIN_INDEX)?)
    }

    /// Returns the nonce from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The nonce as `BytesN<32>`, or an error if it cannot be read
    pub fn get_nonce(data: &Bytes) -> Result<BytesN<32>, MessageV2Error> {
        Ok(bytes::read_bytes32(data, NONCE_INDEX)?)
    }

    /// Returns the sender from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The sender as `BytesN<32>`, or an error if it cannot be read
    pub fn get_sender(data: &Bytes) -> Result<BytesN<32>, MessageV2Error> {
        Ok(bytes::read_bytes32(data, SENDER_INDEX)?)
    }

    /// Returns the recipient from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The recipient as `BytesN<32>`, or an error if it cannot be read
    pub fn get_recipient(data: &Bytes) -> Result<BytesN<32>, MessageV2Error> {
        Ok(bytes::read_bytes32(data, RECIPIENT_INDEX)?)
    }

    /// Returns the destination caller from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The destination caller as `BytesN<32>`, or an error if it cannot be read
    pub fn get_destination_caller(data: &Bytes) -> Result<BytesN<32>, MessageV2Error> {
        Ok(bytes::read_bytes32(data, DESTINATION_CALLER_INDEX)?)
    }

    /// Returns the minimum finality threshold from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The minimum finality threshold, or an error if it cannot be read
    pub fn get_min_finality_threshold(data: &Bytes) -> Result<u32, MessageV2Error> {
        Ok(bytes::read_u32(data, MIN_FINALITY_THRESHOLD_INDEX)?)
    }

    /// Returns the finality threshold executed from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes
    ///
    /// # Returns
    ///
    /// The finality threshold executed, or an error if it cannot be read
    pub fn get_finality_threshold_executed(data: &Bytes) -> Result<u32, MessageV2Error> {
        Ok(bytes::read_u32(data, FINALITY_THRESHOLD_EXECUTED_INDEX)?)
    }

    /// Returns the message body from serialized V2 message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 message bytes (must be at least 148 bytes)
    ///
    /// # Returns
    ///
    /// The message body as `Bytes`
    pub fn get_message_body(data: &Bytes) -> Bytes {
        data.slice(MESSAGE_BODY_INDEX..data.len())
    }
}
