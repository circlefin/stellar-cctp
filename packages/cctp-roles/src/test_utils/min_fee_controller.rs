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

//! Test utilities for the MinFeeController role.

use soroban_sdk::{testutils::MockAuth, Address, Env, IntoVal};

/// Mocks authorization for setting the minimum fee controller.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `contract_id` - The contract address.
/// * `owner` - The owner address (must be authorized).
/// * `new_controller` - The new minimum fee controller address.
pub fn mock_set_min_fee_controller_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    new_controller: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_min_fee_controller",
            args: (new_controller.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks authorization for setting a minimum fee for a burn token.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `contract_id` - The contract address.
/// * `controller` - The minimum fee controller address (must be authorized).
/// * `burn_token` - The burn token whose fee is being set.
/// * `min_fee` - The minimum fee value.
pub fn mock_set_min_fee_auth(
    env: &Env,
    contract_id: &Address,
    controller: &Address,
    burn_token: &Address,
    min_fee: &i128,
) {
    env.mock_auths(&[MockAuth {
        address: controller,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_min_fee",
            args: (burn_token.clone(), min_fee).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
