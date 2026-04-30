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
use crate::contract::MessageTransmitterV2ContractInitParams;
use crate::test_utils::{
    create_test_message, fixture_invalid_destination_caller_message,
    fixture_invalid_destination_domain_message, fixture_invalid_version_message,
    fixture_message_too_short, fixture_not_enabled_attester_message, fixture_valid_message,
    mock_receive_message_auth, mock_send_message_auth, mock_set_max_message_body_size_auth,
};

use super::contract::{MessageTransmitterV2Contract, MessageTransmitterV2ContractClient};
use cctp_roles::test_utils::attestable::{
    mock_disable_attester_auth, mock_enable_attester_auth, mock_set_signature_threshold_auth,
    mock_update_attester_manager_auth,
};
use cctp_roles::test_utils::CctpEventAssertions;
use cctp_utils::MessageV2;
use common_roles::test_utils::pausable::mock_pause_auth;
use common_roles::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, vec, Address, Bytes, BytesN, Env,
};

/// A minimal mock contract that implements the MessageHandler interface.
/// It simply returns true from handler methods without any storage operations.
#[contract]
pub struct MockMessageHandler;

#[contractimpl]
impl MockMessageHandler {
    pub fn handle_recv_finalized_message(
        _e: &Env,
        _remote_domain: u32,
        _sender: BytesN<32>,
        _finality_threshold_executed: u32,
        _message_body: Bytes,
    ) -> bool {
        true
    }

    pub fn handle_recv_unfinalized_message(
        _e: &Env,
        _remote_domain: u32,
        _sender: BytesN<32>,
        _finality_threshold_executed: u32,
        _message_body: Bytes,
    ) -> bool {
        true
    }
}

// Reuse existing test WASM from the roles package
const UPGRADE_V2_WASM: &[u8] = include_bytes!("../../../testdata/upgrade_v2.wasm");

// Test configuration constants
const TEST_LOCAL_DOMAIN: u32 = 2;
const TEST_VERSION: u32 = 1;
const TEST_MAX_MESSAGE_BODY_SIZE: u32 = 8192;
// address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
const TEST_ATTESTER_ADDRESS_1: [u8; 20] = [
    0xf3, 0x9f, 0xd6, 0xe5, 0x1a, 0xad, 0x88, 0xf6, 0xf4, 0xce, 0x6a, 0xb8, 0x82, 0x72, 0x79, 0xcf,
    0xff, 0xb9, 0x22, 0x66,
];

// address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
const TEST_ATTESTER_ADDRESS_2: [u8; 20] = [
    0x70, 0x99, 0x79, 0x70, 0xc5, 0x18, 0x12, 0xdc, 0x3a, 0x01, 0x0c, 0x7d, 0x01, 0xb5, 0x0e, 0x0d,
    0x17, 0xdc, 0x79, 0xc8,
];
struct TestContext {
    env: Env,
    contract_id: Address,
    owner: Address,
    pauser: Address,
    rescuer: Address,
    attester_manager: Address,
    attester1: BytesN<20>,
    attester2: BytesN<20>,
    admin: Address,
}

impl TestContext {
    fn client(&self) -> MessageTransmitterV2ContractClient<'_> {
        MessageTransmitterV2ContractClient::new(&self.env, &self.contract_id)
    }
}

fn setup_contract() -> TestContext {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let attester1 = BytesN::from_array(&env, &TEST_ATTESTER_ADDRESS_1);
    let attester2 = BytesN::from_array(&env, &TEST_ATTESTER_ADDRESS_2);
    let admin = Address::generate(&env);

    let contract_id = env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer: rescuer.clone(),
            attester_manager: attester_manager.clone(),
            attesters: vec![&env, attester1.clone(), attester2.clone()],
            signature_threshold: 2,
            admin: admin.clone(),
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );

    TestContext {
        env,
        contract_id,
        owner,
        pauser,
        rescuer,
        attester_manager,
        attester1,
        attester2,
        admin,
    }
}

#[test]
fn test_constructor_initializes_state_and_emits_events() {
    let ctx = setup_contract();
    let client = ctx.client();

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());

    events.assert_event_count(9);
    events.assert_ownership_transfer_completed(&ctx.owner);
    events.assert_pauser_changed(&ctx.pauser);
    events.assert_rescuer_changed(&ctx.rescuer);
    events.assert_attester_manager_updated(&None, &ctx.attester_manager);
    events.assert_attester_enabled(&ctx.attester1);
    events.assert_attester_enabled(&ctx.attester2);
    events.assert_signature_threshold_updated(Some(0), 2);
    events.assert_admin_changed(None, &ctx.admin);
    events.assert_max_message_body_size_updated(&TEST_MAX_MESSAGE_BODY_SIZE);

    assert_eq!(client.get_owner(), Some(ctx.owner.clone()));
    assert_eq!(client.get_pauser(), Some(ctx.pauser.clone()));
    assert_eq!(client.get_rescuer(), Some(ctx.rescuer.clone()));
    assert_eq!(
        client.get_attester_manager(),
        Some(ctx.attester_manager.clone())
    );
    assert_eq!(client.get_enabled_attester(&0), ctx.attester1);
    assert_eq!(client.get_enabled_attester(&1), ctx.attester2);
    assert_eq!(client.get_signature_threshold(), Some(2));
    assert!(!client.paused());
}

#[test]
fn test_message_transmitter_is_ownable() {
    let ctx = setup_contract();
    let client = ctx.client();

    common_roles::assert_contract_is_ownable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_message_transmitter_is_pausable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_pausable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.pauser,
        &ctx.owner
    );
}

#[test]
fn test_message_transmitter_is_rescuable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_rescuable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_contract_is_manageable() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Test full upgrade flow including authorization checks, actual upgrade, and state persistence
    common_roles::assert_contract_is_manageable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.owner,
        &ctx.admin,
        UPGRADE_V2_WASM
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_send_message_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let dest_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let msg_body = Bytes::from_array(&ctx.env, &[]);

    client.send_message(&caller, &1u32, &recipient, &dest_caller, &0u32, &msg_body);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_receive_message_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    let message = Bytes::from_array(&ctx.env, &[]);
    let attestation = Bytes::from_array(&ctx.env, &[]);

    client.receive_message(&caller, &message, &attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_send_message_requires_caller_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let dest_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let msg_body = Bytes::from_array(&ctx.env, &[]);

    // No auth set for caller; require_auth should panic.
    client.send_message(&caller, &1u32, &recipient, &dest_caller, &0u32, &msg_body);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_receive_message_requires_caller_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let message = Bytes::from_array(&ctx.env, &[]);
    let attestation = Bytes::from_array(&ctx.env, &[]);

    // No auth set for caller; require_auth should panic.
    client.receive_message(&caller, &message, &attestation);
}

//=============================================================
// Configuration Tests
//=============================================================

#[test]
fn test_get_local_domain() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_local_domain(), TEST_LOCAL_DOMAIN);
}

#[test]
fn test_get_version() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_version(), TEST_VERSION);
}

#[test]
fn test_get_max_message_body_size() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(
        client.get_max_message_body_size(),
        TEST_MAX_MESSAGE_BODY_SIZE
    );
}

#[test]
fn test_set_max_message_body_size() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(
        client.get_max_message_body_size(),
        TEST_MAX_MESSAGE_BODY_SIZE
    );

    let new_size = TEST_MAX_MESSAGE_BODY_SIZE * 2;
    mock_set_max_message_body_size_auth(&ctx.env, &ctx.contract_id, &ctx.owner, new_size);
    client.set_max_message_body_size(&new_size);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_max_message_body_size_updated(&new_size);

    assert_eq!(client.get_max_message_body_size(), new_size);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_max_message_body_size_requires_owner_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Try to set without auth - should fail
    client.set_max_message_body_size(&16384);
}

#[test]
fn test_is_nonce_used_for_unused_nonce() {
    let ctx = setup_contract();
    let client = ctx.client();

    let nonce = BytesN::from_array(&ctx.env, &[1u8; 32]);
    assert!(
        !client.is_nonce_used(&nonce),
        "Fresh nonce should not be used"
    );
}

#[test]
fn test_zero_nonce_is_claimed_in_constructor() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Verify that the zero nonce is marked as used
    let zero_nonce = BytesN::from_array(&ctx.env, &[0u8; 32]);
    assert!(
        client.is_nonce_used(&zero_nonce),
        "Zero nonce should be marked as used after initialization"
    );
}

#[test]
fn test_is_nonce_used_returns_distinct_values() {
    let ctx = setup_contract();
    let client = ctx.client();

    let nonce1 = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let nonce2 = BytesN::from_array(&ctx.env, &[2u8; 32]);
    let zero_nonce = BytesN::from_array(&ctx.env, &[0u8; 32]);

    // Zero nonce is used, others are not
    assert!(client.is_nonce_used(&zero_nonce));
    assert!(!client.is_nonce_used(&nonce1));
    assert!(!client.is_nonce_used(&nonce2));
}

// =============================================================
// send_message Tests
// =============================================================
#[test]
fn test_send_message_formats_message_correctly_in_event() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[0xAA; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0xBB; 32]);
    let min_finality_threshold = 1000u32;
    let message_body = Bytes::from_slice(&ctx.env, &[0x01, 0x02, 0x03, 0x04]);

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &message_body,
    );

    // Extract and verify the message from the MessageSent event
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_message_sent();
    let message = events.expect_message_sent();

    // Verify the message format using Message accessors
    assert_eq!(
        MessageV2::get_version(&message).unwrap(),
        TEST_VERSION,
        "Version should match test version"
    );
    assert_eq!(
        MessageV2::get_source_domain(&message).unwrap(),
        TEST_LOCAL_DOMAIN,
        "Source domain should match local domain"
    );
    assert_eq!(
        MessageV2::get_destination_domain(&message).unwrap(),
        destination_domain,
        "Destination domain should match input"
    );
    assert_eq!(
        MessageV2::get_recipient(&message).unwrap(),
        recipient,
        "Recipient should match input"
    );
    assert_eq!(
        MessageV2::get_destination_caller(&message).unwrap(),
        destination_caller,
        "Destination caller should match input"
    );
    assert_eq!(
        MessageV2::get_min_finality_threshold(&message).unwrap(),
        min_finality_threshold,
        "Min finality threshold should match input"
    );
    assert_eq!(
        MessageV2::get_message_body(&message),
        message_body,
        "Message body should match input"
    );

    // Verify nonce and finality_threshold_executed are zero (empty)
    assert_eq!(
        MessageV2::get_nonce(&message).unwrap(),
        BytesN::from_array(&ctx.env, &[0u8; 32]),
        "Nonce should be empty (all zeros)"
    );
    assert_eq!(
        MessageV2::get_finality_threshold_executed(&message).unwrap(),
        0,
        "Finality threshold executed should be 0"
    );
}

#[test]
fn test_send_message_with_empty_message_body() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let min_finality_threshold = 0u32;
    let message_body = Bytes::new(&ctx.env); // Empty message body

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &message_body,
    );

    // Extract and verify message
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_message_sent();
    let message = events.expect_message_sent();

    assert_eq!(
        MessageV2::get_message_body(&message),
        Bytes::new(&ctx.env),
        "Message body should be empty"
    );
}

#[test]
fn test_send_message_with_zero_destination_caller() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[0xFF; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]); // Zero = any caller can relay
    let min_finality_threshold = 100u32;
    let message_body = Bytes::from_slice(&ctx.env, b"hello");

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &message_body,
    );

    // Extract and verify message
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_message_sent();
    let message = events.expect_message_sent();

    assert_eq!(
        MessageV2::get_destination_caller(&message).unwrap(),
        BytesN::from_array(&ctx.env, &[0u8; 32]),
        "Destination caller should be zero"
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6900)")]
fn test_send_message_fails_destination_is_local_domain() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    // Use the same domain as local domain to trigger the error
    let destination_domain = TEST_LOCAL_DOMAIN;
    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let min_finality_threshold = 500u32;
    let message_body = Bytes::new(&ctx.env);

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6902)")]
fn test_send_message_fails_recipient_is_zero() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let min_finality_threshold = 500u32;
    let message_body = Bytes::new(&ctx.env);

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6901)")]
fn test_send_message_fails_message_body_too_large() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let min_finality_threshold = 500u32;
    // Create a message body larger than max_message_body_size
    let large_body = Bytes::from_slice(
        &ctx.env,
        &[0xFFu8; (TEST_MAX_MESSAGE_BODY_SIZE + 1) as usize],
    );

    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        large_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &large_body,
    );
}

#[test]
fn test_send_message_at_max_message_body_size_boundary() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let destination_domain = 5u32;
    let recipient = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let min_finality_threshold = 500u32;
    // Create a message body exactly at max_message_body_size - should succeed
    let max_body = Bytes::from_slice(&ctx.env, &[0xAAu8; TEST_MAX_MESSAGE_BODY_SIZE as usize]);
    mock_send_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        destination_domain,
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        max_body.clone(),
    );

    client.send_message(
        &caller,
        &destination_domain,
        &recipient,
        &destination_caller,
        &min_finality_threshold,
        &max_body,
    );

    // Verify event was emitted
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_message_sent();
}

// =============================================================
// receive_message Tests
// =============================================================

#[test]
fn test_receive_valid_message() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    // This should always be the same because it's the second contract registered after MessageTransmitter.
    // The fixture_valid_message uses the expected contract ID as the recipient (0x0000000000000000000000000000000000000000000000000000000000000008)
    let _mock_handler_address = ctx.env.register(MockMessageHandler, ());

    let fixture = fixture_valid_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_message_received();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6000)")]
fn test_receive_message_fails_invalid_attestation_length_empty() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    // Any message with empty attestation will fail attestation verification first
    let message = Bytes::new(&ctx.env);
    let attestation = Bytes::new(&ctx.env); // Expected length: 65 * signature_threshold (1) = 65

    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        message.clone(),
        attestation.clone(),
    );
    client.receive_message(&caller, &message, &attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6000)")]
fn test_receive_message_fails_invalid_attestation_length_too_short() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let message = create_test_message(
        &ctx.env,
        TEST_VERSION,
        5u32,
        TEST_LOCAL_DOMAIN,
        BytesN::from_array(&ctx.env, &[1u8; 32]),
        BytesN::from_array(&ctx.env, &[0xAA; 32]),
        BytesN::from_array(&ctx.env, &[0xBB; 32]),
        BytesN::from_array(&ctx.env, &[0u8; 32]),
        1000u32,
        2000u32,
        Bytes::new(&ctx.env),
    );

    // Attestation too short - needs 65 bytes for 1 signature
    let attestation = Bytes::from_slice(&ctx.env, &[0u8; 64]);

    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        message.clone(),
        attestation.clone(),
    );
    client.receive_message(&caller, &message, &attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6000)")]
fn test_receive_message_fails_invalid_attestation_length_too_long() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let message = create_test_message(
        &ctx.env,
        TEST_VERSION,
        5u32,
        TEST_LOCAL_DOMAIN,
        BytesN::from_array(&ctx.env, &[1u8; 32]),
        BytesN::from_array(&ctx.env, &[0xAA; 32]),
        BytesN::from_array(&ctx.env, &[0xBB; 32]),
        BytesN::from_array(&ctx.env, &[0u8; 32]),
        1000u32,
        2000u32,
        Bytes::new(&ctx.env),
    );

    // Attestation too long - threshold is 1, so only 65 bytes expected
    let attestation = Bytes::from_slice(&ctx.env, &[0u8; 66]);

    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        message.clone(),
        attestation.clone(),
    );
    client.receive_message(&caller, &message, &attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6904)")]
fn test_receive_message_fails_invalid_message_format_too_short() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_message_too_short(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6905)")]
fn test_receive_message_fails_invalid_message_invalid_destination_domain() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_invalid_destination_domain_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6906)")]
fn test_receive_message_fails_invalid_message_invalid_destination_caller() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_invalid_destination_caller_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6906)")]
fn test_receive_message_fails_invalid_message_invalid_caller() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_invalid_destination_caller_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6002)")]
fn test_receive_message_fails_invalid_attestation_signatures() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_not_enabled_attester_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6907)")] // InvalidMessageVersion
fn test_receive_message_fails_with_invalid_message_version() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    let fixture = fixture_invalid_version_message(&ctx.env);
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );

    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6908)")] // NonceAlreadyUsed
fn test_receive_message_fails_with_nonce_already_used() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    // Register the mock handler that the fixture expects as recipient
    let _mock_handler_address = ctx.env.register(MockMessageHandler, ());

    let fixture = fixture_valid_message(&ctx.env);

    // First receive should succeed
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );
    client.receive_message(&caller, &fixture.message, &fixture.attestation);

    // Second receive with same nonce should fail
    mock_receive_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &caller,
        fixture.message.clone(),
        fixture.attestation.clone(),
    );
    client.receive_message(&caller, &fixture.message, &fixture.attestation);
}

// =============================================================
// Attestable Tests
// =============================================================

#[test]
fn test_get_attester_manager() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(
        client.get_attester_manager(),
        Some(ctx.attester_manager.clone())
    );
}

#[test]
fn test_update_attester_manager() {
    let ctx = setup_contract();
    let client = ctx.client();
    let new_attester_manager = Address::generate(&ctx.env);

    mock_update_attester_manager_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.owner,
        &new_attester_manager,
    );

    client.update_attester_manager(&new_attester_manager);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_manager_updated(
        &Some(ctx.attester_manager.clone()),
        &new_attester_manager,
    );

    assert_eq!(
        client.get_attester_manager(),
        Some(new_attester_manager.clone())
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_attester_manager_requires_owner_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let new_attester_manager = Address::generate(&ctx.env);

    // No auth set - should fail
    client.update_attester_manager(&new_attester_manager);
}

#[test]
fn test_enable_attester() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create a new attester address that's not already enabled
    let new_attester = BytesN::from_array(&ctx.env, &[0xAA; 20]);

    assert!(!client.is_enabled_attester(&new_attester));
    assert_eq!(client.get_num_enabled_attesters(), 2);

    mock_enable_attester_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &new_attester,
    );

    client.enable_attester(&new_attester);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&new_attester);

    assert!(client.is_enabled_attester(&new_attester));
    assert_eq!(client.get_num_enabled_attesters(), 3);
    assert_eq!(client.get_enabled_attester(&2), new_attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_enable_attester_requires_attester_manager_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let new_attester = BytesN::from_array(&ctx.env, &[0xAA; 20]);

    // No auth set - should fail
    client.enable_attester(&new_attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6005)")]
fn test_enable_attester_fails_if_already_enabled() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Try to enable an attester that's already enabled
    mock_enable_attester_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &ctx.attester1,
    );

    client.enable_attester(&ctx.attester1);
}

#[test]
fn test_disable_attester() {
    let ctx = setup_contract();
    let client = ctx.client();

    // First lower the signature threshold so we can disable an attester
    mock_set_signature_threshold_auth(&ctx.env, &ctx.contract_id, &ctx.attester_manager, 1);
    client.set_signature_threshold(&1);

    assert!(client.is_enabled_attester(&ctx.attester1));
    assert_eq!(client.get_num_enabled_attesters(), 2);

    mock_disable_attester_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &ctx.attester1,
    );

    client.disable_attester(&ctx.attester1);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_disabled(&ctx.attester1);

    assert!(!client.is_enabled_attester(&ctx.attester1));
    assert_eq!(client.get_num_enabled_attesters(), 1);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_disable_attester_requires_attester_manager_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    // No auth set - should fail
    client.disable_attester(&ctx.attester1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6009)")]
fn test_disable_attester_fails_if_too_few_attesters() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Signature threshold is 2, and there are 2 attesters.
    // Disabling one would leave fewer attesters than threshold.
    mock_disable_attester_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &ctx.attester1,
    );

    client.disable_attester(&ctx.attester1);
}

#[test]
fn test_get_enabled_attester() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_enabled_attester(&0), ctx.attester1);
    assert_eq!(client.get_enabled_attester(&1), ctx.attester2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6007)")]
fn test_get_enabled_attester_fails_index_out_of_bounds() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Only 2 attesters are enabled, so index 2 should fail
    client.get_enabled_attester(&2);
}

#[test]
fn test_get_num_enabled_attesters() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_num_enabled_attesters(), 2);
}

#[test]
fn test_is_enabled_attester() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Enabled attesters
    assert!(client.is_enabled_attester(&ctx.attester1));
    assert!(client.is_enabled_attester(&ctx.attester2));

    // Non-enabled attester
    let unknown_attester = BytesN::from_array(&ctx.env, &[0xBB; 20]);
    assert!(!client.is_enabled_attester(&unknown_attester));
}

#[test]
fn test_get_signature_threshold() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_signature_threshold(), Some(2));
}

#[test]
fn test_set_signature_threshold() {
    let ctx = setup_contract();
    let client = ctx.client();

    // First add another attester so we can increase the threshold
    let new_attester = BytesN::from_array(&ctx.env, &[0xCC; 20]);
    mock_enable_attester_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &new_attester,
    );
    client.enable_attester(&new_attester);

    assert_eq!(client.get_signature_threshold(), Some(2));

    mock_set_signature_threshold_auth(&ctx.env, &ctx.contract_id, &ctx.attester_manager, 3);
    client.set_signature_threshold(&3);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(2), 3);

    assert_eq!(client.get_signature_threshold(), Some(3));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_signature_threshold_requires_attester_manager_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    // No auth set - should fail
    client.set_signature_threshold(&1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6010)")]
fn test_set_signature_threshold_fails_if_exceeds_num_attesters() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Only 2 attesters are enabled, so threshold of 3 should fail
    mock_set_signature_threshold_auth(&ctx.env, &ctx.contract_id, &ctx.attester_manager, 3);
    client.set_signature_threshold(&3);
}

// =============================================================
// Storage Error Tests
// =============================================================

/// A minimal contract to provide a contract context for storage tests.
#[contract]
pub struct StorageTestContract;

#[contractimpl]
impl StorageTestContract {}

#[test]
#[should_panic(expected = "Error(Contract, #6910)")] // LocalDomainNotSet
fn test_get_local_domain_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_local_domain(&env);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #6911)")] // VersionNotSet
fn test_get_version_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_version(&env);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #6912)")] // MaxMessageBodySizeNotSet
fn test_get_max_message_body_size_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_max_message_body_size(&env);
    });
}

// =============================================================
// Constructor Validation Tests
// =============================================================

#[test]
#[should_panic(expected = "Error(Contract, #6008)")] // InvalidAttesterPublicKey
fn test_constructor_fails_with_zero_attester_address() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    // Zero address attester should fail
    let zero_attester = BytesN::from_array(&env, &[0u8; 20]);

    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env, zero_attester],
            signature_threshold: 1,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6005)")] // AttesterAlreadyEnabled
fn test_constructor_fails_with_duplicate_attesters() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    let attester1 = BytesN::from_array(&env, &TEST_ATTESTER_ADDRESS_1);

    // Duplicate attester should fail
    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env, attester1.clone(), attester1],
            signature_threshold: 1,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6004)")] // InvalidSignatureThreshold
fn test_constructor_fails_with_zero_signature_threshold() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    let attester1 = BytesN::from_array(&env, &TEST_ATTESTER_ADDRESS_1);

    // Zero threshold should fail
    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env, attester1],
            signature_threshold: 0,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6010)")] // SignatureThresholdTooHigh
fn test_constructor_fails_with_threshold_exceeding_attesters() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    let attester1 = BytesN::from_array(&env, &TEST_ATTESTER_ADDRESS_1);

    // Threshold of 2 with only 1 attester should fail
    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env, attester1],
            signature_threshold: 2,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6913)")] // NoAttesters
fn test_constructor_fails_with_no_attesters() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    // Empty attesters should fail with NoAttesters before reaching signature threshold check
    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env],
            signature_threshold: 1,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6913)")] // NoAttesters
fn test_constructor_fails_with_no_attesters_and_zero_threshold() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let attester_manager = Address::generate(&env);
    let admin = Address::generate(&env);

    // Empty attesters with zero threshold should still fail with NoAttesters
    env.register(
        MessageTransmitterV2Contract,
        (MessageTransmitterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            attester_manager,
            attesters: vec![&env],
            signature_threshold: 0,
            admin,
            local_domain: TEST_LOCAL_DOMAIN,
            version: TEST_VERSION,
            max_message_body_size: TEST_MAX_MESSAGE_BODY_SIZE,
        },),
    );
}
