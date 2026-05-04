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
use rstest::rstest;
use soroban_sdk::Env;

// =============================================================================
// Helper: build hook data with full control over all fields
// =============================================================================

fn build_hook_data(env: &Env, magic: &[u8; 24], version: u32, strkey: &[u8]) -> Bytes {
    let mut data = Bytes::new(env);
    data.extend_from_slice(magic);
    data.extend_from_array(&version.to_be_bytes());
    data.extend_from_array(&(strkey.len() as u32).to_be_bytes());
    data.extend_from_slice(strkey);
    data
}

fn build_hook_data_raw_length(
    env: &Env,
    magic: &[u8; 24],
    version: u32,
    length: u32,
    payload: &[u8],
) -> Bytes {
    let mut data = Bytes::new(env);
    data.extend_from_slice(magic);
    data.extend_from_array(&version.to_be_bytes());
    data.extend_from_array(&length.to_be_bytes());
    data.extend_from_slice(payload);
    data
}

const ZERO_MAGIC: [u8; 24] = [0u8; 24];

const CIRCLE_MAGIC: [u8; 24] = [
    b'c', b'c', b't', b'p', b'-', b'f', b'o', b'r', b'w', b'a', b'r', b'd', 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

const TEST_C_STRKEY: &[u8] = b"CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE";
const TEST_G_STRKEY: &[u8] = b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5";
const TEST_M_STRKEY: &[u8] =
    b"MA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAAAAAAAAAPCICBKU";

// =============================================================================
// validate_hook_data: success cases
// =============================================================================

#[rstest]
#[case::c_account(TEST_C_STRKEY)]
#[case::g_account(TEST_G_STRKEY)]
#[case::m_account(TEST_M_STRKEY)]
fn test_validate_hook_data_strkey_types(#[case] strkey: &[u8]) {
    let env = Env::default();
    let hook_data = build_hook_data(&env, &ZERO_MAGIC, 0, strkey);

    let result = validate_hook_data(&env, &hook_data);

    let expected = MuxedAddress::from_string_bytes(&Bytes::from_slice(&env, strkey));
    assert_eq!(result.forward_recipient, expected);

    // Guard against SDK divergence between from_string_bytes and from_string
    let strkey_str = core::str::from_utf8(strkey).unwrap();
    let expected_from_string =
        MuxedAddress::from_string(&soroban_sdk::String::from_str(&env, strkey_str));
    assert_eq!(result.forward_recipient, expected_from_string);
}

#[test]
fn test_validate_hook_data_with_circle_forwarding_magic() {
    let env = Env::default();
    let hook_data = build_hook_data(&env, &CIRCLE_MAGIC, 0, TEST_C_STRKEY);

    let result = validate_hook_data(&env, &hook_data);

    let expected = MuxedAddress::from_string_bytes(&Bytes::from_slice(&env, TEST_C_STRKEY));
    assert_eq!(result.forward_recipient, expected);
}

#[test]
fn test_validate_hook_data_with_trailing_bytes() {
    let env = Env::default();
    let mut hook_data = build_hook_data(&env, &ZERO_MAGIC, 0, TEST_C_STRKEY);
    hook_data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    let result = validate_hook_data(&env, &hook_data);

    let expected = MuxedAddress::from_string_bytes(&Bytes::from_slice(&env, TEST_C_STRKEY));
    assert_eq!(result.forward_recipient, expected);
}

#[test]
fn test_validate_hook_data_strkey_exact_length() {
    let env = Env::default();
    // Length claims 56 and exactly 56 bytes present — should succeed
    let hook_data = build_hook_data_raw_length(&env, &ZERO_MAGIC, 0, 56, TEST_C_STRKEY);
    let result = validate_hook_data(&env, &hook_data);

    let expected = MuxedAddress::from_string_bytes(&Bytes::from_slice(&env, TEST_C_STRKEY));
    assert_eq!(result.forward_recipient, expected);
}

// =============================================================================
// validate_hook_data: error cases (parameterized)
// =============================================================================

/// When `magic` is `None`, `payload` is used as raw hook data bytes (bypasses header
/// construction). When `magic` is `Some(...)`, hook data is built from the full
/// (magic, version, length, payload) tuple.
#[rstest]
// --- HookDataTooShort (#7300) ---
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::empty(None, 0, 0, b"")]
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::below_min_header(None, 0, 0, &[0u8; 31])]
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::header_claims_data_but_none(Some(ZERO_MAGIC), 0, 1, b"")]
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::strkey_one_byte_short(Some(ZERO_MAGIC), 0, 56, &TEST_C_STRKEY[..55])]
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::length_exceeds_actual(Some(ZERO_MAGIC), 0, 100, &[b'A'; 10])]
#[should_panic(expected = "Error(Contract, #7300)")]
#[case::length_u32_max_overflow(Some(ZERO_MAGIC), 0, u32::MAX, b"")]
// --- InvalidHookVersion (#7313) ---
#[should_panic(expected = "Error(Contract, #7313)")]
#[case::version_1(Some(ZERO_MAGIC), 1, 56, TEST_C_STRKEY)]
#[should_panic(expected = "Error(Contract, #7313)")]
#[case::version_max(Some(ZERO_MAGIC), u32::MAX, 56, TEST_C_STRKEY)]
// --- Invalid strkey (unexpected length) ---
#[should_panic(expected = "unexpected strkey length")]
#[case::length_zero_no_payload(Some(ZERO_MAGIC), 0, 0, b"")]
#[should_panic(expected = "unexpected strkey length")]
#[case::length_zero_ignores_trailing(Some(ZERO_MAGIC), 0, 0, TEST_C_STRKEY)]
#[should_panic(expected = "unexpected strkey length")]
#[case::garbage_strkey(Some(ZERO_MAGIC), 0, 18, b"not-a-valid-strkey")]
// --- Invalid strkey (parse failure) ---
#[should_panic(expected = "couldn't process the string as strkey")]
#[case::invalid_strkey_prefix(
    Some(ZERO_MAGIC),
    0,
    56,
    b"XA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHES5"
)]
#[should_panic(expected = "couldn't process the string as strkey")]
#[case::bad_checksum(
    Some(ZERO_MAGIC),
    0,
    56,
    b"GA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQHESZ"
)]
fn test_validate_hook_data_rejects_invalid(
    #[case] magic: Option<[u8; 24]>,
    #[case] version: u32,
    #[case] length: u32,
    #[case] payload: &[u8],
) {
    let env = Env::default();
    let hook_data = match magic {
        Some(m) => build_hook_data_raw_length(&env, &m, version, length, payload),
        None => Bytes::from_slice(&env, payload),
    };
    validate_hook_data(&env, &hook_data);
}
