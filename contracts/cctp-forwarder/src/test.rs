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
use crate::contract::{
    CctpForwarderContract, CctpForwarderContractClient, CctpForwarderContractInitParams,
};
use cctp_roles::test_utils::CctpEventAssertions;
use cctp_utils::{BurnMessageV2, MessageV2};
use common_roles::test_utils::pausable::{mock_pause_auth, mock_unpause_auth};
use common_roles::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use soroban_sdk::{
    address_payload::AddressPayload, contract, contractimpl, testutils::Address as _, token,
    Address, Bytes, BytesN, Env, IntoVal, MuxedAddress, U256,
};
use stellar_utils::address_to_bytes32;

use soroban_sdk::xdr::{
    AccountEntry, AccountEntryExt, AccountId, LedgerEntry, LedgerEntryData, LedgerEntryExt,
    LedgerKey, LedgerKeyAccount, LedgerKeyTrustLine, PublicKey, SequenceNumber, Thresholds,
    TrustLineAsset, TrustLineEntry, TrustLineEntryExt, TrustLineFlags, Uint256, VecM,
};
extern crate alloc;
use alloc::rc::Rc;

// Reuse existing test WASM from the roles package
const UPGRADE_V2_WASM: &[u8] = include_bytes!("../../../testdata/upgrade_v2.wasm");

// Test configuration constants
const TEST_MESSAGE_VERSION: u32 = 1;
const TEST_BURN_MESSAGE_VERSION: u32 = 1;
const TEST_SOURCE_DOMAIN: u32 = 6;
const TEST_LOCAL_DOMAIN: u32 = 2;
const TEST_MINT_AMOUNT: i128 = 12345;

/// A mock MessageTransmitter contract for testing.
#[contract]
pub struct MockMessageTransmitter;

#[contractimpl]
impl MockMessageTransmitter {
    pub fn set_local_token(e: &Env, token: Address) {
        e.storage().instance().set(&"local_token", &token);
    }

    pub fn receive_message(e: &Env, caller: Address, _message: Bytes, _attestation: Bytes) -> bool {
        let token: Address = e
            .storage()
            .instance()
            .get(&"local_token")
            .expect("local_token not set");
        let token_admin_client = token::StellarAssetClient::new(e, &token);
        token_admin_client
            .mock_all_auths()
            .mint(&caller, &TEST_MINT_AMOUNT);
        true
    }
}

/// A mock TokenMessengerMinter contract for testing.
#[contract]
pub struct MockTokenMessengerMinter;

#[contractimpl]
impl MockTokenMessengerMinter {
    pub fn set_local_token(e: &Env, token: Address) {
        e.storage().instance().set(&"local_token", &token);
    }

    pub fn get_local_token(
        e: &Env,
        _remote_domain: u32,
        _remote_token: BytesN<32>,
    ) -> Option<Address> {
        e.storage().instance().get(&"local_token")
    }
}

/// A mock MessageTransmitter that does NOT mint any tokens.
#[contract]
pub struct MockNoMintTransmitter;

#[contractimpl]
impl MockNoMintTransmitter {
    pub fn receive_message(
        _e: &Env,
        _caller: Address,
        _message: Bytes,
        _attestation: Bytes,
    ) -> bool {
        true
    }
}

/// A mock TokenMessengerMinter that always returns None for get_local_token.
#[contract]
pub struct MockUnresolvedTokenMessengerMinter;

#[contractimpl]
impl MockUnresolvedTokenMessengerMinter {
    pub fn get_local_token(
        _e: &Env,
        _remote_domain: u32,
        _remote_token: BytesN<32>,
    ) -> Option<Address> {
        None
    }
}

/// A mock contract used as the forward recipient.
#[contract]
pub struct MockForwardRecipient;

#[contractimpl]
impl MockForwardRecipient {
    pub fn ping(_e: &Env) -> bool {
        true
    }
}

/// A mock token whose `balance()` returns adversarial values that cause
/// `checked_sub` to overflow i128 (i128::MAX - (-1) > i128::MAX).
#[contract]
pub struct MockOverflowBalanceToken;

#[contractimpl]
impl MockOverflowBalanceToken {
    pub fn balance(e: &Env, _id: Address) -> i128 {
        let count: u32 = e.storage().instance().get(&"call_count").unwrap_or(0);
        e.storage().instance().set(&"call_count", &(count + 1));
        if count == 0 {
            -1i128
        } else {
            i128::MAX
        }
    }
}

struct TestContext {
    env: Env,
    contract_id: Address,
    owner: Address,
    pauser: Address,
    rescuer: Address,
    admin: Address,
    message_transmitter: Address,
    token_messenger_minter: Address,
    local_token: Address,
    local_token_asset: soroban_sdk::xdr::Asset,
}

impl TestContext {
    fn client(&self) -> CctpForwarderContractClient<'_> {
        CctpForwarderContractClient::new(&self.env, &self.contract_id)
    }
}

fn setup_contract() -> TestContext {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let admin = Address::generate(&env);

    // Register mock contracts
    let message_transmitter = env.register(MockMessageTransmitter, ());
    let token_messenger_minter = env.register(MockTokenMessengerMinter, ());

    // Create a mock token for testing
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token.address();

    MockMessageTransmitterClient::new(&env, &message_transmitter).set_local_token(&token_address);

    // Set the mock local token via contract storage
    MockTokenMessengerMinterClient::new(&env, &token_messenger_minter)
        .set_local_token(&token_address);

    let contract_id = env.register(
        CctpForwarderContract,
        (CctpForwarderContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer: rescuer.clone(),
            admin: admin.clone(),
            message_transmitter: message_transmitter.clone(),
            token_messenger_minter: token_messenger_minter.clone(),
            expected_message_version: TEST_MESSAGE_VERSION,
            expected_burn_message_version: TEST_BURN_MESSAGE_VERSION,
        },),
    );

    let local_token_asset = token.asset();

    TestContext {
        env,
        contract_id,
        owner,
        pauser,
        rescuer,
        admin,
        message_transmitter,
        token_messenger_minter,
        local_token: token_address,
        local_token_asset,
    }
}

#[test]
fn test_constructor_initializes_state_and_emits_events() {
    let ctx = setup_contract();
    let client = ctx.client();

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());

    events.assert_event_count(4);
    events.assert_ownership_transfer_completed(&ctx.owner);
    events.assert_pauser_changed(&ctx.pauser);
    events.assert_rescuer_changed(&ctx.rescuer);
    events.assert_admin_changed(None, &ctx.admin);

    assert_eq!(client.get_owner(), Some(ctx.owner.clone()));
    assert_eq!(client.get_pauser(), Some(ctx.pauser.clone()));
    assert_eq!(client.get_rescuer(), Some(ctx.rescuer.clone()));
    assert_eq!(client.get_admin(), Some(ctx.admin.clone()));
    assert_eq!(
        client.get_message_transmitter(),
        ctx.message_transmitter.clone()
    );
    assert_eq!(
        client.get_token_messenger_minter(),
        ctx.token_messenger_minter.clone()
    );
    assert_eq!(client.get_expected_message_version(), TEST_MESSAGE_VERSION);
    assert_eq!(
        client.get_expected_burn_msg_version(),
        TEST_BURN_MESSAGE_VERSION
    );
    assert!(!client.paused());
}

#[test]
fn test_mint_and_forward_emits_event_and_forwards_tokens() {
    let ctx = setup_contract();
    let client = ctx.client();

    let forward_recipient_contract = ctx.env.register(MockForwardRecipient, ());
    let forward_recipient = MuxedAddress::from(&forward_recipient_contract);
    let forward_recipient_strkey_bytes = forward_recipient_contract.to_string().to_bytes();

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let hook_data = create_valid_hook_data(
        &ctx.env,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(&forward_recipient, &ctx.local_token, TEST_MINT_AMOUNT);

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(
        token_client.balance(&forward_recipient_contract),
        TEST_MINT_AMOUNT
    );
}

#[test]
fn test_cctp_forwarder_is_ownable() {
    let ctx = setup_contract();
    let client = ctx.client();

    common_roles::assert_contract_is_ownable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_cctp_forwarder_is_pausable() {
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
fn test_cctp_forwarder_is_rescuable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_rescuable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_contract_is_manageable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_manageable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.owner,
        &ctx.admin,
        UPGRADE_V2_WASM
    );
}

// ################## MESSAGE VALIDATION TESTS ##################

fn create_test_message_and_burn_message(
    env: &Env,
    recipient: &BytesN<32>,
    mint_recipient: &BytesN<32>,
    destination_caller: &BytesN<32>,
    hook_data: Bytes,
) -> Bytes {
    // Create burn message body
    let burn_token = BytesN::from_array(env, &[1u8; 32]);
    let amount = U256::from_u128(env, 1000000);
    let message_sender = BytesN::from_array(env, &[2u8; 32]);
    let max_fee = U256::from_u128(env, 1000);

    let burn_message = BurnMessageV2::format_for_relay(
        env,
        TEST_BURN_MESSAGE_VERSION,
        burn_token,
        mint_recipient.clone(),
        amount,
        message_sender,
        max_fee,
        hook_data,
    );

    // Create outer message
    let sender = BytesN::from_array(env, &[4u8; 32]);

    MessageV2::format_for_relay(
        env,
        TEST_MESSAGE_VERSION,
        TEST_SOURCE_DOMAIN,
        TEST_LOCAL_DOMAIN,
        sender,
        recipient.clone(),
        destination_caller.clone(),
        1000,
        burn_message,
    )
}

fn create_valid_hook_data(env: &Env, forward_recipient_strkey: &[u8]) -> Bytes {
    let mut data = Bytes::new(env);
    data.extend_from_slice(&[0u8; 24]); // magic bytes
    data.extend_from_array(&0u32.to_be_bytes()); // version
    data.extend_from_array(&(forward_recipient_strkey.len() as u32).to_be_bytes());
    data.extend_from_slice(forward_recipient_strkey);
    data
}

#[test]
#[should_panic(expected = "Error(Contract, #7303)")] // InvalidMessageFormat
fn test_mint_and_forward_fails_with_invalid_message_format() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create an invalid message (too short)
    let message = Bytes::from_array(&ctx.env, &[0u8; 10]);
    let attestation = Bytes::new(&ctx.env);

    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7304)")] // UnsupportedMessageVersion
fn test_mint_and_forward_fails_with_unsupported_message_version() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create a message with wrong version
    let recipient = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let mint_recipient = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let hook_data = create_valid_hook_data(
        &ctx.env,
        b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5",
    );

    // Create burn message
    let burn_token = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let burn_message = BurnMessageV2::format_for_relay(
        &ctx.env,
        TEST_BURN_MESSAGE_VERSION,
        burn_token,
        mint_recipient,
        U256::from_u128(&ctx.env, 1000000),
        BytesN::from_array(&ctx.env, &[2u8; 32]),
        U256::from_u128(&ctx.env, 1000),
        hook_data,
    );

    // Create message with wrong version (99 instead of 1)
    let sender = BytesN::from_array(&ctx.env, &[4u8; 32]);
    let message = MessageV2::format_for_relay(
        &ctx.env,
        99, // Wrong version
        TEST_SOURCE_DOMAIN,
        TEST_LOCAL_DOMAIN,
        sender,
        recipient,
        destination_caller,
        1000,
        burn_message,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7306)")] // UnsupportedBurnMessageVersion
fn test_mint_and_forward_fails_with_unsupported_burn_message_version() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data
    let hook_data = create_valid_hook_data(
        &ctx.env,
        b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5",
    );

    // Create burn message with wrong version
    let burn_token = BytesN::from_array(&ctx.env, &[1u8; 32]);
    let burn_message = BurnMessageV2::format_for_relay(
        &ctx.env,
        99, // Wrong burn message version
        burn_token,
        BytesN::from_array(&ctx.env, &[0u8; 32]),
        U256::from_u128(&ctx.env, 1000000),
        BytesN::from_array(&ctx.env, &[2u8; 32]),
        U256::from_u128(&ctx.env, 1000),
        hook_data,
    );

    // Create valid outer message
    let sender = BytesN::from_array(&ctx.env, &[4u8; 32]);
    let recipient = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let message = MessageV2::format_for_relay(
        &ctx.env,
        TEST_MESSAGE_VERSION,
        TEST_SOURCE_DOMAIN,
        TEST_LOCAL_DOMAIN,
        sender,
        recipient,
        destination_caller,
        1000,
        burn_message,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7301)")] // InvalidMintRecipient
fn test_mint_and_forward_fails_when_mint_recipient_is_not_forwarder() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data
    let hook_data = create_valid_hook_data(
        &ctx.env,
        b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5",
    );

    // Get the token messenger minter as bytes32 for the recipient
    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    // Use a different address as mint_recipient (not the forwarder)
    let wrong_mint_recipient = BytesN::from_array(&ctx.env, &[0xdeu8; 32]);

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &wrong_mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7302)")] // InvalidRecipient
fn test_mint_and_forward_fails_when_recipient_is_not_token_messenger_minter() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data
    let hook_data = create_valid_hook_data(
        &ctx.env,
        b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5",
    );

    // Get the forwarder contract as bytes32 for mint_recipient
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    // Use a wrong recipient (not the token messenger minter)
    let wrong_recipient = BytesN::from_array(&ctx.env, &[0xdeu8; 32]);

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &wrong_recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7300)")] // HookDataTooShort
fn test_mint_and_forward_fails_with_hook_data_too_short() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data that's too short (less than MIN_HEADER_LENGTH = 32 bytes)
    let mut hook_data = Bytes::new(&ctx.env);
    for _ in 0..20 {
        hook_data.push_back(0);
    }

    // Get proper addresses
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7313)")] // InvalidHookVersion
fn test_mint_and_forward_fails_with_invalid_hook_version() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data with invalid version (expected is 0)
    let mut hook_data = Bytes::new(&ctx.env);

    // Zero magic bytes (24 bytes)
    for _ in 0..24 {
        hook_data.push_back(0);
    }

    // Version: 99 (invalid, expected 0)
    hook_data.extend_from_array(&99u32.to_be_bytes());

    // Length: 56 (standard strkey length)
    hook_data.extend_from_array(&56u32.to_be_bytes());

    let strkey = b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5";
    hook_data.extend_from_slice(strkey);

    // Get proper addresses
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7300)")] // HookDataTooShort
fn test_mint_and_forward_fails_when_forward_recipient_length_exceeds_actual_data() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data where length field claims more data than is present
    let mut hook_data = Bytes::new(&ctx.env);

    // Zero magic bytes (24 bytes)
    for _ in 0..24 {
        hook_data.push_back(0);
    }

    // Version: 0 (valid)
    hook_data.extend_from_array(&0u32.to_be_bytes());

    // Length: 100 (claims 100 bytes of forward_recipient data)
    hook_data.extend_from_array(&100u32.to_be_bytes());

    // Only provide 20 bytes of forward_recipient data (less than claimed 100)
    for _ in 0..20 {
        hook_data.push_back(b'A');
    }

    // Get proper addresses
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #7300)")]
fn test_mint_and_forward_fails_with_hook_data_length_overflow() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data with forward_recipient_length set to u32::MAX to trigger overflow
    let mut hook_data = Bytes::new(&ctx.env);

    // Zero magic bytes (24 bytes)
    for _ in 0..24 {
        hook_data.push_back(0);
    }

    // Version: 0
    hook_data.extend_from_array(&0u32.to_be_bytes());

    // Length: u32::MAX (would overflow when added to FORWARD_RECIPIENT_OFFSET)
    hook_data.extend_from_array(&u32::MAX.to_be_bytes());

    // Get proper addresses
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        soroban_sdk::address_payload::AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        soroban_sdk::address_payload::AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

#[test]
#[should_panic(expected = "couldn't process the string as strkey")] // Host rejects invalid strkey
fn test_mint_and_forward_fails_with_invalid_strkey_prefix() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Create hook data with invalid strkey prefix (not G, C, or M)
    let invalid_strkey = b"XA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5";
    let mut hook_data = Bytes::new(&ctx.env);

    // Zero magic bytes
    for _ in 0..24 {
        hook_data.push_back(0);
    }

    // Version and length (actual strkey length)
    hook_data.extend_from_array(&0u32.to_be_bytes());
    hook_data.extend_from_array(&(invalid_strkey.len() as u32).to_be_bytes());

    // Invalid strkey (starts with 'X' instead of G, C, or M)
    hook_data.extend_from_slice(invalid_strkey);

    // Get proper addresses
    let forwarder_bytes = ctx.contract_id.to_payload().unwrap();
    let mint_recipient = match forwarder_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let tmm_bytes = ctx.token_messenger_minter.to_payload().unwrap();
    let recipient = match tmm_bytes {
        AddressPayload::ContractIdHash(hash) => hash,
        _ => panic!("Expected contract address"),
    };

    let message = create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    );

    let attestation = Bytes::new(&ctx.env);
    client.mint_and_forward(&message, &attestation);
}

// ################## PAUSED STATE TESTS ##################

#[test]
#[should_panic(expected = "Error(Contract, #1000)")] // EnforcedPaused
fn test_mint_and_forward_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Pause the contract
    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    let message = Bytes::new(&ctx.env);
    let attestation = Bytes::new(&ctx.env);

    client.mint_and_forward(&message, &attestation);
}

// ################## STORAGE ERROR TESTS ##################

/// A minimal contract to provide a contract context for storage tests.
#[contract]
pub struct StorageTestContract;

#[contractimpl]
impl StorageTestContract {}

#[test]
#[should_panic(expected = "Error(Contract, #7308)")] // MessageTransmitterNotSet
fn test_get_message_transmitter_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_message_transmitter(&env);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #7309)")] // TokenMessengerMinterNotSet
fn test_get_token_messenger_minter_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_token_messenger_minter(&env);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #7310)")] // ExpectedMessageVersionNotSet
fn test_get_expected_msg_version_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_expected_msg_version(&env);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #7311)")] // ExpectedBurnMessageVersionNotSet
fn test_get_expected_burn_msg_version_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestContract, ());

    env.as_contract(&contract_id, || {
        crate::storage::get_expected_burn_msg_version(&env);
    });
}

// ################## HELPERS ##################

/// Builds a valid CCTP message targeting the given context's forwarder and TMM,
/// with the specified forward recipient strkey in the hook data.
fn build_valid_message(ctx: &TestContext, forward_recipient_strkey: &[u8]) -> Bytes {
    let recipient = address_to_bytes32(&ctx.token_messenger_minter).unwrap();
    let mint_recipient = address_to_bytes32(&ctx.contract_id).unwrap();
    let hook_data = create_valid_hook_data(&ctx.env, forward_recipient_strkey);

    create_test_message_and_burn_message(
        &ctx.env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        hook_data,
    )
}

/// Creates a Stellar account entry and a trustline for the given asset in the test ledger.
fn create_test_account_with_trustline(
    env: &Env,
    ed25519: &[u8; 32],
    asset: &soroban_sdk::xdr::Asset,
) {
    let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(*ed25519)));

    // Create account entry
    let acct_key = Rc::new(LedgerKey::Account(LedgerKeyAccount {
        account_id: account_id.clone(),
    }));
    let acct_entry = Rc::new(LedgerEntry {
        data: LedgerEntryData::Account(AccountEntry {
            account_id: account_id.clone(),
            balance: 0,
            flags: 0,
            home_domain: Default::default(),
            inflation_dest: None,
            num_sub_entries: 0,
            seq_num: SequenceNumber(0),
            thresholds: Thresholds([1; 4]),
            signers: VecM::default(),
            ext: AccountEntryExt::V0,
        }),
        last_modified_ledger_seq: 0,
        ext: LedgerEntryExt::V0,
    });
    env.host()
        .add_ledger_entry(&acct_key, &acct_entry, None)
        .unwrap();

    // Create trustline entry for the asset
    let tl_asset = match asset {
        soroban_sdk::xdr::Asset::CreditAlphanum4(a) => TrustLineAsset::CreditAlphanum4(a.clone()),
        soroban_sdk::xdr::Asset::CreditAlphanum12(a) => TrustLineAsset::CreditAlphanum12(a.clone()),
        soroban_sdk::xdr::Asset::Native => TrustLineAsset::Native,
    };
    let tl_key = Rc::new(LedgerKey::Trustline(LedgerKeyTrustLine {
        account_id: account_id.clone(),
        asset: tl_asset.clone(),
    }));
    let tl_entry = Rc::new(LedgerEntry {
        data: LedgerEntryData::Trustline(TrustLineEntry {
            account_id,
            asset: tl_asset,
            balance: 0,
            limit: i64::MAX,
            flags: TrustLineFlags::AuthorizedFlag as u32,
            ext: TrustLineEntryExt::V0,
        }),
        last_modified_ledger_seq: 0,
        ext: LedgerEntryExt::V0,
    });
    env.host()
        .add_ledger_entry(&tl_key, &tl_entry, None)
        .unwrap();
}

// ################## ADDITIONAL COVERAGE TESTS ##################

#[test]
fn test_mint_and_forward_with_c_account_recipient() {
    let ctx = setup_contract();
    let client = ctx.client();

    let c_strkey = "CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE";
    let c_account = Address::from_string(&soroban_sdk::String::from_str(&ctx.env, c_strkey));
    assert_eq!(
        c_account.to_string(),
        soroban_sdk::String::from_str(&ctx.env, c_strkey)
    );

    let message = build_valid_message(&ctx, c_strkey.as_bytes());
    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&c_account),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(token_client.balance(&c_account), TEST_MINT_AMOUNT);
}

#[test]
fn test_mint_and_forward_with_m_address_recipient() {
    let ctx = setup_contract();
    let client = ctx.client();

    // M-address strkey with 64-bit memo ID 123456, base account GA3D5KRYM6CB7...
    let m_strkey = "MA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAAAAAAAAAPCICBKU";
    let g_strkey = "GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5";

    let g_account = Address::from_string(&soroban_sdk::String::from_str(&ctx.env, g_strkey));
    let payload = g_account.to_payload().expect("account payload");
    let ed25519 = match payload {
        AddressPayload::AccountIdPublicKeyEd25519(bytes) => {
            let mut buf = [0u8; 32];
            bytes.copy_into_slice(&mut buf);
            buf
        }
        _ => panic!("Expected G-account"),
    };

    // Create the Stellar account and trustline entries so the SAC can transfer to it.
    create_test_account_with_trustline(&ctx.env, &ed25519, &ctx.local_token_asset);

    let expected_muxed =
        MuxedAddress::from_string(&soroban_sdk::String::from_str(&ctx.env, m_strkey));
    assert_eq!(expected_muxed.id(), Some(123456));

    let message = build_valid_message(&ctx, m_strkey.as_bytes());
    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(&expected_muxed, &ctx.local_token, TEST_MINT_AMOUNT);

    // Verify the SAC transfer event includes to_muxed_id with the memo ID
    let sac_events = EventAssertion::new(&ctx.env, ctx.local_token.clone());
    let (_, topics, data) = sac_events
        .find_event_by_symbol("transfer")
        .expect("SAC transfer event not found");
    let event_from: Address = topics.get_unchecked(1).into_val(&ctx.env);
    let event_to: Address = topics.get_unchecked(2).into_val(&ctx.env);
    assert_eq!(event_from, ctx.contract_id);
    assert_eq!(event_to, g_account);
    let data_map: soroban_sdk::Map<soroban_sdk::Symbol, soroban_sdk::Val> = data.into_val(&ctx.env);
    let event_amount: i128 = data_map
        .get(soroban_sdk::symbol_short!("amount"))
        .unwrap()
        .into_val(&ctx.env);
    assert_eq!(event_amount, TEST_MINT_AMOUNT);
    let to_muxed_id: u64 = data_map
        .get(soroban_sdk::Symbol::new(&ctx.env, "to_muxed_id"))
        .expect("SAC transfer event missing to_muxed_id")
        .into_val(&ctx.env);
    assert_eq!(to_muxed_id, 123456);

    // Balance accrues to the underlying G-address, not the muxed address
    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(token_client.balance(&g_account), TEST_MINT_AMOUNT);
}

#[test]
fn test_mint_and_forward_forwarder_balance_is_zero_after_forward() {
    let ctx = setup_contract();
    let client = ctx.client();

    let forward_recipient_contract = ctx.env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient_contract.to_string().to_bytes();

    let message = build_valid_message(
        &ctx,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);

    // Verify the forwarder starts with zero balance
    assert_eq!(token_client.balance(&ctx.contract_id), 0);

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&forward_recipient_contract),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    // Verify forwarder did not retain any tokens
    assert_eq!(token_client.balance(&ctx.contract_id), 0);

    // Verify recipient received tokens
    assert_eq!(
        token_client.balance(&forward_recipient_contract),
        TEST_MINT_AMOUNT
    );
}

#[test]
fn test_mint_and_forward_is_permissionless() {
    let ctx = setup_contract();
    let client = ctx.client();

    let forward_recipient = ctx.env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient.to_string().to_bytes();

    let message = build_valid_message(
        &ctx,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );

    // No auth mocking needed — mint_and_forward is permissionless.
    // If auth were required this would panic with an Auth error.
    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&forward_recipient),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(token_client.balance(&forward_recipient), TEST_MINT_AMOUNT);
}

#[test]
fn test_mint_and_forward_works_after_pause_unpause_cycle() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Pause the contract
    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();
    assert!(client.paused());

    // Unpause the contract
    mock_unpause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.unpause();
    assert!(!client.paused());

    // mint_and_forward should work normally after unpause
    let forward_recipient = ctx.env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient.to_string().to_bytes();

    let message = build_valid_message(
        &ctx,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&forward_recipient),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(token_client.balance(&forward_recipient), TEST_MINT_AMOUNT);
}

#[test]
#[should_panic(expected = "Error(Contract, #7307)")] // InvalidForwardRecipient
fn test_mint_and_forward_fails_when_forward_recipient_is_forwarder_itself() {
    let ctx = setup_contract();
    let client = ctx.client();

    let forwarder_strkey_bytes = ctx.contract_id.to_string().to_bytes();
    let message = build_valid_message(&ctx, forwarder_strkey_bytes.to_buffer::<64>().as_slice());

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));
}

#[test]
fn test_mint_and_forward_succeeds_when_forward_recipient_is_message_transmitter() {
    let ctx = setup_contract();
    let client = ctx.client();

    let mt_strkey_bytes = ctx.message_transmitter.to_string().to_bytes();
    let message = build_valid_message(&ctx, mt_strkey_bytes.to_buffer::<64>().as_slice());

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&ctx.message_transmitter),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(
        token_client.balance(&ctx.message_transmitter),
        TEST_MINT_AMOUNT
    );
}

#[test]
fn test_mint_and_forward_succeeds_when_forward_recipient_is_token_messenger_minter() {
    let ctx = setup_contract();
    let client = ctx.client();

    let tmm_strkey_bytes = ctx.token_messenger_minter.to_string().to_bytes();
    let message = build_valid_message(&ctx, tmm_strkey_bytes.to_buffer::<64>().as_slice());

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(3);
    events.assert_mint_and_forward(
        &MuxedAddress::from(&ctx.token_messenger_minter),
        &ctx.local_token,
        TEST_MINT_AMOUNT,
    );

    let token_client = token::TokenClient::new(&ctx.env, &ctx.local_token);
    assert_eq!(
        token_client.balance(&ctx.token_messenger_minter),
        TEST_MINT_AMOUNT
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7307)")] // InvalidForwardRecipient
fn test_mint_and_forward_fails_when_forward_recipient_is_local_token() {
    let ctx = setup_contract();
    let client = ctx.client();

    // Use the local token address as the forward recipient
    let local_token_strkey_bytes = ctx.local_token.to_string().to_bytes();
    let message = build_valid_message(&ctx, local_token_strkey_bytes.to_buffer::<64>().as_slice());

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));
}

#[test]
#[should_panic(expected = "Error(Contract, #7314)")] // NoTokensMinted
fn test_mint_and_forward_fails_when_no_tokens_minted() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let admin = Address::generate(&env);

    // Register a mock transmitter that does NOT mint tokens
    let no_mint_transmitter = env.register(MockNoMintTransmitter, ());
    let token_messenger_minter = env.register(MockTokenMessengerMinter, ());

    // Create a mock token
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token.address();

    MockTokenMessengerMinterClient::new(&env, &token_messenger_minter)
        .set_local_token(&token_address);

    let contract_id = env.register(
        CctpForwarderContract,
        (CctpForwarderContractInitParams {
            owner: owner.clone(),
            pauser,
            rescuer,
            admin,
            message_transmitter: no_mint_transmitter.clone(),
            token_messenger_minter: token_messenger_minter.clone(),
            expected_message_version: TEST_MESSAGE_VERSION,
            expected_burn_message_version: TEST_BURN_MESSAGE_VERSION,
        },),
    );

    let client = CctpForwarderContractClient::new(&env, &contract_id);

    let forward_recipient = env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient.to_string().to_bytes();

    let recipient = address_to_bytes32(&token_messenger_minter).unwrap();
    let mint_recipient = address_to_bytes32(&contract_id).unwrap();
    let hook_data = create_valid_hook_data(
        &env,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );
    let message = create_test_message_and_burn_message(
        &env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&env, &[0u8; 32]),
        hook_data,
    );

    client.mint_and_forward(&message, &Bytes::new(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #7314)")] // NoTokensMinted (checked_sub overflow)
fn test_mint_and_forward_fails_when_balance_overflows() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let admin = Address::generate(&env);

    let no_mint_transmitter = env.register(MockNoMintTransmitter, ());
    let token_messenger_minter = env.register(MockTokenMessengerMinter, ());

    // Register an adversarial mock token that returns values causing checked_sub overflow
    let overflow_token = env.register(MockOverflowBalanceToken, ());

    MockTokenMessengerMinterClient::new(&env, &token_messenger_minter)
        .set_local_token(&overflow_token);

    let contract_id = env.register(
        CctpForwarderContract,
        (CctpForwarderContractInitParams {
            owner: owner.clone(),
            pauser,
            rescuer,
            admin,
            message_transmitter: no_mint_transmitter.clone(),
            token_messenger_minter: token_messenger_minter.clone(),
            expected_message_version: TEST_MESSAGE_VERSION,
            expected_burn_message_version: TEST_BURN_MESSAGE_VERSION,
        },),
    );

    let client = CctpForwarderContractClient::new(&env, &contract_id);

    let forward_recipient = env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient.to_string().to_bytes();

    let recipient = address_to_bytes32(&token_messenger_minter).unwrap();
    let mint_recipient = address_to_bytes32(&contract_id).unwrap();
    let hook_data = create_valid_hook_data(
        &env,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );
    let message = create_test_message_and_burn_message(
        &env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&env, &[0u8; 32]),
        hook_data,
    );

    client.mint_and_forward(&message, &Bytes::new(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #7312)")] // LocalTokenNotResolved
fn test_mint_and_forward_fails_when_local_token_not_resolved() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let admin = Address::generate(&env);

    let message_transmitter = env.register(MockMessageTransmitter, ());
    // Use a TMM that always returns None for get_local_token
    let unresolved_tmm = env.register(MockUnresolvedTokenMessengerMinter, ());

    // Create a mock token (needed for the message transmitter mock)
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    MockMessageTransmitterClient::new(&env, &message_transmitter).set_local_token(&token.address());

    let contract_id = env.register(
        CctpForwarderContract,
        (CctpForwarderContractInitParams {
            owner,
            pauser,
            rescuer,
            admin,
            message_transmitter,
            token_messenger_minter: unresolved_tmm.clone(),
            expected_message_version: TEST_MESSAGE_VERSION,
            expected_burn_message_version: TEST_BURN_MESSAGE_VERSION,
        },),
    );

    let client = CctpForwarderContractClient::new(&env, &contract_id);

    let forward_recipient = env.register(MockForwardRecipient, ());
    let forward_recipient_strkey_bytes = forward_recipient.to_string().to_bytes();

    let recipient = address_to_bytes32(&unresolved_tmm).unwrap();
    let mint_recipient = address_to_bytes32(&contract_id).unwrap();
    let hook_data = create_valid_hook_data(
        &env,
        forward_recipient_strkey_bytes.to_buffer::<64>().as_slice(),
    );
    let message = create_test_message_and_burn_message(
        &env,
        &recipient,
        &mint_recipient,
        &BytesN::from_array(&env, &[0u8; 32]),
        hook_data,
    );

    client.mint_and_forward(&message, &Bytes::new(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #7305)")] // InvalidBurnMessageFormat
fn test_mint_and_forward_fails_with_invalid_burn_message_format() {
    let ctx = setup_contract();
    let client = ctx.client();

    let recipient = address_to_bytes32(&ctx.token_messenger_minter).unwrap();

    // Create a valid outer message but with a burn message body that's too short
    let invalid_burn_body = Bytes::from_array(&ctx.env, &[0u8; 10]);

    let sender = BytesN::from_array(&ctx.env, &[4u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);

    let message = MessageV2::format_for_relay(
        &ctx.env,
        TEST_MESSAGE_VERSION,
        TEST_SOURCE_DOMAIN,
        TEST_LOCAL_DOMAIN,
        sender,
        recipient,
        destination_caller,
        1000,
        invalid_burn_body,
    );

    client.mint_and_forward(&message, &Bytes::new(&ctx.env));
}
