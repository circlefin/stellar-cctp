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

//! Test utilities for verifying Denylistable contract implementations
//!
//! This module provides assertion macros to verify that a contract correctly
//! implements the Denylistable role, including:
//! - Setting and updating the denylister
//! - Adding addresses to the denylist
//! - Removing addresses from the denylist
//! - Authorization checks (only denylister can manage denylist)
//! - Ownership checks (only owner can update denylister)

/// Asserts that a contract correctly implements the Denylistable interface.
///
/// # Arguments
/// * `$env` - The test environment
/// * `$client` - Client for the contract being tested (must have Denylistable methods)
/// * `$contract_id` - Address of the contract being tested
/// * `$owner` - Address of the contract owner
#[macro_export]
macro_rules! assert_contract_is_denylistable {
    ($env:expr, $client:expr, $contract_id:expr, $owner:expr) => {{
        extern crate std;
        use soroban_sdk::{testutils::Address as _, Address};

        let env = $env;
        let contract_client = $client;
        let contract_id = $contract_id;
        let owner = $owner;

        let denylister = Address::generate(env);
        let account1 = Address::generate(env);
        let account2 = Address::generate(env);

        // Set denylister and verify it can be retrieved
        $crate::test_utils::denylistable::mock_update_denylister_auth(
            env,
            &contract_id,
            &owner,
            &denylister,
        );
        contract_client.update_denylister(&denylister);
        let stored_denylister = contract_client.get_denylister();
        assert_eq!(
            Some(denylister.clone()),
            stored_denylister,
            "Denylister should be set and retrievable"
        );

        // Account should not be denylisted initially
        assert!(
            !contract_client.is_denylisted(&account1),
            "Account should not be denylisted initially"
        );

        // Successfully denylist an address
        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account1,
        );
        contract_client.denylist(&account1);

        assert!(
            contract_client.is_denylisted(&account1),
            "Account should be denylisted after denylist() call"
        );

        // Successfully un-denylist an address
        $crate::test_utils::denylistable::mock_un_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account1,
        );
        contract_client.un_denylist(&account1);

        assert!(
            !contract_client.is_denylisted(&account1),
            "Account should not be denylisted after un_denylist() call"
        );

        // Can manage multiple addresses independently
        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account1,
        );
        contract_client.denylist(&account1);

        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account2,
        );
        contract_client.denylist(&account2);

        assert!(
            contract_client.is_denylisted(&account1),
            "Account1 should be denylisted"
        );
        assert!(
            contract_client.is_denylisted(&account2),
            "Account2 should be denylisted"
        );

        // Un-denylist one address should not affect the other
        $crate::test_utils::denylistable::mock_un_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account1,
        );
        contract_client.un_denylist(&account1);

        assert!(
            !contract_client.is_denylisted(&account1),
            "Account1 should not be denylisted"
        );
        assert!(
            contract_client.is_denylisted(&account2),
            "Account2 should still be denylisted"
        );

        // Non-denylister cannot denylist (should panic)
        let non_denylister = Address::generate(env);
        let account3 = Address::generate(env);
        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &non_denylister,
            &account3,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.denylist(&account3);
        }));

        assert!(
            result.is_err(),
            "Non-denylister should not be able to denylist addresses"
        );

        // Non-denylister cannot un-denylist (should panic)
        $crate::test_utils::denylistable::mock_un_denylist_auth(
            env,
            &contract_id,
            &non_denylister,
            &account2,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.un_denylist(&account2);
        }));

        assert!(
            result.is_err(),
            "Non-denylister should not be able to un-denylist addresses"
        );

        // Non-owner cannot update denylister (should panic)
        let non_owner = Address::generate(env);
        let new_denylister = Address::generate(env);
        $crate::test_utils::denylistable::mock_update_denylister_auth(
            env,
            &contract_id,
            &non_owner,
            &new_denylister,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.update_denylister(&new_denylister);
        }));

        assert!(
            result.is_err(),
            "Non-owner should not be able to update denylister"
        );

        // Owner can update denylister
        let new_denylister = Address::generate(env);
        $crate::test_utils::denylistable::mock_update_denylister_auth(
            env,
            &contract_id,
            &owner,
            &new_denylister,
        );
        contract_client.update_denylister(&new_denylister);

        let stored_denylister = contract_client.get_denylister();
        assert_eq!(
            Some(new_denylister.clone()),
            stored_denylister,
            "Denylister should be updated to new address"
        );

        // Verify new denylister can manage denylist
        let account4 = Address::generate(env);
        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &new_denylister,
            &account4,
        );
        contract_client.denylist(&account4);

        assert!(
            contract_client.is_denylisted(&account4),
            "New denylister should be able to denylist addresses"
        );

        // Old denylister can no longer manage denylist (should panic)
        let account5 = Address::generate(env);
        $crate::test_utils::denylistable::mock_denylist_auth(
            env,
            &contract_id,
            &denylister,
            &account5,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract_client.denylist(&account5);
        }));

        assert!(
            result.is_err(),
            "Old denylister should not be able to denylist addresses after being replaced"
        );
    }};
}

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

/// Sets mock auth so that the owner can call `update_denylister` on `contract`.
pub fn mock_update_denylister_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    denylister: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_denylister",
            args: (denylister,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the denylister can call `denylist` on `contract`.
pub fn mock_denylist_auth(
    env: &Env,
    contract_id: &Address,
    denylister: &Address,
    account: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: denylister,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "denylist",
            args: (account,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the denylister can call `un_denylist` on `contract`.
pub fn mock_un_denylist_auth(
    env: &Env,
    contract_id: &Address,
    denylister: &Address,
    account: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: denylister,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "un_denylist",
            args: (account,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
