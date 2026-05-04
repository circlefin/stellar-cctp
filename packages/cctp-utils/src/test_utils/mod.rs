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
//! Test utilities for the utils package

use soroban_sdk::{Bytes, BytesN, Env};

/// Creates a BytesN<32> with a single byte value at the last position.
///
/// # Arguments
///
/// * `env` - Access to the Soroban environment
/// * `value` - The byte value to place at position 31
///
/// # Returns
///
/// A `BytesN<32>` with zeroes except for the last byte
pub fn create_test_bytes32(env: &Env, value: u8) -> BytesN<32> {
    let mut arr = [0u8; 32];
    arr[31] = value; // Put value in the last byte
    BytesN::from_array(env, &arr)
}

/// Converts a hex string to Soroban `Bytes`.
///
/// # Arguments
///
/// * `env` - Access to the Soroban environment
/// * `hex` - A hex-encoded string (no "0x" prefix, even number of characters)
pub fn hex_to_bytes(env: &Env, hex: &str) -> Bytes {
    let mut bytes = Bytes::new(env);
    for i in (0..hex.len()).step_by(2) {
        bytes.push_back(u8::from_str_radix(&hex[i..i + 2], 16).unwrap());
    }
    bytes
}
