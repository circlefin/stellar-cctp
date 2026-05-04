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
use soroban_sdk::{contracttype, Address, Env};
use stellar_utils::storage::ttl::{
    get_and_extend_persistent_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use super::{emit_denylisted, emit_un_denylisted, DENYLISTER};

/// Storage key for denylist entries (per-address)
#[contracttype]
enum DenylistableStorageKey {
    Denylist(Address),
}

/// Adds an address to the denylist
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `account` - The address to add to the denylist.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the denylister
///   fails.
/// * `HostError: Error(Contract, #7000)` – Denylister is not set.
///
/// # Events
///
/// * topics - `["denylisted", account: Address]`
#[enforce_role_auth(DENYLISTER)]
pub fn denylist(e: &Env, account: &Address) {
    e.storage()
        .persistent()
        .set(&DenylistableStorageKey::Denylist(account.clone()), &true);
    emit_denylisted(e, account);
}

/// Removes an address from the denylist
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `account` - The address to remove from the denylist.
///
/// # Errors
///
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the denylister
///   fails.
/// * `HostError: Error(Contract, #7000)` – Denylister is not set.
///
/// # Events
///
/// * topics - `["un_denylisted", account: Address]`
#[enforce_role_auth(DENYLISTER)]
pub fn un_denylist(e: &Env, account: &Address) {
    e.storage()
        .persistent()
        .remove(&DenylistableStorageKey::Denylist(account.clone()));
    emit_un_denylisted(e, account);
}

/// Checks if an address is on the denylist
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `account` - The address to check.
///
/// # Returns
///
/// * `true` if the account is denylisted, `false` otherwise.
pub fn is_denylisted(e: &Env, account: &Address) -> bool {
    let key = DenylistableStorageKey::Denylist(account.clone());
    get_and_extend_persistent_ttl(e, &key, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT)
        .unwrap_or(false)
}

/// Requires that an address is not on the denylist
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `account` - The address to check.
///
/// # Errors
///
/// * [`DenylistError::AccountDenylisted`] – If the account is on the denylist.
pub fn require_not_denylisted(e: &Env, account: &Address) {
    if is_denylisted(e, account) {
        soroban_sdk::panic_with_error!(e, crate::denylistable::DenylistError::AccountDenylisted);
    }
}
