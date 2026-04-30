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
//! Receiver module for CCTP defining an interface for receiving messages on a destination domain.

use soroban_sdk::{contractclient, Address, Bytes, Env};

/// Receives messages on the destination domain and forwards them to contracts implementing MessageHandler.
#[contractclient(name = "ReceiverClient")]
pub trait Receiver {
    /// Receives an incoming message, validating the header and passing the body to a contract implementing MessageHandler.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller.
    /// * `message` - The message raw bytes to receive.
    /// * `signature` - The message signature.
    ///
    /// # Returns
    ///
    /// `true` if the message was received successfully.
    fn receive_message(e: &Env, caller: Address, message: Bytes, signature: Bytes) -> bool;
}
