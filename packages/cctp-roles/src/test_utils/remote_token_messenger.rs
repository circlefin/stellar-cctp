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

//! Test utilities for verifying RemoteTokenMessenger contract implementations

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

/// Sets mock auth so that the owner can call `add_remote_token_messenger` on `contract_id`.
pub fn mock_add_remote_token_messenger_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    domain: u32,
    token_messenger: &BytesN<32>,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "add_remote_token_messenger",
            args: (domain, token_messenger).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the owner can call `remove_remote_token_messenger` on `contract_id`.
pub fn mock_remove_remote_token_messenger_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    domain: u32,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "remove_remote_token_messenger",
            args: (domain,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
