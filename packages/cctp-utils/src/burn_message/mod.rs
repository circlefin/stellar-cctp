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
//! Burn Message V2 module
//!
//! This module provides functionality for serializing and deserializing CCTP V2 burn messages.
//! Burn messages are used by TokenMessenger to encode token burn/mint operations.
//!
//! BurnMessageV2 format:
//!
//! Field                 Bytes      Type       Index
//! version               4          uint32     0
//! burnToken             32         bytes32    4
//! mintRecipient         32         bytes32    36
//! amount                32         uint256    68
//! messageSender         32         bytes32    100
//! maxFee                32         uint256    132
//! feeExecuted           32         uint256    164
//! expirationBlock       32         uint256    196
//! hookData              dynamic    bytes      228

#[cfg(test)]
mod test;

use crate::bytes::{self, ByteReadError};
use soroban_sdk::{Bytes, BytesN, Env, U256};

// Field indices in the serialized burn message
const VERSION_INDEX: u32 = 0;
const BURN_TOKEN_INDEX: u32 = 4;
const MINT_RECIPIENT_INDEX: u32 = 36;
const AMOUNT_INDEX: u32 = 68;
const MESSAGE_SENDER_INDEX: u32 = 100;
const MAX_FEE_INDEX: u32 = 132;
const FEE_EXECUTED_INDEX: u32 = 164;
const EXPIRATION_BLOCK_INDEX: u32 = 196;
const HOOK_DATA_INDEX: u32 = 228;

// Empty/default values
const EMPTY_FEE_EXECUTED: u128 = 0;
const EMPTY_EXPIRATION_BLOCK: u32 = 0;

/// Error types for burn message operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BurnMessageV2Error {
    /// The burn message is too short to contain all required fields
    MessageTooShort = 6600,
    /// Failed to read a field from the message
    FieldReadError = 6601,
    /// A u256 value exceeds the maximum representable value for the target type
    ValueTooLarge = 6602,
}

impl From<ByteReadError> for BurnMessageV2Error {
    fn from(err: ByteReadError) -> Self {
        match err {
            ByteReadError::OutOfBounds => BurnMessageV2Error::FieldReadError,
            ByteReadError::ValueTooLarge => BurnMessageV2Error::ValueTooLarge,
        }
    }
}

/// Represents a CCTP V2 burn message with all its fields.
///
/// This struct can be used to construct new burn messages or parse existing ones.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnMessageV2 {
    /// The version of the burn message format
    pub version: u32,
    /// Address of the token burned on the source domain (as bytes32)
    pub burn_token: BytesN<32>,
    /// Address of the mint recipient on the destination domain (as bytes32)
    pub mint_recipient: BytesN<32>,
    /// Amount of tokens burned
    pub amount: U256,
    /// Address of the message sender on the source domain (as bytes32)
    pub message_sender: BytesN<32>,
    /// Maximum fee to be paid on the destination domain (in units of burn token)
    pub max_fee: U256,
    /// Fee executed on the destination domain
    pub fee_executed: U256,
    /// Block number after which the message expires (0 means no expiration)
    pub expiration_block: U256,
    /// Optional hook data for processing on the destination domain
    pub hook_data: Bytes,
}

impl BurnMessageV2 {
    /// Formats a V2 burn message for relay with empty fee_executed and expiration_block.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment
    /// * `version` - The version of the burn message format
    /// * `burn_token` - Address of the token burned on the source domain (as bytes32)
    /// * `mint_recipient` - Address of the mint recipient on the destination domain (as bytes32)
    /// * `amount` - Amount of tokens burned
    /// * `message_sender` - Address of the message sender on the source domain (as bytes32)
    /// * `max_fee` - Maximum fee to be paid on the destination domain
    /// * `hook_data` - Optional hook data for processing on the destination domain
    ///
    /// # Returns
    ///
    /// Serialized burn message as `Bytes`
    #[allow(clippy::too_many_arguments)]
    pub fn format_for_relay(
        env: &Env,
        version: u32,
        burn_token: BytesN<32>,
        mint_recipient: BytesN<32>,
        amount: U256,
        message_sender: BytesN<32>,
        max_fee: U256,
        hook_data: Bytes,
    ) -> Bytes {
        let message = Self {
            version,
            burn_token,
            mint_recipient,
            amount,
            message_sender,
            max_fee,
            fee_executed: U256::from_u128(env, EMPTY_FEE_EXECUTED),
            expiration_block: U256::from_u32(env, EMPTY_EXPIRATION_BLOCK),
            hook_data,
        };
        message.serialize(env)
    }

    /// Serializes the V2 burn message to bytes.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment
    ///
    /// # Returns
    ///
    /// The serialized V2 burn message as `Bytes`
    pub fn serialize(&self, env: &Env) -> Bytes {
        let mut out = Bytes::new(env);

        out.extend_from_array(&self.version.to_be_bytes());
        out.append(&self.burn_token.to_bytes());
        out.append(&self.mint_recipient.to_bytes());
        out.append(&self.amount.to_be_bytes());
        out.append(&self.message_sender.to_bytes());
        out.append(&self.max_fee.to_be_bytes());
        out.append(&self.fee_executed.to_be_bytes());
        out.append(&self.expiration_block.to_be_bytes());
        out.append(&self.hook_data);

        out
    }

    /// Validates the V2 burn message format.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// `Ok(())` if the V2 burn message is valid, or an error if it is an invalid length
    ///
    /// # Errors
    ///
    /// * [`BurnMessageError::MessageTooShort`] - If the message is shorter than the minimum required length
    pub fn validate_format(data: &Bytes) -> Result<(), BurnMessageV2Error> {
        if data.len() < HOOK_DATA_INDEX {
            return Err(BurnMessageV2Error::MessageTooShort);
        }
        Ok(())
    }

    /// Returns the version from serialized V2 burn message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The version, or an error if it cannot be read
    pub fn get_version(data: &Bytes) -> Result<u32, BurnMessageV2Error> {
        Ok(bytes::read_u32(data, VERSION_INDEX)?)
    }

    /// Returns the burn token from serialized V2 burn message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The burn token as `BytesN<32>`, or an error if it cannot be read
    pub fn get_burn_token(data: &Bytes) -> Result<BytesN<32>, BurnMessageV2Error> {
        Ok(bytes::read_bytes32(data, BURN_TOKEN_INDEX)?)
    }

    /// Returns the mint recipient from serialized V2 burn message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The mint recipient as `BytesN<32>`, or an error if it cannot be read
    pub fn get_mint_recipient(data: &Bytes) -> Result<BytesN<32>, BurnMessageV2Error> {
        Ok(bytes::read_bytes32(data, MINT_RECIPIENT_INDEX)?)
    }

    /// Returns the amount from serialized V2 burn message bytes as a U256.
    ///
    /// The raw U256 value from the message is returned without conversion.
    /// Use [`bytes::u256_to_positive_i128`] to convert to i128 with overflow checking.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The amount as `U256`, or an error if it cannot be read
    pub fn get_amount(e: &Env, data: &Bytes) -> Result<U256, BurnMessageV2Error> {
        Ok(bytes::read_u256(e, data, AMOUNT_INDEX)?)
    }

    /// Returns the message sender from serialized V2 burn message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The message sender as `BytesN<32>`, or an error if it cannot be read
    pub fn get_message_sender(data: &Bytes) -> Result<BytesN<32>, BurnMessageV2Error> {
        Ok(bytes::read_bytes32(data, MESSAGE_SENDER_INDEX)?)
    }

    /// Returns the max fee from serialized V2 burn message bytes as a U256.
    ///
    /// The raw U256 value from the message is returned without conversion.
    /// Use [`bytes::u256_to_positive_i128`] to convert to i128 with overflow checking.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The max fee as `U256`, or an error if it cannot be read
    pub fn get_max_fee(e: &Env, data: &Bytes) -> Result<U256, BurnMessageV2Error> {
        Ok(bytes::read_u256(e, data, MAX_FEE_INDEX)?)
    }

    /// Returns the fee executed from serialized V2 burn message bytes as a U256.
    ///
    /// The raw U256 value from the message is returned without conversion.
    /// Use [`bytes::u256_to_positive_i128`] to convert to i128 with overflow checking.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The fee executed as `U256`, or an error if it cannot be read
    pub fn get_fee_executed(e: &Env, data: &Bytes) -> Result<U256, BurnMessageV2Error> {
        Ok(bytes::read_u256(e, data, FEE_EXECUTED_INDEX)?)
    }

    /// Returns the expiration block from serialized V2 burn message bytes as a U256.
    ///
    /// The caller is responsible for converting to `u32` and checking for overflow
    /// if a bounded value is required.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized V2 burn message bytes
    ///
    /// # Returns
    ///
    /// The expiration block as a U256, or an error if the bytes cannot be read.
    ///
    /// # Errors
    ///
    /// * [`BurnMessageV2Error::ReadError`] - If the field cannot be read from the message bytes.
    pub fn get_expiration_block(e: &Env, data: &Bytes) -> Result<U256, BurnMessageV2Error> {
        Ok(bytes::read_u256(e, data, EXPIRATION_BLOCK_INDEX)?)
    }

    /// Returns the hook data from serialized V2 burn message bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized burn message bytes (must be at least 228 bytes)
    ///
    /// # Returns
    ///
    /// The hook data as `Bytes`
    pub fn get_hook_data(data: &Bytes) -> Bytes {
        data.slice(HOOK_DATA_INDEX..data.len())
    }
}
