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
//! Relayer module for CCTP defining an interface for sending messages from a source domain.

use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env};

/// Sends messages from the source domain to the destination domain and recipient.
#[contractclient(name = "RelayerClient")]
pub trait Relayer {
    /// Sends an outgoing message from the source domain.
    ///
    /// WARNING: if `destination_caller` does not represent a valid address as BytesN<32>, then it will not be possible
    /// to broadcast the message on the destination domain. If set to all zeros (`BytesN::from_array(env, &[0u8; 32])`),
    /// anyone will be able to broadcast it. Using all zeros is preferred for use cases where a specific destination caller is not required.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller (message sender).
    /// * `destination_domain` - The destination domain identifier.
    /// * `recipient` - Address of message recipient on destination domain as bytesN<32>.
    /// * `destination_caller` - Allowed caller on destination domain (see above WARNING).
    /// * `min_finality_threshold` - The minimum finality threshold at which the message must be attested to.
    /// * `message_body` - Content of the message, as raw bytes.
    fn send_message(
        e: &Env,
        caller: Address,
        destination_domain: u32,
        recipient: BytesN<32>,
        destination_caller: BytesN<32>,
        min_finality_threshold: u32,
        message_body: Bytes,
    );
}
