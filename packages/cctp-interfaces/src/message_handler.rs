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
//! MessageHandler module for CCTP defining an interface for handling received messages.

use soroban_sdk::{contractclient, Bytes, BytesN, Env};

/// Handles messages on the destination domain, forwarded from a Receiver.
#[contractclient(name = "MessageHandlerClient")]
pub trait MessageHandler {
    /// Handles an incoming finalized message from a Receiver.
    ///
    /// Finalized messages have finality threshold values greater than or equal to 2000.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `source_domain` - The source domain of the message.
    /// * `sender` - The sender of the message on the source domain (as bytes32).
    /// * `finality_threshold_executed` - The finality level at which the message was attested to.
    /// * `message_body` - The raw bytes of the message body.
    ///
    /// # Returns
    ///
    /// `true` if the message was handled successfully, `false` otherwise.
    fn handle_recv_finalized_message(
        e: &Env,
        source_domain: u32,
        sender: BytesN<32>,
        finality_threshold_executed: u32,
        message_body: Bytes,
    ) -> bool;

    /// Handles an incoming unfinalized message from a Receiver.
    ///
    /// Unfinalized messages have finality threshold values less than 2000.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `source_domain` - The source domain of the message.
    /// * `sender` - The sender of the message on the source domain (as bytes32).
    /// * `finality_threshold_executed` - The finality level at which the message was attested to.
    /// * `message_body` - The raw bytes of the message body.
    ///
    /// # Returns
    ///
    /// `true` if the message was handled successfully, `false` otherwise
    fn handle_recv_unfinalized_message(
        e: &Env,
        source_domain: u32,
        sender: BytesN<32>,
        finality_threshold_executed: u32,
        message_body: Bytes,
    ) -> bool;
}
