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
extern crate std;

use crate::test_utils::token_controller::{
    mock_link_token_pair_auth, mock_remove_swap_minter_config_auth,
    mock_set_max_burn_amount_per_message_auth, mock_set_swap_minter_config_auth,
    mock_set_token_controller_auth, mock_set_token_decimal_config_auth,
    mock_unlink_token_pair_auth,
};
use crate::test_utils::CctpEventAssertions;
use ::proptest::prelude::*;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::BytesN as _, Address, BytesN, Env,
};

use super::{SwapMinterConfig, TokenController, TokenDecimalConfig};

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn set_owner_unchecked(env: Env, owner: Address) {
        common_roles::ownable::set_owner_unchecked(&env, &owner);
    }

    pub fn set_token_controller_unchecked(env: Env, token_controller: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::TOKEN_CONTROLLER,
            &token_controller,
            super::emit_set_token_controller,
        );
    }

    pub fn enforce_within_burn_limit(env: Env, burn_token: Address, amount: i128) {
        super::enforce_within_burn_limit(&env, &burn_token, amount);
    }
}

#[contractimpl]
impl TokenController for TestContract {
    fn get_token_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::TOKEN_CONTROLLER)
    }

    fn set_token_controller(e: &Env, new_token_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            super::TOKEN_CONTROLLER,
            &new_token_controller,
            super::emit_set_token_controller,
        );
    }

    fn link_token_pair(
        e: &Env,
        local_token: Address,
        remote_domain: u32,
        remote_token: BytesN<32>,
    ) {
        super::link_token_pair(e, &local_token, remote_domain, &remote_token);
    }

    fn unlink_token_pair(
        e: &Env,
        local_token: Address,
        remote_domain: u32,
        remote_token: BytesN<32>,
    ) {
        super::unlink_token_pair(e, &local_token, remote_domain, &remote_token);
    }

    fn set_max_burn_amount_per_message(
        e: &Env,
        local_token: Address,
        burn_limit_per_message: i128,
    ) {
        super::set_max_burn_amount_per_message(e, &local_token, burn_limit_per_message);
    }

    fn get_max_burn_amount_per_message(e: &Env, local_token: Address) -> Option<i128> {
        super::get_max_burn_amount_per_message(e, &local_token)
    }

    fn get_local_token(e: &Env, remote_domain: u32, remote_token: BytesN<32>) -> Option<Address> {
        super::get_local_token(e, remote_domain, &remote_token)
    }

    fn get_token_decimal_config(e: &Env, token: Address) -> Option<TokenDecimalConfig> {
        super::get_token_decimal_config(e, &token)
    }

    fn set_token_decimal_config(
        e: &Env,
        token: Address,
        local_decimals: u32,
        canonical_decimals: u32,
    ) {
        super::set_token_decimal_config(e, &token, local_decimals, canonical_decimals);
    }

    fn get_swap_minter_config(e: &Env, token: Address) -> Option<SwapMinterConfig> {
        super::get_swap_minter_config(e, &token)
    }

    fn set_swap_minter_config(e: &Env, token: Address, swap_minter: Address, allow_asset: Address) {
        super::set_swap_minter_config(e, &token, &swap_minter, &allow_asset);
    }

    fn remove_swap_minter_config(e: &Env, token: Address) {
        super::remove_swap_minter_config(e, &token);
    }
}

fn setup_controller(
    env: &Env,
    contract_id: &Address,
    client: &TestContractClient,
) -> (Address, Address) {
    let owner = Address::generate(env);
    client.set_owner_unchecked(&owner);
    let controller = Address::generate(env);
    mock_set_token_controller_auth(env, contract_id, &owner, &controller);
    client.set_token_controller(&controller);
    (controller, owner)
}

// =============================================================================
// Token Controller Role Lifecycle Tests
// =============================================================================

#[test]
fn test_get_token_controller_returns_none_initially() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_token_controller(), None);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2100)")]
fn test_set_token_controller_fails_when_owner_not_set() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_token_controller(&controller);
}

#[test]
fn test_set_token_controller_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_token_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_token_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_token_controller(&controller);

    assert_eq!(client.get_token_controller(), Some(controller));
}

#[test]
fn test_set_token_controller_can_be_updated() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller_1 = Address::generate(&env);
    let controller_2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_token_controller_auth(&env, &contract_id, &owner, &controller_1);
    client.set_token_controller(&controller_1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_set_token_controller(&controller_1);
    assert_eq!(client.get_token_controller(), Some(controller_1));

    mock_set_token_controller_auth(&env, &contract_id, &owner, &controller_2);
    client.set_token_controller(&controller_2);

    // An event is still emitted for updating to the same controller
    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_token_controller(&controller_2);
    assert_eq!(client.get_token_controller(), Some(controller_2));
}

#[test]
fn test_set_token_controller_unchecked_success() {
    let env = Env::default();
    let token_controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_token_controller_unchecked(&token_controller);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_token_controller(&token_controller);

    assert_eq!(
        client.get_token_controller(),
        Some(token_controller.clone())
    );
}

// =============================================================================
// Token Controller Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_token_controller_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // No mock auth — should fail
    client.set_token_controller(&controller);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_owner_cannot_set_token_controller() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_token_controller_auth(&env, &contract_id, &non_owner, &controller);
    client.set_token_controller(&controller);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_controller_cannot_set_token_controller() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, _) = setup_controller(&env, &contract_id, &client);

    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &controller, &new_controller);
    client.set_token_controller(&new_controller);
}

// =============================================================================
// Link Token Pair Tests
// =============================================================================

#[test]
fn test_link_token_pair_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let remote_domain = 1u32;
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token, remote_domain, &remote_token);

    assert_eq!(
        client.get_local_token(&remote_domain, &remote_token),
        Some(local_token)
    );
}

#[test]
#[should_panic(expected = "#6300")]
fn test_link_token_pair_fails_if_already_linked() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "#6300")]
fn test_link_token_pair_fails_if_different_local_on_same_pair() {
    let env = Env::default();
    let local_token_1 = Address::generate(&env);
    let local_token_2 = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token_1,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token_1, &1, &remote_token);

    // Same domain + remote_token but different local — still fails (already linked)
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token_2,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token_2, &1, &remote_token);
}

#[test]
fn test_link_token_pair_multiple_domains_same_remote_token() {
    let env = Env::default();
    let local_token1 = Address::generate(&env);
    let local_token2 = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token1,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token1, &1, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token1, 1, &remote_token);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token2,
        2,
        &remote_token,
    );
    client.link_token_pair(&local_token2, &2, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token2, 2, &remote_token);

    assert_eq!(
        client.get_local_token(&1, &remote_token),
        Some(local_token1)
    );
    assert_eq!(
        client.get_local_token(&2, &remote_token),
        Some(local_token2)
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_link_token_pair_fails_when_role_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    client.link_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_link_token_pair_requires_controller_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    // No mock auth — should fail
    client.link_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_link_token_pair() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (_, owner) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    mock_link_token_pair_auth(&env, &contract_id, &owner, &local_token, 1, &remote_token);
    client.link_token_pair(&local_token, &1, &remote_token);
}

#[test]
fn test_link_persists_after_controller_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, owner) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_domain = 1u32;
    let remote_token = BytesN::<32>::random(&env);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token, remote_domain, &remote_token);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Link persists
    assert_eq!(
        client.get_local_token(&remote_domain, &remote_token),
        Some(local_token)
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_link_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Old controller tries to link — should fail
    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);
}

// =============================================================================
// Unlink Token Pair Tests
// =============================================================================

#[test]
fn test_unlink_token_pair_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let remote_domain = 1u32;
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    mock_unlink_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.unlink_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_token_pair_unlinked(&local_token, remote_domain, &remote_token);

    assert_eq!(client.get_local_token(&remote_domain, &remote_token), None);
}

#[test]
#[should_panic(expected = "#6301")]
fn test_unlink_token_pair_fails_if_not_linked() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_unlink_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1,
        &remote_token,
    );
    client.unlink_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "#6309")]
fn test_unlink_token_pair_fails_if_local_token_mismatch() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let wrong_local_token = Address::generate(&env);
    let remote_domain = 1u32;
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    mock_unlink_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &wrong_local_token,
        remote_domain,
        &remote_token,
    );
    client.unlink_token_pair(&wrong_local_token, &remote_domain, &remote_token);
}

#[test]
fn test_can_relink_after_unlinking() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let remote_domain = 1u32;
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    // Link
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token, remote_domain, &remote_token);

    assert_eq!(
        client.get_local_token(&remote_domain, &remote_token),
        Some(local_token.clone())
    );

    // Unlink
    mock_unlink_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.unlink_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_unlinked(&local_token, remote_domain, &remote_token);

    assert_eq!(client.get_local_token(&remote_domain, &remote_token), None);

    // Re-link
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token, remote_domain, &remote_token);

    assert_eq!(
        client.get_local_token(&remote_domain, &remote_token),
        Some(local_token)
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_unlink_token_pair_fails_when_role_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    client.unlink_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unlink_token_pair_requires_controller_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, _) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);

    // No mock auth for unlink — should fail
    client.unlink_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_unlink_token_pair() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, owner) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);

    mock_unlink_token_pair_auth(&env, &contract_id, &owner, &local_token, 1, &remote_token);
    client.unlink_token_pair(&local_token, &1, &remote_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_unlink_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let remote_token = BytesN::<32>::random(&env);
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        1,
        &remote_token,
    );
    client.link_token_pair(&local_token, &1, &remote_token);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Old controller tries to unlink — should fail
    mock_unlink_token_pair_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        1,
        &remote_token,
    );
    client.unlink_token_pair(&local_token, &1, &remote_token);
}

// =============================================================================
// Get Local Token Tests
// =============================================================================

#[test]
fn test_get_local_token_returns_none_when_not_mapped() {
    let env = Env::default();
    let remote_token = BytesN::<32>::random(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_local_token(&1, &remote_token), None);
}

// =============================================================================
// Burn Limit Tests
// =============================================================================

#[test]
fn test_set_max_burn_amount_per_message_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    let burn_limit = 1_000_000i128;
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        burn_limit,
    );
    client.set_max_burn_amount_per_message(&local_token, &burn_limit);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_burn_limit_per_message(&local_token, burn_limit);

    assert_eq!(
        client.get_max_burn_amount_per_message(&local_token),
        Some(burn_limit)
    );
}

#[test]
fn test_get_max_burn_amount_per_message_returns_none_when_not_set() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_max_burn_amount_per_message(&local_token), None);
}

#[test]
fn test_set_max_burn_amount_can_update() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        1_000_000,
    );
    client.set_max_burn_amount_per_message(&local_token, &1_000_000);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_set_burn_limit_per_message(&local_token, 1_000_000);

    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        2_000_000,
    );
    client.set_max_burn_amount_per_message(&local_token, &2_000_000);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_burn_limit_per_message(&local_token, 2_000_000);

    assert_eq!(
        client.get_max_burn_amount_per_message(&local_token),
        Some(2_000_000)
    );
}

#[test]
#[should_panic(expected = "#6306")]
fn test_set_max_burn_amount_fails_with_negative_value() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &controller, &local_token, -1);
    client.set_max_burn_amount_per_message(&local_token, &-1);
}

#[test]
fn test_set_max_burn_amount_accepts_zero() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &controller, &local_token, 0);
    client.set_max_burn_amount_per_message(&local_token, &0);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_set_burn_limit_per_message(&local_token, 0);

    assert_eq!(
        client.get_max_burn_amount_per_message(&local_token),
        Some(0)
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_set_max_burn_amount_fails_when_role_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_max_burn_amount_per_message(&local_token, &1_000_000);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_max_burn_amount_requires_controller_auth() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    setup_controller(&env, &contract_id, &client);

    // No mock auth — should fail
    client.set_max_burn_amount_per_message(&local_token, &1_000_000);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_set_max_burn_amount() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (_, owner) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &owner, &local_token, 1_000_000);
    client.set_max_burn_amount_per_message(&local_token, &1_000_000);
}

#[test]
fn test_burn_limit_persists_after_controller_transfer() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, owner) = setup_controller(&env, &contract_id, &client);

    let burn_limit = 1_000_000i128;
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        burn_limit,
    );
    client.set_max_burn_amount_per_message(&local_token, &burn_limit);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Burn limit persists
    assert_eq!(
        client.get_max_burn_amount_per_message(&local_token),
        Some(burn_limit)
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_set_burn_limit_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    let local_token = Address::generate(&env);
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        1_000_000,
    );
    client.set_max_burn_amount_per_message(&local_token, &1_000_000);
}

// =============================================================================
// Enforce Within Burn Limit Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_enforce_within_burn_limit_succeeds_for_valid_amounts(amount in 1i128..=1_000_000i128) {
        let env = Env::default();
        let burn_token = Address::generate(&env);
        let contract_id = env.register(TestContract, ());
        let client = TestContractClient::new(&env, &contract_id);

        let (controller, _) = setup_controller(&env, &contract_id, &client);

        mock_set_max_burn_amount_per_message_auth(
            &env,
            &contract_id,
            &controller,
            &burn_token,
            1_000_000,
        );
        client.set_max_burn_amount_per_message(&burn_token, &1_000_000);

        // Should not panic for any amount within the limit (including the exact limit)
        client.enforce_within_burn_limit(&burn_token, &amount);
    }
}

#[test]
#[should_panic(expected = "#6304")]
fn test_enforce_within_burn_limit_fails_when_exceeded() {
    let env = Env::default();
    let burn_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &controller,
        &burn_token,
        1_000_000,
    );
    client.set_max_burn_amount_per_message(&burn_token, &1_000_000);

    client.enforce_within_burn_limit(&burn_token, &1_000_001);
}

#[test]
#[should_panic(expected = "#6303")]
fn test_enforce_within_burn_limit_fails_when_not_set() {
    let env = Env::default();
    let burn_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.enforce_within_burn_limit(&burn_token, &100);
}

#[test]
#[should_panic(expected = "#6303")]
fn test_enforce_within_burn_limit_fails_when_limit_is_zero() {
    let env = Env::default();
    let burn_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &controller, &burn_token, 0);
    client.set_max_burn_amount_per_message(&burn_token, &0);

    client.enforce_within_burn_limit(&burn_token, &100);
}

#[test]
fn test_burn_limit_is_per_token() {
    let env = Env::default();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &controller, &token_a, 1_000_000);
    client.set_max_burn_amount_per_message(&token_a, &1_000_000);

    mock_set_max_burn_amount_per_message_auth(&env, &contract_id, &controller, &token_b, 5_000_000);
    client.set_max_burn_amount_per_message(&token_b, &5_000_000);

    assert_eq!(
        client.get_max_burn_amount_per_message(&token_a),
        Some(1_000_000)
    );
    assert_eq!(
        client.get_max_burn_amount_per_message(&token_b),
        Some(5_000_000)
    );

    // Enforce is per-token: token_a rejects above its own limit
    client.enforce_within_burn_limit(&token_a, &1_000_000);
    client.enforce_within_burn_limit(&token_b, &5_000_000);
}

// =============================================================================
// Token Decimal Config Tests
// =============================================================================

#[test]
fn test_set_token_decimal_config_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &local_token, 7, 6);
    client.set_token_decimal_config(&local_token, &7, &6);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_token_decimal_config_added(&local_token, 7, 6);

    let config = client.get_token_decimal_config(&local_token).unwrap();
    assert_eq!(config.local_decimals, 7);
    assert_eq!(config.canonical_decimals, 6);
}

#[test]
fn test_get_token_decimal_config_returns_none_when_not_set() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_token_decimal_config(&local_token), None);
}

#[test]
#[should_panic(expected = "#6308")] // TokenDecimalConfigAlreadySet
fn test_set_token_decimal_config_fails_if_already_set() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &local_token, 7, 6);
    client.set_token_decimal_config(&local_token, &7, &6);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_decimal_config_added(&local_token, 7, 6);

    // Setting again on the same token should fail
    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &local_token, 18, 6);
    client.set_token_decimal_config(&local_token, &18, &6);
}

#[test]
#[should_panic(expected = "#6307")] // InvalidDecimalScale
fn test_set_token_decimal_config_fails_if_local_less_than_canonical() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_token_controller_unchecked(&controller);

    // local=6, canonical=7 is invalid: local_decimals must be >= canonical_decimals
    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &local_token, 6, 7);
    client.set_token_decimal_config(&local_token, &6, &7);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_set_token_decimal_config_fails_when_role_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_token_decimal_config(&local_token, &7, &6);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_token_decimal_config_requires_controller_auth() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    setup_controller(&env, &contract_id, &client);

    // No mock auth — should fail
    client.set_token_decimal_config(&local_token, &7, &6);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_set_token_decimal_config() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (_, owner) = setup_controller(&env, &contract_id, &client);

    mock_set_token_decimal_config_auth(&env, &contract_id, &owner, &local_token, 7, 6);
    client.set_token_decimal_config(&local_token, &7, &6);
}

#[test]
fn test_decimal_config_persists_after_controller_transfer() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (controller, owner) = setup_controller(&env, &contract_id, &client);

    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &local_token, 7, 6);
    client.set_token_decimal_config(&local_token, &7, &6);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Config persists
    let config = client.get_token_decimal_config(&local_token).unwrap();
    assert_eq!(config.local_decimals, 7);
    assert_eq!(config.canonical_decimals, 6);
}

#[test]
fn test_decimal_config_is_per_token() {
    let env = Env::default();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &token_a, 7, 6);
    client.set_token_decimal_config(&token_a, &7, &6);

    mock_set_token_decimal_config_auth(&env, &contract_id, &controller, &token_b, 18, 8);
    client.set_token_decimal_config(&token_b, &18, &8);

    let config_a = client.get_token_decimal_config(&token_a).unwrap();
    assert_eq!(config_a.local_decimals, 7);
    assert_eq!(config_a.canonical_decimals, 6);

    let config_b = client.get_token_decimal_config(&token_b).unwrap();
    assert_eq!(config_b.local_decimals, 18);
    assert_eq!(config_b.canonical_decimals, 8);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_set_decimal_config_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    let local_token = Address::generate(&env);
    mock_set_token_decimal_config_auth(&env, &contract_id, &old_controller, &local_token, 7, 6);
    client.set_token_decimal_config(&local_token, &7, &6);
}

// =============================================================================
// Swap Minter Config Tests
// =============================================================================

#[test]
fn test_get_swap_minter_config_returns_none_when_not_set() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_swap_minter_config(&local_token), None);
}

#[test]
fn test_set_swap_minter_config_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_swap_minter_config_set(&local_token, &swap_minter, &allow_asset);

    let config = client.get_swap_minter_config(&local_token).unwrap();
    assert_eq!(config.swap_minter, swap_minter);
    assert_eq!(config.allow_asset, allow_asset);
}

#[test]
fn test_set_swap_minter_config_can_update() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let swap_minter1 = Address::generate(&env);
    let allow_asset1 = Address::generate(&env);
    let swap_minter2 = Address::generate(&env);
    let allow_asset2 = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter1,
        &allow_asset1,
    );
    client.set_swap_minter_config(&local_token, &swap_minter1, &allow_asset1);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter2,
        &allow_asset2,
    );
    client.set_swap_minter_config(&local_token, &swap_minter2, &allow_asset2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_swap_minter_config_set(&local_token, &swap_minter2, &allow_asset2);

    let config = client.get_swap_minter_config(&local_token).unwrap();
    assert_eq!(config.swap_minter, swap_minter2);
    assert_eq!(config.allow_asset, allow_asset2);
}

#[test]
fn test_remove_swap_minter_config_success() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &controller, &local_token);
    client.remove_swap_minter_config(&local_token);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_swap_minter_config_removed(&local_token, &swap_minter, &allow_asset);

    assert_eq!(client.get_swap_minter_config(&local_token), None);
}

#[test]
fn test_swap_minter_config_is_per_token() {
    let env = Env::default();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let swap_minter_a = Address::generate(&env);
    let allow_asset_a = Address::generate(&env);
    let swap_minter_b = Address::generate(&env);
    let allow_asset_b = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &token_a,
        &swap_minter_a,
        &allow_asset_a,
    );
    client.set_swap_minter_config(&token_a, &swap_minter_a, &allow_asset_a);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &token_b,
        &swap_minter_b,
        &allow_asset_b,
    );
    client.set_swap_minter_config(&token_b, &swap_minter_b, &allow_asset_b);

    let config_a = client.get_swap_minter_config(&token_a).unwrap();
    assert_eq!(config_a.swap_minter, swap_minter_a);
    assert_eq!(config_a.allow_asset, allow_asset_a);

    let config_b = client.get_swap_minter_config(&token_b).unwrap();
    assert_eq!(config_b.swap_minter, swap_minter_b);
    assert_eq!(config_b.allow_asset, allow_asset_b);
}

#[test]
fn test_swap_minter_config_persists_after_controller_transfer() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, owner) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    // Config persists
    let config = client.get_swap_minter_config(&local_token).unwrap();
    assert_eq!(config.swap_minter, swap_minter);
    assert_eq!(config.allow_asset, allow_asset);
}

#[test]
fn test_can_re_add_swap_minter_config_after_removal() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &controller, &local_token);
    client.remove_swap_minter_config(&local_token);
    assert_eq!(client.get_swap_minter_config(&local_token), None);

    // Re-add
    let swap_minter_2 = Address::generate(&env);
    let allow_asset_2 = Address::generate(&env);
    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter_2,
        &allow_asset_2,
    );
    client.set_swap_minter_config(&local_token, &swap_minter_2, &allow_asset_2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_swap_minter_config_set(&local_token, &swap_minter_2, &allow_asset_2);

    let config = client.get_swap_minter_config(&local_token).unwrap();
    assert_eq!(config.swap_minter, swap_minter_2);
    assert_eq!(config.allow_asset, allow_asset_2);
}

#[test]
#[should_panic(expected = "#6305")]
fn test_remove_swap_minter_config_fails_if_not_set() {
    let env = Env::default();
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &controller, &local_token);
    client.remove_swap_minter_config(&local_token);
}

#[test]
#[should_panic(expected = "#7000")]
fn test_set_swap_minter_config_fails_if_role_not_set() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);
}

#[test]
#[should_panic(expected = "#7000")]
fn test_remove_swap_minter_config_fails_if_role_not_set() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let local_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &controller, &local_token);
    client.remove_swap_minter_config(&local_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_swap_minter_config_fails_with_wrong_controller() {
    let env = Env::default();
    let wrong_controller = Address::generate(&env);
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &wrong_controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_remove_swap_minter_config_fails_with_wrong_controller() {
    let env = Env::default();
    let wrong_controller = Address::generate(&env);
    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (controller, _) = setup_controller(&env, &contract_id, &client);

    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &wrong_controller, &local_token);
    client.remove_swap_minter_config(&local_token);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_set_swap_minter_config_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_remove_swap_minter_config_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_controller, owner) = setup_controller(&env, &contract_id, &client);

    let local_token = Address::generate(&env);
    let swap_minter = Address::generate(&env);
    let allow_asset = Address::generate(&env);
    mock_set_swap_minter_config_auth(
        &env,
        &contract_id,
        &old_controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    // Transfer controller
    let new_controller = Address::generate(&env);
    mock_set_token_controller_auth(&env, &contract_id, &owner, &new_controller);
    client.set_token_controller(&new_controller);

    mock_remove_swap_minter_config_auth(&env, &contract_id, &old_controller, &local_token);
    client.remove_swap_minter_config(&local_token);
}
