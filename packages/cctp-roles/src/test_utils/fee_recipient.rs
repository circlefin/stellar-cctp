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

//! Test utilities for the FeeRecipient role.

use soroban_sdk::{testutils::MockAuth, Address, Env, IntoVal};

/// Mocks authorization for setting the fee recipient.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `contract_id` - The contract address.
/// * `owner` - The owner address (must be authorized).
/// * `new_fee_recipient` - The new fee recipient address.
pub fn mock_set_fee_recipient_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    new_fee_recipient: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_fee_recipient",
            args: (new_fee_recipient.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
