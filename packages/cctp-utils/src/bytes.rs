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
//! Byte manipulation utilities
//!
//! This module provides common functions for serializing and deserializing
//! values to/from bytes in big-endian format, compatible with EVM encoding.

use soroban_sdk::{Bytes, BytesN, Env, U256};

/// Error types for byte read operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByteReadError {
    /// Failed to read bytes at the specified index (out of bounds)
    OutOfBounds,
    /// A value exceeds the maximum representable value for the target type
    ValueTooLarge,
}

/// Reads a u32 from bytes at the given index (big-endian).
pub fn read_u32(data: &Bytes, index: u32) -> Result<u32, ByteReadError> {
    let b0 = data.get(index).ok_or(ByteReadError::OutOfBounds)?;
    let b1 = data.get(index + 1).ok_or(ByteReadError::OutOfBounds)?;
    let b2 = data.get(index + 2).ok_or(ByteReadError::OutOfBounds)?;
    let b3 = data.get(index + 3).ok_or(ByteReadError::OutOfBounds)?;

    Ok(u32::from_be_bytes([b0, b1, b2, b3]))
}

/// Reads 32 bytes from the data at the given index.
pub(crate) fn read_bytes32(data: &Bytes, index: u32) -> Result<BytesN<32>, ByteReadError> {
    if data.len() < index + 32 {
        return Err(ByteReadError::OutOfBounds);
    }
    data.slice(index..index + 32)
        .try_into()
        .map_err(|_| ByteReadError::OutOfBounds)
}

/// Reads a u256 from bytes at the given index (big-endian).
///
/// # Arguments
///
/// * `env` - Access to the Soroban environment
/// * `data` - The byte buffer to read from
/// * `index` - The starting index in the buffer
///
/// # Returns
///
/// The value as `U256`, or an error if it cannot be read.
///
/// # Errors
///
/// * [`ByteReadError::OutOfBounds`] – If there aren't enough bytes at the index.
pub(crate) fn read_u256(env: &Env, data: &Bytes, index: u32) -> Result<U256, ByteReadError> {
    if data.len() < index + 32 {
        return Err(ByteReadError::OutOfBounds);
    }
    let bytes_slice = data.slice(index..index + 32);
    Ok(U256::from_be_bytes(env, &bytes_slice))
}

/// Converts a U256 to i128.
///
/// This function validates that the U256 value fits within the positive range
/// of an i128 (i.e., value <= i128::MAX).
///
/// # Arguments
///
/// * `env` - Access to the Soroban environment
/// * `value` - The U256 value to convert
///
/// # Returns
///
/// The value as `i128`, or an error if it doesn't fit.
///
/// # Errors
///
/// * [`ByteReadError::ValueTooLarge`] – If the value > i128::MAX (2^127 - 1).
pub fn u256_to_positive_i128(value: &U256) -> Result<i128, ByteReadError> {
    let value_u128 = value.to_u128().ok_or(ByteReadError::ValueTooLarge)?;
    if value_u128 > i128::MAX as u128 {
        return Err(ByteReadError::ValueTooLarge);
    }
    Ok(value_u128 as i128)
}

/// Converts a U256 to u32.
///
/// # Arguments
///
/// * `value` - The U256 value to convert
///
/// # Returns
///
/// The value as `u32`, or an error if it doesn't fit.
///
/// # Errors
///
/// * [`ByteReadError::ValueTooLarge`] – If the value > u32::MAX (2^32 - 1).
pub fn u256_to_u32(value: &U256) -> Result<u32, ByteReadError> {
    let value_u128 = value.to_u128().ok_or(ByteReadError::ValueTooLarge)?;
    u32::try_from(value_u128).map_err(|_| ByteReadError::ValueTooLarge)
}

/// Converts a positive i128 to U256.
///
/// This function validates that the i128 value is non-negative before conversion.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment
/// * `value` - The i128 value to convert (must be >= 0)
///
/// # Returns
///
/// The value as `U256`, or an error if the value is negative.
///
/// # Errors
///
/// * [`ByteReadError::ValueTooLarge`] – If the value is negative.
pub fn positive_i128_to_u256(e: &Env, value: i128) -> Result<U256, ByteReadError> {
    let value_u128: u128 = value.try_into().map_err(|_| ByteReadError::ValueTooLarge)?;
    Ok(U256::from_u128(e, value_u128))
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_u256_to_positive_i128_success() {
        let env = Env::default();

        // Zero
        let zero = U256::from_u32(&env, 0);
        assert_eq!(u256_to_positive_i128(&zero), Ok(0i128));

        // Small positive value
        let small = U256::from_u32(&env, 12345);
        assert_eq!(u256_to_positive_i128(&small), Ok(12345i128));

        // i128::MAX
        let max_i128 = U256::from_u128(&env, i128::MAX as u128);
        assert_eq!(u256_to_positive_i128(&max_i128), Ok(i128::MAX));
    }

    #[test]
    fn test_u256_to_positive_i128_value_too_large() {
        let env = Env::default();

        // i128::MAX + 1 should fail
        let too_large = U256::from_u128(&env, (i128::MAX as u128) + 1);
        assert_eq!(
            u256_to_positive_i128(&too_large),
            Err(ByteReadError::ValueTooLarge)
        );

        // u128::MAX should fail
        let u128_max = U256::from_u128(&env, u128::MAX);
        assert_eq!(
            u256_to_positive_i128(&u128_max),
            Err(ByteReadError::ValueTooLarge)
        );
    }

    #[test]
    fn test_u256_to_positive_i128_exceeds_u128() {
        let env = Env::default();

        // Value larger than u128::MAX (uses full 256 bits)
        // Create by multiplying u128::MAX by 2
        let large = U256::from_u128(&env, u128::MAX).mul(&U256::from_u32(&env, 2));
        assert_eq!(
            u256_to_positive_i128(&large),
            Err(ByteReadError::ValueTooLarge)
        );
    }

    #[test]
    fn test_u256_to_u32_success() {
        let env = Env::default();

        // Zero
        let zero = U256::from_u32(&env, 0);
        assert_eq!(u256_to_u32(&zero), Ok(0u32));

        // Small positive value
        let small = U256::from_u32(&env, 42);
        assert_eq!(u256_to_u32(&small), Ok(42u32));

        // u32::MAX
        let max = U256::from_u128(&env, u32::MAX as u128);
        assert_eq!(u256_to_u32(&max), Ok(u32::MAX));
    }

    #[test]
    fn test_u256_to_u32_value_too_large() {
        let env = Env::default();

        // u32::MAX + 1 should fail
        let too_large = U256::from_u128(&env, (u32::MAX as u128) + 1);
        assert_eq!(u256_to_u32(&too_large), Err(ByteReadError::ValueTooLarge));

        // u128::MAX should fail
        let u128_max = U256::from_u128(&env, u128::MAX);
        assert_eq!(u256_to_u32(&u128_max), Err(ByteReadError::ValueTooLarge));
    }

    #[test]
    fn test_u256_to_u32_exceeds_u128() {
        let env = Env::default();

        // Value larger than u128::MAX (uses full 256 bits)
        let large = U256::from_u128(&env, u128::MAX).mul(&U256::from_u32(&env, 2));
        assert_eq!(u256_to_u32(&large), Err(ByteReadError::ValueTooLarge));
    }

    #[test]
    fn test_positive_i128_to_u256_success() {
        let env = Env::default();

        // Zero
        assert_eq!(positive_i128_to_u256(&env, 0), Ok(U256::from_u32(&env, 0)));

        // Small positive value
        assert_eq!(
            positive_i128_to_u256(&env, 12345),
            Ok(U256::from_u128(&env, 12345))
        );

        // i128::MAX
        assert_eq!(
            positive_i128_to_u256(&env, i128::MAX),
            Ok(U256::from_u128(&env, i128::MAX as u128))
        );
    }

    #[test]
    fn test_positive_i128_to_u256_negative_value() {
        let env = Env::default();

        // Negative value should fail
        assert_eq!(
            positive_i128_to_u256(&env, -1),
            Err(ByteReadError::ValueTooLarge)
        );

        // i128::MIN should fail
        assert_eq!(
            positive_i128_to_u256(&env, i128::MIN),
            Err(ByteReadError::ValueTooLarge)
        );
    }

    #[test]
    fn test_roundtrip_conversion() {
        let env = Env::default();

        // Test roundtrip: i128 -> U256 -> i128
        let original: i128 = 1_000_000_000_000;
        let u256_val = positive_i128_to_u256(&env, original).unwrap();
        let back = u256_to_positive_i128(&u256_val).unwrap();
        assert_eq!(original, back);

        // Test roundtrip with i128::MAX
        let max_val = i128::MAX;
        let u256_max = positive_i128_to_u256(&env, max_val).unwrap();
        let back_max = u256_to_positive_i128(&u256_max).unwrap();
        assert_eq!(max_val, back_max);
    }
}
