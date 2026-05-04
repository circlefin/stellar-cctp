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
use proptest::prelude::*;

// =============================================================================
// Standard Conversion Tests (7→6 decimals)
// Primary use case: Stellar USDC (7 decimals) ↔ CCTP (6 decimals)
// =============================================================================

#[test]
fn test_to_canonical_amount_standard_conversion() {
    // 7 decimals (Stellar) to 6 decimals (CCTP)
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // 1.234567 -> 0.123456
    assert_eq!(to_canonical_amount(1234567, decimal_pair).unwrap(), 123456);

    // 10.000000 -> 1.000000
    assert_eq!(
        to_canonical_amount(10000000, decimal_pair).unwrap(),
        1000000
    );

    // 0.000001 -> 0.000000
    assert_eq!(to_canonical_amount(1, decimal_pair).unwrap(), 0);
}

#[test]
fn test_to_local_amount_standard_conversion() {
    // 6 decimals (CCTP) to 7 decimals (Stellar)
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // 0.123456 -> 1.234560
    assert_eq!(to_local_amount(123456, decimal_pair).unwrap(), 1234560);

    // 1.000000 -> 10.000000
    assert_eq!(to_local_amount(1000000, decimal_pair).unwrap(), 10000000);

    // 0.000001 -> 0.000010
    assert_eq!(to_local_amount(1, decimal_pair).unwrap(), 10);
}

#[test]
fn test_normalize_for_burn() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // 1.234567 -> 1.234560
    assert_eq!(normalize_for_burn(1234567, decimal_pair).unwrap(), 1234560);

    // 1.234560 -> 1.234560 (already normalized)
    assert_eq!(normalize_for_burn(1234560, decimal_pair).unwrap(), 1234560);

    // 9.999999 -> 9.999990
    assert_eq!(normalize_for_burn(9999999, decimal_pair).unwrap(), 9999990);

    // 0.000001 -> 0.000000
    assert_eq!(normalize_for_burn(1, decimal_pair).unwrap(), 0);
}

// =============================================================================
// Invalid Decimal Scale Tests (local < canonical)
// Local decimals must be >= canonical decimals
// =============================================================================

#[test]
fn test_to_canonical_amount_local_smaller_than_canonical() {
    // 6 decimals (local) to 7 decimals (canonical) - should fail with InvalidDecimalScale
    let decimal_pair = TokenDecimalPair {
        local_decimals: 6,
        canonical_decimals: 7,
    };

    assert_eq!(
        to_canonical_amount(1234567, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        to_canonical_amount(1000000, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        to_canonical_amount(1, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
}

#[test]
fn test_to_local_amount_local_smaller_than_canonical_fails() {
    // 7 decimals (canonical) to 6 decimals (local) - should fail with InvalidDecimalScale
    let decimal_pair = TokenDecimalPair {
        local_decimals: 6,
        canonical_decimals: 7,
    };

    assert_eq!(
        to_local_amount(12345670, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        to_local_amount(1, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        to_local_amount(0, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_equal_decimals() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 6,
        canonical_decimals: 6,
    };

    let amount = 1234567;
    assert_eq!(to_canonical_amount(amount, decimal_pair).unwrap(), amount);
    assert_eq!(to_local_amount(amount, decimal_pair).unwrap(), amount);
    assert_eq!(normalize_for_burn(amount, decimal_pair).unwrap(), amount);
}

#[test]
fn test_zero_amounts() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    assert_eq!(to_canonical_amount(0, decimal_pair).unwrap(), 0);
    assert_eq!(to_local_amount(0, decimal_pair).unwrap(), 0);
    assert_eq!(normalize_for_burn(0, decimal_pair).unwrap(), 0);
}

#[test]
fn test_normalize_for_burn_local_less_than_canonical_fails() {
    // local_decimals < canonical_decimals should fail with InvalidDecimalScale
    let decimal_pair = TokenDecimalPair {
        local_decimals: 6,
        canonical_decimals: 7,
    };

    assert_eq!(
        normalize_for_burn(1234567, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        normalize_for_burn(1, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
    assert_eq!(
        normalize_for_burn(0, decimal_pair),
        Err(ConversionError::InvalidDecimalScale)
    );
}

#[test]
fn test_normalize_for_burn_negative_amounts() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // -1.234567 -> -1.234560 (rounds toward zero)
    assert_eq!(
        normalize_for_burn(-1234567, decimal_pair).unwrap(),
        -1234560
    );

    // -0.000001 -> 0 (dust removed)
    assert_eq!(normalize_for_burn(-1, decimal_pair).unwrap(), 0);

    // -9.999999 -> -9.999990
    assert_eq!(
        normalize_for_burn(-9999999, decimal_pair).unwrap(),
        -9999990
    );

    // i128::MIN normalizes without overflow
    let result = normalize_for_burn(i128::MIN, decimal_pair).unwrap();
    assert_eq!(result, (i128::MIN / 10) * 10);
}

// =============================================================================
// Large Value Tests
// =============================================================================

#[test]
fn test_large_amounts() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // 10 million USDC with 7 decimals
    let ten_million = 100_000_000_000_000_i128;
    let canonical = to_canonical_amount(ten_million, decimal_pair).unwrap();
    assert_eq!(canonical, 10_000_000_000_000_i128);

    let back = to_local_amount(canonical, decimal_pair).unwrap();
    assert_eq!(back, ten_million);
}

#[test]
fn test_max_safe_canonical_amount() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // Maximum safe canonical amount that can be converted to local without overflow
    // i128::MAX / 10 = 170141183460469231731687303715884105727 / 10
    let max_safe_canonical = i128::MAX / 10;
    let result = to_local_amount(max_safe_canonical, decimal_pair).unwrap();

    // Should be max_safe_canonical * 10
    assert_eq!(result, max_safe_canonical * 10);
}

#[test]
fn test_min_safe_canonical_amount() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // Minimum safe canonical amount (most negative) that can be converted without overflow
    // i128::MIN / 10 (rounding toward zero)
    let min_safe_canonical = i128::MIN / 10;
    let result = to_local_amount(min_safe_canonical, decimal_pair).unwrap();

    // Should be min_safe_canonical * 10
    assert_eq!(result, min_safe_canonical * 10);
}

#[test]
fn test_normalize_for_burn_with_large_values() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // Max value should normalize without panic (divides then multiplies)
    let max_value = 170141183460469231731687303715884105727; // i128::MAX
    let expected = 170141183460469231731687303715884105720; // i128:MAX with last digit set to 0

    let result = normalize_for_burn(max_value, decimal_pair).unwrap();

    assert_eq!(result, expected);
}

// =============================================================================
// Overflow/Underflow Tests
// =============================================================================

#[test]
fn test_overflow_to_local_amount() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // i128::MAX * 10 will overflow
    let result = to_local_amount(i128::MAX, decimal_pair);
    assert_eq!(result, Err(ConversionError::AmountExceedsLimit));
}

#[test]
fn test_underflow_to_local_amount_negative() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // i128::MIN * 10 will overflow (underflow in the negative direction)
    let result = to_local_amount(i128::MIN, decimal_pair);
    assert_eq!(result, Err(ConversionError::AmountExceedsLimit));
}

#[test]
fn test_to_canonical_amount_with_max_local() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // Converting from local to canonical divides, so no overflow
    let result = to_canonical_amount(i128::MAX, decimal_pair).unwrap();
    assert_eq!(result, i128::MAX / 10);
}

#[test]
fn test_to_canonical_amount_with_min_local() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: 7,
        canonical_decimals: 6,
    };

    // Converting from local to canonical divides, so no overflow
    let result = to_canonical_amount(i128::MIN, decimal_pair).unwrap();
    assert_eq!(result, i128::MIN / 10);
}

#[test]
fn test_to_canonical_amount_divisor_overflow() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: u32::MAX,
        canonical_decimals: 0,
    };

    let result = to_canonical_amount(1_000_000, decimal_pair);
    assert_eq!(result, Err(ConversionError::DecimalScaleExceedsLimit));
}

#[test]
fn test_to_local_amount_multiplier_overflow() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: u32::MAX,
        canonical_decimals: 0,
    };

    let result = to_local_amount(1_000_000, decimal_pair);
    assert_eq!(result, Err(ConversionError::DecimalScaleExceedsLimit));
}

#[test]
fn test_normalize_for_burn_divisor_overflow() {
    let decimal_pair = TokenDecimalPair {
        local_decimals: u32::MAX,
        canonical_decimals: 0,
    };

    let result = normalize_for_burn(1_000_000, decimal_pair);
    assert_eq!(result, Err(ConversionError::DecimalScaleExceedsLimit));
}

// =============================================================================
// Prop Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Tests that normalizing an amount twice produces the same result as normalizing once.
    /// This is a fundamental property - normalize_for_burn should remove all dust such that
    /// applying it again has no effect.
    #[test]
    fn prop_normalization_is_idempotent(
        amount in 0i128..=i128::MAX,
        local_decimals in 0u32..=18u32,
        canonical_decimals in 0u32..=18u32,
    ) {
        let decimal_pair = TokenDecimalPair { local_decimals, canonical_decimals };

        let normalized_once = normalize_for_burn(amount, decimal_pair);
        if normalized_once.is_err() {
            return Ok(());
        }
        let normalized_once = normalized_once.unwrap();

        let normalized_twice = normalize_for_burn(normalized_once, decimal_pair);
        if normalized_twice.is_err() {
            return Ok(());
        }
        let normalized_twice = normalized_twice.unwrap();

        prop_assert_eq!(
            normalized_once,
            normalized_twice,
            "Normalizing twice should equal normalizing once (idempotent): \
             amount={}, normalized_once={}, normalized_twice={}, decimals=({}, {})",
            amount, normalized_once, normalized_twice, local_decimals, canonical_decimals
        );
    }

    /// Tests that if amount1 < amount2, then their conversions maintain this ordering.
    /// This ensures that the conversion functions are monotonically increasing.
    #[test]
    fn prop_conversions_preserve_ordering(
        amount1 in 0i128..=(i64::MAX as i128),
        amount2 in 0i128..=(i64::MAX as i128),
        local_decimals in 0u32..=18u32,
        canonical_decimals in 0u32..=18u32,
    ) {
        let decimal_pair = TokenDecimalPair { local_decimals, canonical_decimals };

        // Skip if the conversions would overflow
        let canonical1 = to_canonical_amount(amount1, decimal_pair);
        let canonical2 = to_canonical_amount(amount2, decimal_pair);
        if canonical1.is_err() || canonical2.is_err() {
            return Ok(());
        }
        let canonical1 = canonical1.unwrap();
        let canonical2 = canonical2.unwrap();

        if amount1 < amount2 {
            prop_assert!(
                canonical1 <= canonical2,
                "to_canonical_amount should preserve ordering: {} < {} but {} > {}, decimals=({}, {})",
                amount1, amount2, canonical1, canonical2, local_decimals, canonical_decimals
            );
        } else if amount1 == amount2 {
            prop_assert_eq!(
                canonical1,
                canonical2,
                "Equal amounts should convert to equal canonical amounts"
            );
        }

        let local1 = to_local_amount(amount1, decimal_pair);
        let local2 = to_local_amount(amount2, decimal_pair);
        if local1.is_err() || local2.is_err() {
            return Ok(());
        }
        let local1 = local1.unwrap();
        let local2 = local2.unwrap();

        if amount1 < amount2 {
            prop_assert!(
                local1 <= local2,
                "to_local_amount should preserve ordering: {} < {} but {} > {}, decimals=({}, {})",
                amount1, amount2, local1, local2, local_decimals, canonical_decimals
            );
        } else if amount1 == amount2 {
            prop_assert_eq!(
                local1,
                local2,
                "Equal amounts should convert to equal local amounts"
            );
        }
    }

    /// Tests that when converting local→canonical→local (without normalization),
    /// the precision loss is bounded by 10^(decimal_difference) - 1.
    /// This only applies when local_decimals > canonical_decimals.
    #[test]
    fn prop_precision_loss_is_bounded(
        amount in 0i128..=(i64::MAX as i128),
        local_decimals in 0u32..=18u32,
        canonical_decimals in 0u32..=18u32,
    ) {
        // Only test when local > canonical (where precision loss occurs)
        if local_decimals <= canonical_decimals {
            return Ok(());
        }

        let decimal_pair = TokenDecimalPair { local_decimals, canonical_decimals };

        let canonical = to_canonical_amount(amount, decimal_pair);
        if canonical.is_err() {
            return Ok(());
        }
        let canonical = canonical.unwrap();

        let back_to_local = to_local_amount(canonical, decimal_pair);
        if back_to_local.is_err() {
            return Ok(());
        }
        let back_to_local = back_to_local.unwrap();

        let loss = amount - back_to_local;
        let max_loss = 10i128.pow(local_decimals - canonical_decimals) - 1;

        prop_assert!(
            loss >= 0 && loss <= max_loss,
            "Precision loss {} should be between 0 and {} (inclusive): \
             amount={}, canonical={}, back_to_local={}, decimals=({}, {})",
            loss, max_loss, amount, canonical, back_to_local, local_decimals, canonical_decimals
        );
    }

    /// Tests that after normalization, roundtrip conversion is lossless.
    #[test]
    fn prop_normalize_removes_all_dust(
        amount in 0i128..=(i64::MAX as i128),
        local_decimals in 0u32..=18u32,
        canonical_decimals in 0u32..=18u32,
    ) {
        let decimal_pair = TokenDecimalPair { local_decimals, canonical_decimals };

        let normalized = normalize_for_burn(amount, decimal_pair);
        if normalized.is_err() {
            return Ok(());
        }
        let normalized = normalized.unwrap();

        let canonical = to_canonical_amount(normalized, decimal_pair);
        if canonical.is_err() {
            return Ok(());
        }
        let canonical = canonical.unwrap();

        let back_to_local = to_local_amount(canonical, decimal_pair);
        if back_to_local.is_err() {
            return Ok(());
        }
        let back_to_local = back_to_local.unwrap();

        prop_assert_eq!(
            back_to_local,
            normalized,
            "Normalized amount should survive roundtrip without loss: \
             original={}, normalized={}, canonical={}, back_to_local={}, decimals=({}, {})",
            amount, normalized, canonical, back_to_local, local_decimals, canonical_decimals
        );
    }
}
