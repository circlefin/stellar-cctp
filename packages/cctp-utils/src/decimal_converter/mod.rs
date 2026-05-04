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
//! Decimal Converter module
//!
//! This module provides functionality for converting amounts between local and canonical decimal formats.
//! USDC on Stellar has 7 decimals of precision, while the standard CCTP format uses 6 decimals.
//! This module handles the conversion between these formats and normalizes amounts for burning.

#[cfg(test)]
mod test;

/// Represents a pair of decimal configurations for local and canonical tokens
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenDecimalPair {
    /// Local decimals (e.g., 7 for Stellar USDC)
    pub local_decimals: u32,
    /// Canonical decimals (e.g., 6 for standard CCTP)
    pub canonical_decimals: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConversionError {
    /// The decimal scale difference is too large to compute the conversion factor
    DecimalScaleExceedsLimit,
    /// The amount exceeds the limit for conversion with the given decimal pair
    AmountExceedsLimit,
    /// The decimal scale is invalid. Local decimals must be greater than canonical decimals.
    InvalidDecimalScale,
}

/// Converts a local amount to a canonical amount
///
/// # Arguments
/// * `local_amount` - The amount in local decimal format
/// * `decimal_pair` - The decimal configuration pair
///
/// # Returns
/// The amount in canonical decimal format, or an error if the conversion would overflow
///
/// # Errors
/// Returns `ConversionError::AmountExceedsLimit` if the amount exceeds the limit for conversion
/// for the given TokenDecimalPair
///
/// # Examples
/// ```
/// // Converting from 7 decimals (Stellar) to 6 decimals (CCTP)
/// // 1234567 -> 123456
/// let amount = to_canonical_amount(1234567, TokenDecimalPair { local_decimals: 7, canonical_decimals: 6 }).unwrap();
/// assert_eq!(amount, 123456);
/// ```
pub fn to_canonical_amount(
    local_amount: i128,
    decimal_pair: TokenDecimalPair,
) -> Result<i128, ConversionError> {
    let TokenDecimalPair {
        local_decimals,
        canonical_decimals,
    } = decimal_pair;

    if local_decimals == canonical_decimals {
        return Ok(local_amount);
    }

    if local_decimals < canonical_decimals {
        return Err(ConversionError::InvalidDecimalScale);
    }

    let diff = local_decimals - canonical_decimals;
    let divisor = 10_i128
        .checked_pow(diff)
        .ok_or(ConversionError::DecimalScaleExceedsLimit)?;

    local_amount
        .checked_div(divisor)
        .ok_or(ConversionError::AmountExceedsLimit)
}

/// Converts a canonical amount to a local amount
///
/// # Arguments
/// * `canonical_amount` - The amount in canonical decimal format
/// * `decimal_pair` - The decimal configuration pair
///
/// # Returns
/// The amount in local decimal format, or an error if the conversion would overflow
///
/// # Errors
/// Returns `ConversionError::AmountExceedsLimit` if the amount exceeds the limit for conversion
/// for the given TokenDecimalPair
/// Returns `ConversionError::InvalidDecimalScale` if the decimal scale is invalid
///
/// # Examples
/// ```
/// // Converting from 6 decimals (CCTP) to 7 decimals (Stellar)
/// // 123456 -> 1234560
/// let amount = to_local_amount(123456, TokenDecimalPair { local_decimals: 7, canonical_decimals: 6 }).unwrap();
/// assert_eq!(amount, 1234560);
/// ```
pub fn to_local_amount(
    canonical_amount: i128,
    decimal_pair: TokenDecimalPair,
) -> Result<i128, ConversionError> {
    let TokenDecimalPair {
        local_decimals,
        canonical_decimals,
    } = decimal_pair;

    if local_decimals == canonical_decimals {
        return Ok(canonical_amount);
    }

    if local_decimals < canonical_decimals {
        return Err(ConversionError::InvalidDecimalScale);
    }

    let diff = local_decimals - canonical_decimals;
    let multiplier = 10_i128
        .checked_pow(diff)
        .ok_or(ConversionError::DecimalScaleExceedsLimit)?;
    canonical_amount
        .checked_mul(multiplier)
        .ok_or(ConversionError::AmountExceedsLimit)
}

/// Normalizes a local amount for burning by removing dust
///
/// This function rounds down the local amount to the nearest value that can be
/// represented in the canonical decimal format. This ensures that no "dust" remains
/// after the conversion.
///
/// # Arguments
/// * `local_amount` - The amount in local decimal format
/// * `decimal_pair` - The decimal configuration pair
///
/// # Returns
/// The normalized amount in local decimal format (with dust removed), or an error if the conversion would overflow
///
/// # Errors
/// Returns `ConversionError::AmountExceedsLimit` if the amount exceeds the limit for conversion
/// for the given TokenDecimalPair
///
/// # Examples
/// ```
/// // Normalizing 1234567 (7 decimals) to be compatible with 6 decimals
/// // 1234567 -> 1234560 (removes the last digit which would be lost in conversion)
/// let amount = normalize_for_burn(1234567, TokenDecimalPair { local_decimals: 7, canonical_decimals: 6 }).unwrap();
/// assert_eq!(amount, 1234560);
/// ```
pub fn normalize_for_burn(
    local_amount: i128,
    decimal_pair: TokenDecimalPair,
) -> Result<i128, ConversionError> {
    let TokenDecimalPair {
        local_decimals,
        canonical_decimals,
    } = decimal_pair;

    if local_decimals == canonical_decimals {
        // No normalization needed when local has equal decimals
        return Ok(local_amount);
    }

    if local_decimals < canonical_decimals {
        return Err(ConversionError::InvalidDecimalScale);
    }

    let diff = local_decimals - canonical_decimals;
    let divisor = 10_i128
        .checked_pow(diff)
        .ok_or(ConversionError::DecimalScaleExceedsLimit)?;
    let dust = local_amount
        .checked_rem(divisor)
        .ok_or(ConversionError::AmountExceedsLimit)?;
    local_amount
        .checked_sub(dust)
        .ok_or(ConversionError::AmountExceedsLimit)
}
