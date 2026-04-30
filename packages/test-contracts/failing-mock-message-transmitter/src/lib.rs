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

//! Failing Mock MessageTransmitter contract for testing error propagation.
//!
//! This contract always fails on `send_message` and `receive_message` calls,
//! allowing tests to verify that calling contracts properly handle and propagate
//! MessageTransmitter failures.

#![no_std]

use cctp_interfaces::{Receiver, Relayer};
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};

/// Mock MessageTransmitter that always fails on send_message and receive_message.
/// Used to test that contracts properly propagate MessageTransmitter failures.
#[contract]
pub struct FailingMockMessageTransmitterContract;

#[contractimpl]
impl FailingMockMessageTransmitterContract {
    pub fn __constructor(_env: Env) {}
}

#[contractimpl]
impl Relayer for FailingMockMessageTransmitterContract {
    /// Mock send_message that always panics to simulate a failure.
    fn send_message(
        _e: &Env,
        _caller: Address,
        _destination_domain: u32,
        _recipient: BytesN<32>,
        _destination_caller: BytesN<32>,
        _min_finality_threshold: u32,
        _message_body: Bytes,
    ) {
        panic!("MessageTransmitter send_message failed");
    }
}

#[contractimpl]
impl Receiver for FailingMockMessageTransmitterContract {
    /// Mock receive_message that always panics to simulate a failure.
    fn receive_message(_e: &Env, _caller: Address, _message: Bytes, _attestation: Bytes) -> bool {
        panic!("MessageTransmitter receive_message failed");
    }
}
