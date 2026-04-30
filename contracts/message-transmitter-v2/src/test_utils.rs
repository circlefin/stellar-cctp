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

//! Test utilities for MessageTransmitter contract testing.
//!
//! This module provides helper functions for mocking authorization for
//! `send_message` and `receive_message` calls.

use soroban_sdk::{
    bytes,
    testutils::{MockAuth, MockAuthInvoke},
    Address, Bytes, BytesN, Env, IntoVal,
};

pub struct TestAttestationFixture {
    pub message: Bytes,
    pub attestation: Bytes,
}

/// Sets mock auth for a `send_message` call with the given parameters.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The MessageTransmitter contract address
/// * `caller` - The address that will be authorized to send the message
/// * `destination_domain` - The destination domain identifier
/// * `recipient` - The recipient address on the destination chain (as bytes32)
/// * `destination_caller` - The authorized caller on the destination chain (as bytes32)
/// * `min_finality_threshold` - The minimum finality threshold for attestation
/// * `message_body` - The message body bytes
#[allow(clippy::too_many_arguments)]
pub fn mock_send_message_auth(
    env: &Env,
    contract_id: &Address,
    caller: &Address,
    destination_domain: u32,
    recipient: BytesN<32>,
    destination_caller: BytesN<32>,
    min_finality_threshold: u32,
    message_body: Bytes,
) {
    env.mock_auths(&[MockAuth {
        address: caller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "send_message",
            args: (
                caller.clone(),
                destination_domain,
                recipient,
                destination_caller,
                min_finality_threshold,
                message_body,
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth for a `receive_message` call.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The MessageTransmitter contract address
/// * `caller` - The address that will be authorized to receive the message
/// * `message` - The message bytes
/// * `attestation` - The attestation bytes
pub fn mock_receive_message_auth(
    env: &Env,
    contract_id: &Address,
    caller: &Address,
    message: Bytes,
    attestation: Bytes,
) {
    env.mock_auths(&[MockAuth {
        address: caller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "receive_message",
            args: (caller.clone(), message, attestation).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth for a `set_max_message_body_size` call.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `contract_id` - The MessageTransmitter contract address
/// * `owner` - The owner address that will be authorized to set the max message body size
/// * `max_message_body_size` - The new max message body size
pub fn mock_set_max_message_body_size_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    max_message_body_size: u32,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_max_message_body_size",
            args: (max_message_body_size,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Creates a test message with full control over all fields.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `version` - The message format version
/// * `source_domain` - The source domain identifier
/// * `destination_domain` - The destination domain identifier
/// * `nonce` - The message nonce
/// * `sender` - The sender address as bytes32
/// * `recipient` - The recipient address as bytes32
/// * `destination_caller` - The destination caller as bytes32
/// * `min_finality_threshold` - The minimum finality threshold
/// * `finality_threshold_executed` - The executed finality threshold
/// * `message_body` - The message body bytes
///
/// # Returns
///
/// Serialized message as `Bytes`
#[allow(clippy::too_many_arguments)]
pub fn create_test_message(
    env: &Env,
    version: u32,
    source_domain: u32,
    destination_domain: u32,
    nonce: BytesN<32>,
    sender: BytesN<32>,
    recipient: BytesN<32>,
    destination_caller: BytesN<32>,
    min_finality_threshold: u32,
    finality_threshold_executed: u32,
    message_body: Bytes,
) -> Bytes {
    let mut out = Bytes::new(env);
    out.extend_from_array(&version.to_be_bytes());
    out.extend_from_array(&source_domain.to_be_bytes());
    out.extend_from_array(&destination_domain.to_be_bytes());
    out.append(&nonce.to_bytes());
    out.append(&sender.to_bytes());
    out.append(&recipient.to_bytes());
    out.append(&destination_caller.to_bytes());
    out.extend_from_array(&min_finality_threshold.to_be_bytes());
    out.extend_from_array(&finality_threshold_executed.to_be_bytes());
    out.append(&message_body);
    out
}

pub fn fixture_message_too_short(env: &Env) -> TestAttestationFixture {
    let message = bytes!(env, 0x1234567890abcdef);
    let attestation = bytes!(env, 0x3e2925985a4007c548ce4bfe8851045a45ff9bb437d85361036471df1e79872e1239566023d5890a29fa8d738bf72c7888784516d02e4495bc9623eb35d967ce1bc80eca93ce8aae9fa89e06943e2efdc32fe2f1df2b82548fdcccd92075b6263f137d0d568d46e20f109a7e22fd1a83f3789266c01893a92c3f7aadefc6b7022c1b);
    TestAttestationFixture {
        message,
        attestation,
    }
}

pub fn fixture_invalid_destination_domain_message(env: &Env) -> TestAttestationFixture {
    // sets destination domain to 9999
    let message = bytes!(env, 0x00000001000000060000270f1026816c3509483cac15c0b1cfdd2b533408e63bb2a62e88e470cfcb7ac199f20000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daaa65fc81d0fefa8860cb3b83f089b0224be8a6687b7ae49f594c0b9b4d7e938930000000000000000000000000000000000000000000000000000000000000000000003e8000007d000000001000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e15a5bcfb9572ff8d607fcda0a91cf850fd194be728b285a7b81540ac1d7735540000000000000000000000000000000000000000000000000000000000002710000000000000000000000000c5567a5e3370d4dbfb0540025078e283e36a363d000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000);
    let attestation = bytes!(env, 0xdab73c228e920800c0d9a4adb3ac6be6ac561011d0089d4013eaee90c557ea8837f1d1e40d0b29b3068cb0cc7a49805b33edf9e799dfa1decde16736573701901b39223f435b9fd2ea9be81f384773ec7c001fe75bea7a5ad1ceb2add8a6763ff377948d5711734571e72a96a31a1d12e7131cca9f387499cae9481720cdfb8d4a1c);
    TestAttestationFixture {
        message,
        attestation,
    }
}

pub fn fixture_invalid_destination_caller_message(env: &Env) -> TestAttestationFixture {
    // 0xdeadbeef as destination caller
    let message = bytes!(env, 0x0000000100000006000000021026816c3509483cac15c0b1cfdd2b533408e63bb2a62e88e470cfcb7ac199f20000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daaa65fc81d0fefa8860cb3b83f089b0224be8a6687b7ae49f594c0b9b4d7e93893deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef000003e8000007d000000001000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e15a5bcfb9572ff8d607fcda0a91cf850fd194be728b285a7b81540ac1d7735540000000000000000000000000000000000000000000000000000000000002710000000000000000000000000c5567a5e3370d4dbfb0540025078e283e36a363d000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000);
    let attestation = bytes!(env, 0x4ecba6600fa4fbd11d82d61eb1edea52911a4ce2c292b0d758b5026919a59dd1177444421c2395dc586549cfe8af47016a2f28f29dbee1bfeea4ed8477be7fb01cd9c2c644affaff8d63fb357f42ce777e867980d990830f6263c5545e605e304c3cf4b5faeb73d41092d01716b8075e74a8600b4f99d839a42af0152e2db705691c);
    TestAttestationFixture {
        message,
        attestation,
    }
}

pub fn fixture_valid_message(env: &Env) -> TestAttestationFixture {
    // Using the contract ID of the mock handler contract as the recipient (0x0000000000000000000000000000000000000000000000000000000000000008)
    let message = bytes!(env, 0x0000000100000006000000021026816c3509483cac15c0b1cfdd2b533408e63bb2a62e88e470cfcb7ac199f20000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa00000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000003e8000007d000000001000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e15a5bcfb9572ff8d607fcda0a91cf850fd194be728b285a7b81540ac1d7735540000000000000000000000000000000000000000000000000000000000002710000000000000000000000000c5567a5e3370d4dbfb0540025078e283e36a363d000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000);
    let attestation = bytes!(env, 0xba2ba8f51eac1b2f9554691e6de723d137c8f94fad04212645e2c67baeddcb52364f17c0e57ce6eb4243797b64ab40e18a7dbead260459ae061defd5b910d3651ccc81990ad6d37809ec6288195f9831377a7f5320cedcb51afe6e5f184bf19ce933ea7ad126cbdb85769d58120253aff0b6c5b758b7e6519519075b8d4321a4871c);
    TestAttestationFixture {
        message,
        attestation,
    }
}

pub fn fixture_not_enabled_attester_message(env: &Env) -> TestAttestationFixture {
    let message = bytes!(env, 0x0000000100000006000000021026816c3509483cac15c0b1cfdd2b533408e63bb2a62e88e470cfcb7ac199f20000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa00000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000003e8000007d000000001000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e15a5bcfb9572ff8d607fcda0a91cf850fd194be728b285a7b81540ac1d7735540000000000000000000000000000000000000000000000000000000000002710000000000000000000000000c5567a5e3370d4dbfb0540025078e283e36a363d000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000);
    // first signature is by 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC (not one of the enabled attesters)
    let attestation = bytes!(env, 0xb987f41b8e13ef93f69b126dcb00b0251d9075dbfee2f5ff10560754960ab7a97609c4ae44a4897f81b32a57beaeb1d27abd59db9f2a8d58c0892de055039ad01ccc81990ad6d37809ec6288195f9831377a7f5320cedcb51afe6e5f184bf19ce933ea7ad126cbdb85769d58120253aff0b6c5b758b7e6519519075b8d4321a4871c);
    TestAttestationFixture {
        message,
        attestation,
    }
}

pub fn fixture_invalid_version_message(env: &Env) -> TestAttestationFixture {
    // version is 99
    let message = bytes!(env, 0x0000006300000006000000021026816c3509483cac15c0b1cfdd2b533408e63bb2a62e88e470cfcb7ac199f20000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa00000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000000000003e8000007d000000001000000000000000000000000036cbd53842c5426634e7929541ec2318f3dcf7e15a5bcfb9572ff8d607fcda0a91cf850fd194be728b285a7b81540ac1d7735540000000000000000000000000000000000000000000000000000000000002710000000000000000000000000c5567a5e3370d4dbfb0540025078e283e36a363d000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000);
    let attestation = bytes!(env, 0xf6ee42254c6145106017d827faafa94a4b9bdab9a380d9139634db3814b250a50018b710dfd4269b79698c2679a586264f8f31b09b889a063545310f3cc44f471c72c500f4f200fedc272a8b829c4a1a8d4e08f1538eafe62bb100b73973910b076469cb514a6d3a464d243551186bfd4df52959ddf7aa9eddc6b5718064a6e4901b);
    TestAttestationFixture {
        message,
        attestation,
    }
}
