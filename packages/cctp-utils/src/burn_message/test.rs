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
use super::*;
use crate::bytes::{u256_to_positive_i128, ByteReadError};
use crate::test_utils::{create_test_bytes32, hex_to_bytes};
use rstest::rstest;
use soroban_sdk::{Env, U256};

// ================================
// Helper functions
// ================================

fn create_test_burn_message(env: &Env) -> BurnMessageV2 {
    BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(env, 10),
        mint_recipient: create_test_bytes32(env, 20),
        amount: U256::from_u128(env, 1_000_000_000_000),
        message_sender: create_test_bytes32(env, 30),
        max_fee: U256::from_u128(env, 100_000),
        fee_executed: U256::from_u128(env, 50_000),
        expiration_block: U256::from_u32(env, 999_999),
        hook_data: Bytes::from_slice(env, &[0xCA, 0xFE, 0xBA, 0xBE]),
    }
}

// ================================
// Serialization tests
// ================================

#[test]
fn test_serialize_produces_correct_length() {
    let env = Env::default();

    // With hook data: 228 bytes header + 4 bytes hook data
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);
    assert_eq!(serialized.len(), 232);

    // Without hook data: 228 bytes header only
    let empty_hook_message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: Bytes::new(&env),
    };
    let serialized_empty = empty_hook_message.serialize(&env);
    assert_eq!(serialized_empty.len(), 228);
}

// ================================
// Getter function tests
// ================================

#[test]
fn test_get_version() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(BurnMessageV2::get_version(&serialized).unwrap(), 1);
}

#[test]
fn test_get_burn_token() {
    let env = Env::default();
    let expected_token = create_test_bytes32(&env, 10);
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_burn_token(&serialized).unwrap(),
        expected_token
    );
}

#[test]
fn test_get_mint_recipient() {
    let env = Env::default();
    let expected_recipient = create_test_bytes32(&env, 20);
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_mint_recipient(&serialized).unwrap(),
        expected_recipient
    );
}

#[test]
fn test_get_amount() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_amount(&env, &serialized).unwrap(),
        U256::from_u128(&env, 1_000_000_000_000)
    );
}

#[test]
fn test_get_message_sender() {
    let env = Env::default();
    let expected_sender = create_test_bytes32(&env, 30);
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_message_sender(&serialized).unwrap(),
        expected_sender
    );
}

#[test]
fn test_get_max_fee() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &serialized).unwrap(),
        U256::from_u128(&env, 100_000)
    );
}

#[test]
fn test_get_fee_executed() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &serialized).unwrap(),
        U256::from_u128(&env, 50_000)
    );
}

#[test]
fn test_get_expiration_block() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &serialized).unwrap(),
        U256::from_u32(&env, 999_999)
    );
}

#[test]
fn test_get_hook_data() {
    let env = Env::default();
    let expected_hook_data = Bytes::from_slice(&env, &[0xCA, 0xFE, 0xBA, 0xBE]);
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_hook_data(&serialized),
        expected_hook_data
    );
}

// ================================
// Format for relay tests
// ================================

#[test]
fn test_format_for_relay_roundtrip_all_fields() {
    let env = Env::default();
    let version = 42u32;
    let burn_token = create_test_bytes32(&env, 10);
    let mint_recipient = create_test_bytes32(&env, 20);
    let amount = U256::from_u128(&env, 5_000_000_000);
    let message_sender = create_test_bytes32(&env, 30);
    let max_fee = U256::from_u128(&env, 250_000);
    let hook_data = Bytes::from_slice(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);

    let serialized = BurnMessageV2::format_for_relay(
        &env,
        version,
        burn_token.clone(),
        mint_recipient.clone(),
        amount.clone(),
        message_sender.clone(),
        max_fee.clone(),
        hook_data.clone(),
    );

    // Verify all provided fields
    assert_eq!(BurnMessageV2::get_version(&serialized).unwrap(), version);
    assert_eq!(
        BurnMessageV2::get_burn_token(&serialized).unwrap(),
        burn_token
    );
    assert_eq!(
        BurnMessageV2::get_mint_recipient(&serialized).unwrap(),
        mint_recipient
    );
    assert_eq!(
        BurnMessageV2::get_amount(&env, &serialized).unwrap(),
        amount
    );
    assert_eq!(
        BurnMessageV2::get_message_sender(&serialized).unwrap(),
        message_sender
    );
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &serialized).unwrap(),
        max_fee
    );
    assert_eq!(BurnMessageV2::get_hook_data(&serialized), hook_data);

    // Verify auto-populated fields are set to zero
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
}

#[test]
fn test_format_for_relay_empty_hook_data() {
    let env = Env::default();
    let version = 1u32;
    let burn_token = create_test_bytes32(&env, 10);
    let mint_recipient = create_test_bytes32(&env, 20);
    let amount = U256::from_u128(&env, 1_000_000);
    let message_sender = create_test_bytes32(&env, 30);
    let max_fee = U256::from_u32(&env, 0);
    let hook_data = Bytes::new(&env);

    let serialized = BurnMessageV2::format_for_relay(
        &env,
        version,
        burn_token.clone(),
        mint_recipient.clone(),
        amount,
        message_sender.clone(),
        max_fee,
        hook_data.clone(),
    );

    assert_eq!(BurnMessageV2::get_hook_data(&serialized).len(), 0);
}

// ================================
// Validation tests
// ================================

#[test]
fn test_validate_format_valid_message() {
    let env = Env::default();
    let message = create_test_burn_message(&env);
    let serialized = message.serialize(&env);

    assert!(BurnMessageV2::validate_format(&serialized).is_ok());
}

#[test]
fn test_validate_format_exact_minimum_length() {
    let env = Env::default();
    let exact_min_data = Bytes::from_slice(&env, &[0u8; 228]);

    assert!(BurnMessageV2::validate_format(&exact_min_data).is_ok());
}

#[rstest]
#[case::empty(0)]
#[case::arbitrary_short(100)]
#[case::one_byte_short(227)]
fn test_validate_format_too_short(#[case] size: usize) {
    let env = Env::default();
    let data = Bytes::from_slice(&env, &[0u8; 256][..size]);
    assert_eq!(
        BurnMessageV2::validate_format(&data),
        Err(BurnMessageV2Error::MessageTooShort)
    );
}

// ================================
// Edge case tests
// ================================

#[test]
fn test_max_u32_version() {
    let env = Env::default();
    let message = BurnMessageV2 {
        version: u32::MAX,
        burn_token: create_test_bytes32(&env, 0xFF),
        mint_recipient: create_test_bytes32(&env, 0xFF),
        amount: U256::from_u32(&env, 0),
        message_sender: create_test_bytes32(&env, 0xFF),
        max_fee: U256::from_u32(&env, 0),
        fee_executed: U256::from_u32(&env, 0),
        expiration_block: U256::from_u32(&env, 0),
        hook_data: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(BurnMessageV2::get_version(&serialized).unwrap(), u32::MAX);
}

#[test]
fn test_max_i128_values() {
    let env = Env::default();
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, i128::MAX as u128),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, i128::MAX as u128),
        fee_executed: U256::from_u128(&env, i128::MAX as u128),
        expiration_block: U256::from_u32(&env, u32::MAX),
        hook_data: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_amount(&env, &serialized).unwrap(),
        U256::from_u128(&env, i128::MAX as u128)
    );
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &serialized).unwrap(),
        U256::from_u128(&env, i128::MAX as u128)
    );
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &serialized).unwrap(),
        U256::from_u128(&env, i128::MAX as u128)
    );
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &serialized).unwrap(),
        U256::from_u32(&env, u32::MAX)
    );
}

#[test]
fn test_large_hook_data() {
    let env = Env::default();
    let large_hook = Bytes::from_slice(&env, &[0xAB; 1000]);

    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: large_hook.clone(),
    };

    let serialized = message.serialize(&env);
    let hook_data = BurnMessageV2::get_hook_data(&serialized);

    assert_eq!(hook_data.len(), 1000);
    assert_eq!(hook_data, large_hook);
}

#[test]
fn test_zero_amount_and_fees() {
    let env = Env::default();
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u32(&env, 0),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u32(&env, 0),
        fee_executed: U256::from_u32(&env, 0),
        expiration_block: U256::from_u32(&env, 0),
        hook_data: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_amount(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &serialized).unwrap(),
        U256::from_u32(&env, 0)
    );
}

#[test]
fn test_u256_to_positive_i128_upper_bits_too_large() {
    let env = Env::default();

    // Create a valid burn message first
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: Bytes::new(&env),
    };
    let mut serialized = message.serialize(&env);

    // Manually set a non-zero byte in the upper 128 bits of the amount field (index 68)
    // The upper bits are at indices 68-83, lower bits at 84-99
    serialized.set(68, 0x01); // Set first byte of amount's upper 128 bits to non-zero

    // get_amount returns U256 successfully (it doesn't validate the range)
    let amount_u256 = BurnMessageV2::get_amount(&env, &serialized).unwrap();

    // But u256_to_positive_i128 fails because the value exceeds i128::MAX
    assert_eq!(
        u256_to_positive_i128(&amount_u256),
        Err(ByteReadError::ValueTooLarge)
    );
}

#[test]
fn test_u256_to_positive_i128_sign_bit_set() {
    let env = Env::default();

    // Create a valid burn message first
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: Bytes::new(&env),
    };
    let mut serialized = message.serialize(&env);

    // Simulate a uint256 value of 2^127 (which is valid on EVM but would be negative as i128)
    // Set the lower 128 bits to 0x80000000000000000000000000000000 (2^127)
    for i in 68..100 {
        serialized.set(i, 0x00);
    }
    serialized.set(84, 0x80); // Set sign bit of lower 128 bits

    // get_amount returns U256 successfully
    let amount_u256 = BurnMessageV2::get_amount(&env, &serialized).unwrap();

    let i128_max_plus_one = U256::from_u128(&env, i128::MAX as u128).add(&U256::from_u128(&env, 1));
    assert_eq!(amount_u256, i128_max_plus_one);

    // But u256_to_positive_i128 fails because 2^127 > i128::MAX
    assert_eq!(
        u256_to_positive_i128(&amount_u256),
        Err(ByteReadError::ValueTooLarge)
    );
}

#[test]
fn test_expiration_block_large_u256_value() {
    let env = Env::default();

    // Create a valid burn message first
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: Bytes::new(&env),
    };
    let mut serialized = message.serialize(&env);

    // Manually set a non-zero byte in the upper bits of the expiration_block field (index 196)
    // With U256, this is now a valid large value, not an error
    serialized.set(196, 0x01); // Set first byte of expiration_block's upper bits to non-zero

    // Should succeed - U256 can hold this value
    let result = BurnMessageV2::get_expiration_block(&env, &serialized);
    assert!(result.is_ok());
    // Verify it's greater than u32::MAX
    let exp_block = result.unwrap();
    assert!(exp_block > U256::from_u32(&env, u32::MAX));
}

// ================================
// Real message tests
// ================================

#[test]
fn test_parse_real_burn_message_v2() {
    let env = Env::default();

    // burnToken: 0x036cbd53842c5426634e7929541ec2318f3dcf7e
    // mintRecipient: 2TW9CZtaZC6TDSxLBYAYQSzwE5JmeZgNovEBPfRTvZZm (Solana address)
    // amount: 10000
    // messageSender: 0xc5567a5e3370d4dbfb0540025078e283e36a363d (Base address)
    // maxFee: 1, feeExecuted: 1, expirationBlock: 25000000, hookData: empty
    #[rustfmt::skip]
    let message_bytes: [u8; 228] = [
        // version (4 bytes) = 1
        0x00, 0x00, 0x00, 0x01,
        // burnToken (32 bytes) - EVM address left-padded with zeros
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x03, 0x6c, 0xbd, 0x53,
        0x84, 0x2c, 0x54, 0x26, 0x63, 0x4e, 0x79, 0x29,
        0x54, 0x1e, 0xc2, 0x31, 0x8f, 0x3d, 0xcf, 0x7e,
        // mintRecipient (32 bytes) - Stellar public key bytes
        0x15, 0xa5, 0xbc, 0xfb, 0x95, 0x72, 0xff, 0x8d,
        0x60, 0x7f, 0xcd, 0xa0, 0xa9, 0x1c, 0xf8, 0x50,
        0xfd, 0x19, 0x4b, 0xe7, 0x28, 0xb2, 0x85, 0xa7,
        0xb8, 0x15, 0x40, 0xac, 0x1d, 0x77, 0x35, 0x54,
        // amount (32 bytes) = 10000 (0x2710)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x27, 0x10,
        // messageSender (32 bytes) - EVM address left-padded with zeros
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xc5, 0x56, 0x7a, 0x5e,
        0x33, 0x70, 0xd4, 0xdb, 0xfb, 0x05, 0x40, 0x02,
        0x50, 0x78, 0xe2, 0x83, 0xe3, 0x6a, 0x36, 0x3d,
        // maxFee (32 bytes) = 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // feeExecuted (32 bytes) = 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        // expirationBlock (32 bytes) = 25000000 (0x17D7840)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x7D, 0x78, 0x40,
        // hookData: empty (no bytes after index 228)
    ];

    let data = Bytes::from_slice(&env, &message_bytes);

    // Validate format
    assert!(BurnMessageV2::validate_format(&data).is_ok());

    // Verify version
    assert_eq!(BurnMessageV2::get_version(&data).unwrap(), 1);

    // Verify burnToken (0x036cbd53842c5426634e7929541ec2318f3dcf7e left-padded)
    #[rustfmt::skip]
    let expected_burn_token: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x03, 0x6c, 0xbd, 0x53,
        0x84, 0x2c, 0x54, 0x26, 0x63, 0x4e, 0x79, 0x29,
        0x54, 0x1e, 0xc2, 0x31, 0x8f, 0x3d, 0xcf, 0x7e,
    ];
    assert_eq!(
        BurnMessageV2::get_burn_token(&data).unwrap(),
        BytesN::from_array(&env, &expected_burn_token)
    );

    // Verify mintRecipient (address: 2TW9CZtaZC6TDSxLBYAYQSzwE5JmeZgNovEBPfRTvZZm)
    #[rustfmt::skip]
    let expected_mint_recipient: [u8; 32] = [
        0x15, 0xa5, 0xbc, 0xfb, 0x95, 0x72, 0xff, 0x8d,
        0x60, 0x7f, 0xcd, 0xa0, 0xa9, 0x1c, 0xf8, 0x50,
        0xfd, 0x19, 0x4b, 0xe7, 0x28, 0xb2, 0x85, 0xa7,
        0xb8, 0x15, 0x40, 0xac, 0x1d, 0x77, 0x35, 0x54,
    ];
    assert_eq!(
        BurnMessageV2::get_mint_recipient(&data).unwrap(),
        BytesN::from_array(&env, &expected_mint_recipient)
    );

    // Verify amount = 10000
    assert_eq!(
        BurnMessageV2::get_amount(&env, &data).unwrap(),
        U256::from_u128(&env, 10000)
    );

    // Verify messageSender (0xc5567a5e3370d4dbfb0540025078e283e36a363d left-padded)
    #[rustfmt::skip]
    let expected_message_sender: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xc5, 0x56, 0x7a, 0x5e,
        0x33, 0x70, 0xd4, 0xdb, 0xfb, 0x05, 0x40, 0x02,
        0x50, 0x78, 0xe2, 0x83, 0xe3, 0x6a, 0x36, 0x3d,
    ];
    assert_eq!(
        BurnMessageV2::get_message_sender(&data).unwrap(),
        BytesN::from_array(&env, &expected_message_sender)
    );

    // Verify maxFee = 1
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &data).unwrap(),
        U256::from_u128(&env, 1)
    );

    // Verify feeExecuted = 1
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &data).unwrap(),
        U256::from_u128(&env, 1)
    );

    // Verify expirationBlock = 25000000
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &data).unwrap(),
        U256::from_u32(&env, 25_000_000)
    );

    // Verify hookData is empty
    assert_eq!(BurnMessageV2::get_hook_data(&data).len(), 0);
}

#[test]
fn test_all_zeros_message() {
    let env = Env::default();
    let zeros = Bytes::from_slice(&env, &[0u8; 228]);

    assert!(BurnMessageV2::validate_format(&zeros).is_ok());
    assert_eq!(BurnMessageV2::get_version(&zeros).unwrap(), 0);
    assert_eq!(
        BurnMessageV2::get_amount(&env, &zeros).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &zeros).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &zeros).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &zeros).unwrap(),
        U256::from_u32(&env, 0)
    );
    assert_eq!(BurnMessageV2::get_hook_data(&zeros).len(), 0);
}

/// `None` for amount/max_fee/fee_executed means U256::MAX (all 0xFF bytes).
#[rstest]
#[case::all_zeros(0, Some(0u128), Some(0u128), Some(0u128), 0)]
#[case::ones(1, Some(1), Some(1), Some(1), 1)]
#[case::max_i128(u32::MAX, Some(i128::MAX as u128), Some(i128::MAX as u128), Some(i128::MAX as u128), u32::MAX)]
#[case::typical(42, Some(1000), Some(500), Some(250), 12345)]
#[case::large(
    100,
    Some(1_000_000_000_000),
    Some(1_000_000),
    Some(500_000),
    999_999_999
)]
#[case::u256_max(u32::MAX, None, None, None, u32::MAX)]
fn test_serialize_deserialize_roundtrip(
    #[case] version: u32,
    #[case] amount_raw: Option<u128>,
    #[case] max_fee_raw: Option<u128>,
    #[case] fee_executed_raw: Option<u128>,
    #[case] expiration_block: u32,
) {
    let env = Env::default();
    let u256_max = || U256::from_be_bytes(&env, &Bytes::from_slice(&env, &[0xFF; 32]));
    let amount = amount_raw.map_or_else(&u256_max, |v| U256::from_u128(&env, v));
    let max_fee = max_fee_raw.map_or_else(&u256_max, |v| U256::from_u128(&env, v));
    let fee_executed = fee_executed_raw.map_or_else(&u256_max, |v| U256::from_u128(&env, v));

    let message = BurnMessageV2 {
        version,
        burn_token: create_test_bytes32(&env, 0xAB),
        mint_recipient: create_test_bytes32(&env, 0xCD),
        amount: amount.clone(),
        message_sender: create_test_bytes32(&env, 0xEF),
        max_fee: max_fee.clone(),
        fee_executed: fee_executed.clone(),
        expiration_block: U256::from_u32(&env, expiration_block),
        hook_data: Bytes::from_slice(&env, &[0x01, 0x02, 0x03]),
    };

    let serialized = message.serialize(&env);

    assert_eq!(BurnMessageV2::get_version(&serialized).unwrap(), version);
    assert_eq!(
        BurnMessageV2::get_burn_token(&serialized).unwrap(),
        create_test_bytes32(&env, 0xAB)
    );
    assert_eq!(
        BurnMessageV2::get_mint_recipient(&serialized).unwrap(),
        create_test_bytes32(&env, 0xCD)
    );
    assert_eq!(
        BurnMessageV2::get_amount(&env, &serialized).unwrap(),
        amount
    );
    assert_eq!(
        BurnMessageV2::get_message_sender(&serialized).unwrap(),
        create_test_bytes32(&env, 0xEF)
    );
    assert_eq!(
        BurnMessageV2::get_max_fee(&env, &serialized).unwrap(),
        max_fee
    );
    assert_eq!(
        BurnMessageV2::get_fee_executed(&env, &serialized).unwrap(),
        fee_executed
    );
    assert_eq!(
        BurnMessageV2::get_expiration_block(&env, &serialized).unwrap(),
        U256::from_u32(&env, expiration_block)
    );
    assert_eq!(
        BurnMessageV2::get_hook_data(&serialized),
        Bytes::from_slice(&env, &[0x01, 0x02, 0x03])
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
fn test_hook_data_various_sizes(#[case] size: usize) {
    extern crate alloc;
    use alloc::vec;

    let env = Env::default();
    let hook_data = Bytes::from_slice(&env, &vec![0xABu8; size]);
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: hook_data.clone(),
    };

    let serialized = message.serialize(&env);
    let parsed_hook = BurnMessageV2::get_hook_data(&serialized);

    assert_eq!(parsed_hook.len() as usize, size);
    assert_eq!(parsed_hook, hook_data);
}

fn check_burn_getter(env: &Env, data: &Bytes, field: usize) -> Result<(), BurnMessageV2Error> {
    match field {
        0 => BurnMessageV2::get_version(data).map(|_| ()),
        1 => BurnMessageV2::get_burn_token(data).map(|_| ()),
        2 => BurnMessageV2::get_mint_recipient(data).map(|_| ()),
        3 => BurnMessageV2::get_amount(env, data).map(|_| ()),
        4 => BurnMessageV2::get_message_sender(data).map(|_| ()),
        5 => BurnMessageV2::get_max_fee(env, data).map(|_| ()),
        6 => BurnMessageV2::get_fee_executed(env, data).map(|_| ()),
        7 => BurnMessageV2::get_expiration_block(env, data).map(|_| ()),
        _ => panic!("unknown field index"),
    }
}

#[rstest]
#[case::version(0, 3, 4)]
#[case::burn_token(1, 35, 36)]
#[case::mint_recipient(2, 67, 68)]
#[case::amount(3, 99, 100)]
#[case::message_sender(4, 131, 132)]
#[case::max_fee(5, 163, 164)]
#[case::fee_executed(6, 195, 196)]
#[case::expiration_block(7, 227, 228)]
fn test_getter_boundary_conditions(
    #[case] field: usize,
    #[case] too_short: usize,
    #[case] exact: usize,
) {
    let env = Env::default();
    let short_data = Bytes::from_slice(&env, &[0u8; 256][..too_short]);
    let exact_data = Bytes::from_slice(&env, &[0u8; 256][..exact]);

    assert_eq!(
        check_burn_getter(&env, &short_data, field),
        Err(BurnMessageV2Error::FieldReadError)
    );
    assert!(check_burn_getter(&env, &exact_data, field).is_ok());
}

#[test]
fn test_bytes32_fields_preserve_all_bytes() {
    let env = Env::default();

    // Create bytes32 with specific pattern to verify no bytes are lost
    let mut burn_token_bytes = [0u8; 32];
    let mut mint_recipient_bytes = [0u8; 32];
    let mut message_sender_bytes = [0u8; 32];

    for i in 0..32 {
        burn_token_bytes[i] = i as u8;
        mint_recipient_bytes[i] = (i + 32) as u8;
        message_sender_bytes[i] = (i + 64) as u8;
    }

    let burn_token = BytesN::from_array(&env, &burn_token_bytes);
    let mint_recipient = BytesN::from_array(&env, &mint_recipient_bytes);
    let message_sender = BytesN::from_array(&env, &message_sender_bytes);

    let message = BurnMessageV2 {
        version: 1,
        burn_token: burn_token.clone(),
        mint_recipient: mint_recipient.clone(),
        amount: U256::from_u32(&env, 0),
        message_sender: message_sender.clone(),
        max_fee: U256::from_u32(&env, 0),
        fee_executed: U256::from_u32(&env, 0),
        expiration_block: U256::from_u32(&env, 0),
        hook_data: Bytes::new(&env),
    };

    let serialized = message.serialize(&env);

    assert_eq!(
        BurnMessageV2::get_burn_token(&serialized).unwrap(),
        burn_token
    );
    assert_eq!(
        BurnMessageV2::get_mint_recipient(&serialized).unwrap(),
        mint_recipient
    );
    assert_eq!(
        BurnMessageV2::get_message_sender(&serialized).unwrap(),
        message_sender
    );
}

// ================================
// Live message roundtrip test
// ================================

#[test]
fn test_decode_and_reencode_live_burn_message_v2() {
    let env = Env::default();

    // Burn message body extracted from a live CCTP V2 message: Arbitrum (domain 3) → Base (domain 6)
    // burnToken: 0xaf88d065e77c8cc2239327c5edb3a432268e5831 (USDC on Arbitrum)
    // mintRecipient: 0x09b043840cd2f32687ec6b63fb0412585de39822
    // amount: 24610208, maxFee: 2514, feeExecuted: 2460, expirationBlock: 32112394
    // Includes 160 bytes of hookData
    let live_burn_hex = "\
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

    let original = hex_to_bytes(&env, live_burn_hex);

    // Decode all fields via getters
    assert!(BurnMessageV2::validate_format(&original).is_ok());
    let version = BurnMessageV2::get_version(&original).unwrap();
    let burn_token = BurnMessageV2::get_burn_token(&original).unwrap();
    let mint_recipient = BurnMessageV2::get_mint_recipient(&original).unwrap();
    let amount = BurnMessageV2::get_amount(&env, &original).unwrap();
    let message_sender = BurnMessageV2::get_message_sender(&original).unwrap();
    let max_fee = BurnMessageV2::get_max_fee(&env, &original).unwrap();
    let fee_executed = BurnMessageV2::get_fee_executed(&env, &original).unwrap();
    let expiration_block = BurnMessageV2::get_expiration_block(&env, &original).unwrap();
    let hook_data = BurnMessageV2::get_hook_data(&original);

    // Reconstruct and re-serialize
    let re_encoded = BurnMessageV2 {
        version,
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

    assert_eq!(
        re_encoded, original,
        "Re-encoded burn message must be byte-identical to the original live message"
    );
}

#[test]
fn test_hook_data_with_special_bytes() {
    let env = Env::default();

    // Test hook data containing null bytes, 0xFF, and other special values
    let special_hook = Bytes::from_slice(&env, &[0x00, 0xFF, 0x7F, 0x80, 0x01, 0xFE]);
    let message = BurnMessageV2 {
        version: 1,
        burn_token: create_test_bytes32(&env, 10),
        mint_recipient: create_test_bytes32(&env, 20),
        amount: U256::from_u128(&env, 1_000_000),
        message_sender: create_test_bytes32(&env, 30),
        max_fee: U256::from_u128(&env, 100_000),
        fee_executed: U256::from_u128(&env, 50_000),
        expiration_block: U256::from_u32(&env, 999_999),
        hook_data: special_hook.clone(),
    };

    let serialized = message.serialize(&env);
    let parsed_hook = BurnMessageV2::get_hook_data(&serialized);

    assert_eq!(parsed_hook, special_hook);
}
