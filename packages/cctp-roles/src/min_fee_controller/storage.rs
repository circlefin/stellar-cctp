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
use simple_role_macros::enforce_role_auth;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use stellar_utils::storage::ttl::{
    get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use super::{emit_min_fee_set, MinFeeControllerError, MIN_FEE_CONTROLLER, MIN_FEE_MULTIPLIER};

#[contracttype]
pub enum MinFeeControllerStorageKey {
    MinFeeByBurnToken(Address),
}

/// Sets the minimum fee for a given burn token.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - The burn token (local token) whose minimum fee is being
///   configured.
/// * `min_fee` - The minimum fee in fractional units (must be non-negative and
///   less than [`MIN_FEE_MULTIPLIER`]).
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the minimum
///   fee controller fails.
/// * [`MinFeeControllerError::MinFeeControllerNotSet`] – If the controller is
///   not set.
/// * [`MinFeeControllerError::MinFeeNegative`] – If `min_fee` is negative.
/// * [`MinFeeControllerError::MinFeeTooHigh`] – If `min_fee` is greater than or
///   equal to [`MIN_FEE_MULTIPLIER`].
///
/// # Events
///
/// * topics - `["min_fee_set", burn_token: Address]`
/// * data - `[min_fee: i128]`
#[enforce_role_auth(MIN_FEE_CONTROLLER)]
pub fn set_min_fee(e: &Env, burn_token: &Address, min_fee: i128) {
    if min_fee < 0 {
        panic_with_error!(e, MinFeeControllerError::MinFeeNegative);
    }

    if min_fee >= MIN_FEE_MULTIPLIER {
        panic_with_error!(e, MinFeeControllerError::MinFeeTooHigh);
    }

    e.storage().persistent().set(
        &MinFeeControllerStorageKey::MinFeeByBurnToken(burn_token.clone()),
        &min_fee,
    );

    emit_min_fee_set(e, burn_token, min_fee);
}

/// Returns the configured minimum fee for a burn token, or zero if not set.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - The burn token (local token) to query.
pub fn get_min_fee(e: &Env, burn_token: &Address) -> i128 {
    let key = MinFeeControllerStorageKey::MinFeeByBurnToken(burn_token.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
        .unwrap_or_default()
}

/// Returns the minimum fee amount for a given burn token and amount.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `burn_token` - The burn token (local token) to query.
/// * `amount` - The transfer amount.
///
/// # Errors
///
/// * [`MinFeeControllerError::AmountTooLow`] – If `amount <= 1` when a non-zero
///   min fee is configured.
/// * [`MinFeeControllerError::MinFeeComputationOverflow`] – If the fee
///   computation overflows `i128`.
pub fn get_min_fee_amount(e: &Env, burn_token: &Address, amount: i128) -> i128 {
    let min_fee = get_min_fee(e, burn_token);
    if min_fee == 0 {
        return 0;
    }

    if amount <= 1 {
        panic_with_error!(e, MinFeeControllerError::AmountTooLow);
    }

    // Calculation: amount * min_fee / MIN_FEE_MULTIPLIER
    let product = amount
        .checked_mul(min_fee)
        .unwrap_or_else(|| panic_with_error!(e, MinFeeControllerError::MinFeeComputationOverflow));
    let min_fee_amount = product / MIN_FEE_MULTIPLIER;

    if min_fee_amount == 0 {
        1
    } else {
        min_fee_amount
    }
}
