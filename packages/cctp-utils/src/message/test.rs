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
// Allowing more than 7 arguments for our test fixtures.
#![allow(clippy::too_many_arguments)]
use super::*;
use crate::burn_message::BurnMessageV2;
use crate::test_utils::{create_test_bytes32, hex_to_bytes};
use rstest::rstest;
use soroban_sdk::Env;

// ================================
// Helper functions
// ================================

fn create_test_message(env: &Env) -> MessageV2 {
    MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: create_test_bytes32(env, 42),
        sender: create_test_bytes32(env, 1),
        recipient: create_test_bytes32(env, 2),
        destination_caller: create_test_bytes32(env, 3),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: Bytes::from_slice(env, &[0xDE, 0xAD, 0xBE, 0xEF]),
    }
}

// ================================
// Serialization tests
// ================================

#[test]
fn test_serialize_produces_correct_length() {
    let env = Env::default();

    // With message body: 148 bytes header + 4 bytes message body
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);
    assert_eq!(serialized.len(), 152);

    // Without message body: 148 bytes header only
    let empty_body_message = MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: create_test_bytes32(&env, 0),
        sender: create_test_bytes32(&env, 1),
        recipient: create_test_bytes32(&env, 2),
        destination_caller: create_test_bytes32(&env, 3),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: Bytes::new(&env),
    };
    let serialized_empty = empty_body_message.serialize(&env);
    assert_eq!(serialized_empty.len(), 148);
}

// ================================
// Getter function tests
// ================================

#[test]
fn test_get_version() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_version(&serialized).unwrap(), 1);
}

#[test]
fn test_get_source_domain() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_source_domain(&serialized).unwrap(), 100);
}

#[test]
fn test_get_destination_domain() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_destination_domain(&serialized).unwrap(), 200);
}

#[test]
fn test_get_nonce() {
    let env = Env::default();
    let expected_nonce = create_test_bytes32(&env, 42);
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_nonce(&serialized).unwrap(), expected_nonce);
}

#[test]
fn test_get_sender() {
    let env = Env::default();
    let expected_sender = create_test_bytes32(&env, 1);
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_sender(&serialized).unwrap(), expected_sender);
}

#[test]
fn test_get_recipient() {
    let env = Env::default();
    let expected_recipient = create_test_bytes32(&env, 2);
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        MessageV2::get_recipient(&serialized).unwrap(),
        expected_recipient
    );
}

#[test]
fn test_get_destination_caller() {
    let env = Env::default();
    let expected_caller = create_test_bytes32(&env, 3);
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        MessageV2::get_destination_caller(&serialized).unwrap(),
        expected_caller
    );
}

#[test]
fn test_get_min_finality_threshold() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        MessageV2::get_min_finality_threshold(&serialized).unwrap(),
        500
    );
}

#[test]
fn test_get_finality_threshold_executed() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        MessageV2::get_finality_threshold_executed(&serialized).unwrap(),
        1000
    );
}

#[test]
fn test_get_message_body() {
    let env = Env::default();
    let expected_body = Bytes::from_slice(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_message_body(&serialized), expected_body);
}

fn check_message_getter(data: &Bytes, field: usize) -> Result<(), MessageV2Error> {
    match field {
        0 => MessageV2::get_version(data).map(|_| ()),
        1 => MessageV2::get_source_domain(data).map(|_| ()),
        2 => MessageV2::get_destination_domain(data).map(|_| ()),
        3 => MessageV2::get_nonce(data).map(|_| ()),
        4 => MessageV2::get_sender(data).map(|_| ()),
        5 => MessageV2::get_recipient(data).map(|_| ()),
        6 => MessageV2::get_destination_caller(data).map(|_| ()),
        7 => MessageV2::get_min_finality_threshold(data).map(|_| ()),
        8 => MessageV2::get_finality_threshold_executed(data).map(|_| ()),
        _ => panic!("unknown field index"),
    }
}

#[rstest]
#[case::version(0, 3, 4)]
#[case::source_domain(1, 7, 8)]
#[case::destination_domain(2, 11, 12)]
#[case::nonce(3, 43, 44)]
#[case::sender(4, 75, 76)]
#[case::recipient(5, 107, 108)]
#[case::destination_caller(6, 139, 140)]
#[case::min_finality_threshold(7, 143, 144)]
#[case::finality_threshold_executed(8, 147, 148)]
fn test_getter_boundary_conditions(
    #[case] field: usize,
    #[case] too_short: usize,
    #[case] exact: usize,
) {
    let env = Env::default();
    let short_data = Bytes::from_slice(&env, &[0u8; 256][..too_short]);
    let exact_data = Bytes::from_slice(&env, &[0u8; 256][..exact]);

    assert_eq!(
        check_message_getter(&short_data, field),
        Err(MessageV2Error::FieldReadError)
    );
    assert!(check_message_getter(&exact_data, field).is_ok());
}

// ================================
// Format for relay tests
// ================================

#[test]
fn test_format_for_relay_roundtrip_all_fields() {
    let env = Env::default();
    let version = 42u32;
    let source_domain = 100u32;
    let destination_domain = 200u32;
    let sender = create_test_bytes32(&env, 1);
    let recipient = create_test_bytes32(&env, 2);
    let destination_caller = create_test_bytes32(&env, 3);
    let min_finality_threshold = 500u32;
    let message_body = Bytes::from_slice(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);

    let serialized = MessageV2::format_for_relay(
        &env,
        version,
        source_domain,
        destination_domain,
        sender.clone(),
        recipient.clone(),
        destination_caller.clone(),
        min_finality_threshold,
        message_body.clone(),
    );

    // Verify all provided fields
    assert_eq!(MessageV2::get_version(&serialized).unwrap(), version);
    assert_eq!(
        MessageV2::get_source_domain(&serialized).unwrap(),
        source_domain
    );
    assert_eq!(
        MessageV2::get_destination_domain(&serialized).unwrap(),
        destination_domain
    );
    assert_eq!(MessageV2::get_sender(&serialized).unwrap(), sender);
    assert_eq!(MessageV2::get_recipient(&serialized).unwrap(), recipient);
    assert_eq!(
        MessageV2::get_destination_caller(&serialized).unwrap(),
        destination_caller
    );
    assert_eq!(
        MessageV2::get_min_finality_threshold(&serialized).unwrap(),
        min_finality_threshold
    );
    assert_eq!(MessageV2::get_message_body(&serialized), message_body);

    // Verify auto-populated fields are set to empty/zero
    assert_eq!(
        MessageV2::get_nonce(&serialized).unwrap(),
        BytesN::from_array(&env, &[0u8; 32])
    );
    assert_eq!(
        MessageV2::get_finality_threshold_executed(&serialized).unwrap(),
        0
    );
}

// ================================
// Validation tests
// ================================

#[test]
fn test_validate_format_valid_message() {
    let env = Env::default();
    let message = create_test_message(&env);
    let serialized = message.serialize(&env);

    assert!(MessageV2::validate_format(&serialized).is_ok());
}

#[test]
fn test_validate_format_exact_minimum_length() {
    let env = Env::default();
    let exact_min_data = Bytes::from_slice(&env, &[0u8; 148]);

    assert!(MessageV2::validate_format(&exact_min_data).is_ok());
}

#[rstest]
#[case::empty(0)]
#[case::arbitrary_short(100)]
#[case::one_byte_short(147)]
fn test_validate_format_too_short(#[case] size: usize) {
    let env = Env::default();
    let data = Bytes::from_slice(&env, &[0u8; 256][..size]);
    assert_eq!(
        MessageV2::validate_format(&data),
        Err(MessageV2Error::MessageTooShort)
    );
}

// ================================
// Edge case tests
// ================================

#[test]
fn test_max_u32_values() {
    let env = Env::default();
    let message = MessageV2 {
        version: u32::MAX,
        source_domain: u32::MAX,
        destination_domain: u32::MAX,
        nonce: create_test_bytes32(&env, 0xFF),
        sender: create_test_bytes32(&env, 0xFF),
        recipient: create_test_bytes32(&env, 0xFF),
        destination_caller: create_test_bytes32(&env, 0xFF),
        min_finality_threshold: u32::MAX,
        finality_threshold_executed: u32::MAX,
        message_body: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_version(&serialized).unwrap(), u32::MAX);
    assert_eq!(MessageV2::get_source_domain(&serialized).unwrap(), u32::MAX);
    assert_eq!(
        MessageV2::get_destination_domain(&serialized).unwrap(),
        u32::MAX
    );
    assert_eq!(
        MessageV2::get_min_finality_threshold(&serialized).unwrap(),
        u32::MAX
    );
    assert_eq!(
        MessageV2::get_finality_threshold_executed(&serialized).unwrap(),
        u32::MAX
    );
}

#[test]
fn test_large_message_body() {
    let env = Env::default();
    let large_body = Bytes::from_slice(&env, &[0xAB; 1000]);

    let message = MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: create_test_bytes32(&env, 0),
        sender: create_test_bytes32(&env, 1),
        recipient: create_test_bytes32(&env, 2),
        destination_caller: create_test_bytes32(&env, 3),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: large_body.clone(),
    };

    let serialized = message.serialize(&env);
    let message_body = MessageV2::get_message_body(&serialized);

    assert_eq!(message_body.len(), 1000);
    assert_eq!(message_body, large_body);
}

#[rstest]
#[case::all_zeros(0, 0, 0, 0, 0, 0, 0, 0, 0)]
#[case::typical(1, 100, 200, 42, 1, 2, 3, 500, 1000)]
#[case::all_max(
    u32::MAX, u32::MAX, u32::MAX, 0xFF, 0xFF, 0xFF, 0xFF, u32::MAX, u32::MAX
)]
#[case::mixed(42, 1, 9, 0xAB, 0xCD, 0xEF, 0x12, 12345, 67890)]
fn test_serialize_deserialize_roundtrip(
    #[case] version: u32,
    #[case] src: u32,
    #[case] dst: u32,
    #[case] nonce_fill: u8,
    #[case] sender_fill: u8,
    #[case] recipient_fill: u8,
    #[case] caller_fill: u8,
    #[case] min_fin: u32,
    #[case] fin_exec: u32,
) {
    let env = Env::default();
    let message = MessageV2 {
        version,
        source_domain: src,
        destination_domain: dst,
        nonce: create_test_bytes32(&env, nonce_fill),
        sender: create_test_bytes32(&env, sender_fill),
        recipient: create_test_bytes32(&env, recipient_fill),
        destination_caller: create_test_bytes32(&env, caller_fill),
        min_finality_threshold: min_fin,
        finality_threshold_executed: fin_exec,
        message_body: Bytes::from_slice(&env, &[0x01, 0x02, 0x03]),
    };

    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_version(&serialized).unwrap(), version);
    assert_eq!(MessageV2::get_source_domain(&serialized).unwrap(), src);
    assert_eq!(MessageV2::get_destination_domain(&serialized).unwrap(), dst);
    assert_eq!(
        MessageV2::get_nonce(&serialized).unwrap(),
        create_test_bytes32(&env, nonce_fill)
    );
    assert_eq!(
        MessageV2::get_sender(&serialized).unwrap(),
        create_test_bytes32(&env, sender_fill)
    );
    assert_eq!(
        MessageV2::get_recipient(&serialized).unwrap(),
        create_test_bytes32(&env, recipient_fill)
    );
    assert_eq!(
        MessageV2::get_destination_caller(&serialized).unwrap(),
        create_test_bytes32(&env, caller_fill)
    );
    assert_eq!(
        MessageV2::get_min_finality_threshold(&serialized).unwrap(),
        min_fin
    );
    assert_eq!(
        MessageV2::get_finality_threshold_executed(&serialized).unwrap(),
        fin_exec
    );
    assert_eq!(
        MessageV2::get_message_body(&serialized),
        Bytes::from_slice(&env, &[0x01, 0x02, 0x03])
    );
}

#[test]
fn test_bytes32_fields_preserve_all_bytes() {
    let env = Env::default();

    let mut nonce_bytes = [0u8; 32];
    let mut sender_bytes = [0u8; 32];
    let mut recipient_bytes = [0u8; 32];
    let mut caller_bytes = [0u8; 32];

    for i in 0..32 {
        nonce_bytes[i] = i as u8;
        sender_bytes[i] = (i + 32) as u8;
        recipient_bytes[i] = (i + 64) as u8;
        caller_bytes[i] = (i + 96) as u8;
    }

    let nonce = BytesN::from_array(&env, &nonce_bytes);
    let sender = BytesN::from_array(&env, &sender_bytes);
    let recipient = BytesN::from_array(&env, &recipient_bytes);
    let destination_caller = BytesN::from_array(&env, &caller_bytes);

    let message = MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: nonce.clone(),
        sender: sender.clone(),
        recipient: recipient.clone(),
        destination_caller: destination_caller.clone(),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(MessageV2::get_nonce(&serialized).unwrap(), nonce);
    assert_eq!(MessageV2::get_sender(&serialized).unwrap(), sender);
    assert_eq!(MessageV2::get_recipient(&serialized).unwrap(), recipient);
    assert_eq!(
        MessageV2::get_destination_caller(&serialized).unwrap(),
        destination_caller
    );
}

#[rstest]
#[case::empty(0)]
#[case::one_byte(1)]
#[case::ten_bytes(10)]
#[case::hundred_bytes(100)]
#[case::five_hundred_bytes(500)]
#[case::thousand_bytes(1000)]
#[case::five_thousand_bytes(5000)]
fn test_message_body_various_sizes(#[case] size: usize) {
    extern crate alloc;
    use alloc::vec;

    let env = Env::default();
    let body = Bytes::from_slice(&env, &vec![0xABu8; size]);
    let message = MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: create_test_bytes32(&env, 0),
        sender: create_test_bytes32(&env, 1),
        recipient: create_test_bytes32(&env, 2),
        destination_caller: create_test_bytes32(&env, 3),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: body.clone(),
    };

    let serialized = message.serialize(&env);
    let parsed_body = MessageV2::get_message_body(&serialized);

    assert_eq!(parsed_body.len() as usize, size);
    assert_eq!(parsed_body, body);
}

#[test]
fn test_message_body_with_special_bytes() {
    let env = Env::default();

    let special_body = Bytes::from_slice(&env, &[0x00, 0xFF, 0x7F, 0x80, 0x01, 0xFE]);
    let message = MessageV2 {
        version: 1,
        source_domain: 100,
        destination_domain: 200,
        nonce: create_test_bytes32(&env, 0),
        sender: create_test_bytes32(&env, 1),
        recipient: create_test_bytes32(&env, 2),
        destination_caller: create_test_bytes32(&env, 3),
        min_finality_threshold: 500,
        finality_threshold_executed: 1000,
        message_body: special_body.clone(),
    };

    let serialized = message.serialize(&env);
    assert_eq!(MessageV2::get_message_body(&serialized), special_body);
}

// ================================
// Live message roundtrip tests
// ================================

#[test]
fn test_decode_and_reencode_live_message_v2() {
    let env = Env::default();

    // Live CCTP V2 message: Arbitrum (domain 3) → Base (domain 6)
    // burnToken: 0xaf88d065e77c8cc2239327c5edb3a432268e5831 (USDC on Arbitrum)
    // amount: 24610208, maxFee: 2514, feeExecuted: 2460, expirationBlock: 32112394
    // Includes 160 bytes of hookData
    let live_hex = "\
        00000001\
        00000003\
        00000006\
        00002d912c9d4d19847afdccdfd737a664c10dfc92be1f42104624784fd4cec3\
        00000000000000000000000028b5a0e9c621a5badaa536219b3a228c8168cf5d\
        00000000000000000000000028b5a0e9c621a5badaa536219b3a228c8168cf5d\
        00000000000000000000000009b043840cd2f32687ec6b63fb0412585de39822\
        000003e8\
        000003e8\
        00000001\
        000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831\
        00000000000000000000000009b043840cd2f32687ec6b63fb0412585de39822\
        00000000000000000000000000000000000000000000000000000000017769a0\
        00000000000000000000000009b043840cd2f32687ec6b63fb0412585de39822\
        00000000000000000000000000000000000000000000000000000000000009d2\
        000000000000000000000000000000000000000000000000000000000000099c\
        0000000000000000000000000000000000000000000000000000000001e89f0a\
        00000000000000000000000000000000000000000000000000000000017769a0\
        b24eca7c5544adcedd850abb9705eb912c1bfa1c4ad09e3dfc19a469fb7344bc\
        000000000000000000000000ae68b7117be0026cbd4366303f74eecbb19e4042\
        0000000000000000000000000000000000000000000000000000000001770ebb\
        00000000000000000000000000000000000000000000000000000000685a5785";

    let original = hex_to_bytes(&env, live_hex);

    // Decode outer message fields
    let version = MessageV2::get_version(&original).unwrap();
    let source_domain = MessageV2::get_source_domain(&original).unwrap();
    let destination_domain = MessageV2::get_destination_domain(&original).unwrap();
    let nonce = MessageV2::get_nonce(&original).unwrap();
    let sender = MessageV2::get_sender(&original).unwrap();
    let recipient = MessageV2::get_recipient(&original).unwrap();
    let destination_caller = MessageV2::get_destination_caller(&original).unwrap();
    let min_finality_threshold = MessageV2::get_min_finality_threshold(&original).unwrap();
    let finality_threshold_executed =
        MessageV2::get_finality_threshold_executed(&original).unwrap();
    let message_body = MessageV2::get_message_body(&original);

    // Decode the inner burn message body
    assert!(BurnMessageV2::validate_format(&message_body).is_ok());
    let burn_version = BurnMessageV2::get_version(&message_body).unwrap();
    let burn_token = BurnMessageV2::get_burn_token(&message_body).unwrap();
    let mint_recipient = BurnMessageV2::get_mint_recipient(&message_body).unwrap();
    let amount = BurnMessageV2::get_amount(&env, &message_body).unwrap();
    let message_sender = BurnMessageV2::get_message_sender(&message_body).unwrap();
    let max_fee = BurnMessageV2::get_max_fee(&env, &message_body).unwrap();
    let fee_executed = BurnMessageV2::get_fee_executed(&env, &message_body).unwrap();
    let expiration_block = BurnMessageV2::get_expiration_block(&env, &message_body).unwrap();
    let hook_data = BurnMessageV2::get_hook_data(&message_body);

    // Re-encode the burn message from decoded fields
    let re_encoded_burn = BurnMessageV2 {
        version: burn_version,
        burn_token,
        mint_recipient,
        amount,
        message_sender,
        max_fee,
        fee_executed,
        expiration_block,
        hook_data,
    }
    .serialize(&env);

    // Re-encode the full message with the re-encoded burn body
    let re_encoded = MessageV2 {
        version,
        source_domain,
        destination_domain,
        nonce,
        sender,
        recipient,
        destination_caller,
        min_finality_threshold,
        finality_threshold_executed,
        message_body: re_encoded_burn,
    }
    .serialize(&env);

    assert_eq!(
        re_encoded, original,
        "Re-encoded message must be byte-identical to the original live message"
    );
}

#[test]
fn test_format_for_relay_empty_body() {
    let env = Env::default();

    let serialized = MessageV2::format_for_relay(
        &env,
        1,
        100,
        200,
        create_test_bytes32(&env, 1),
        create_test_bytes32(&env, 2),
        create_test_bytes32(&env, 3),
        500,
        Bytes::new(&env),
    );

    assert_eq!(MessageV2::get_message_body(&serialized).len(), 0);
}
