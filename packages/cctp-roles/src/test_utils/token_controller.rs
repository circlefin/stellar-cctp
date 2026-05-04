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

//! Test utilities for verifying TokenController contract implementations

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

/// Sets mock auth so that the owner can call `set_token_controller` on `contract_id`.
pub fn mock_set_token_controller_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    token_controller: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_token_controller",
            args: (token_controller,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `link_token_pair` on `contract_id`.
pub fn mock_link_token_pair_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "link_token_pair",
            args: (local_token, remote_domain, remote_token).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `unlink_token_pair` on `contract_id`.
pub fn mock_unlink_token_pair_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    local_token: &Address,
    remote_domain: u32,
    remote_token: &BytesN<32>,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "unlink_token_pair",
            args: (local_token, remote_domain, remote_token).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `set_max_burn_amount_per_message` on `contract_id`.
pub fn mock_set_max_burn_amount_per_message_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    local_token: &Address,
    burn_limit_per_message: i128,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_max_burn_amount_per_message",
            args: (local_token, burn_limit_per_message).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `set_token_decimal_config` on `contract_id`.
pub fn mock_set_token_decimal_config_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    token: &Address,
    local_decimals: u32,
    canonical_decimals: u32,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_token_decimal_config",
            args: (token, local_decimals, canonical_decimals).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `set_swap_minter_config` on `contract_id`.
pub fn mock_set_swap_minter_config_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    token: &Address,
    swap_minter: &Address,
    allow_asset: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_swap_minter_config",
            args: (token, swap_minter, allow_asset).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the token controller can call `remove_swap_minter_config` on `contract_id`.
pub fn mock_remove_swap_minter_config_auth(
    env: &Env,
    contract_id: &Address,
    token_controller: &Address,
    token: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: token_controller,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "remove_swap_minter_config",
            args: (token,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
