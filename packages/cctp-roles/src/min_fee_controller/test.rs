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

use super::{MinFeeController, MIN_FEE_MULTIPLIER};
use crate::test_utils::min_fee_controller::{
    mock_set_min_fee_auth, mock_set_min_fee_controller_auth,
};
use crate::test_utils::CctpEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};
use stellar_access::ownable::set_owner;

use ::proptest::prelude::*;

#[contract]
struct TestContract;

#[contractimpl]
impl MinFeeController for TestContract {
    fn get_min_fee_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::MIN_FEE_CONTROLLER)
    }

    fn set_min_fee_controller(e: &Env, new_min_fee_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            super::MIN_FEE_CONTROLLER,
            &new_min_fee_controller,
            super::emit_min_fee_controller_set,
        );
    }

    fn set_min_fee(e: &Env, burn_token: Address, min_fee: i128) {
        super::set_min_fee(e, &burn_token, min_fee)
    }

    fn get_min_fee(e: &Env, burn_token: Address) -> i128 {
        super::get_min_fee(e, &burn_token)
    }

    fn get_min_fee_amount(e: &Env, burn_token: Address, amount: i128) -> i128 {
        super::get_min_fee_amount(e, &burn_token, amount)
    }
}

#[contractimpl]
impl TestContract {
    pub fn set_min_fee_ctrl_unchkd(env: Env, controller: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::MIN_FEE_CONTROLLER,
            &controller,
            super::emit_min_fee_controller_set,
        );
    }

    pub fn set_owner_unchecked(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }
}

// =============================================================================
// Controller Lifecycle Tests
// =============================================================================

#[test]
fn test_get_min_fee_controller_returns_none_initially() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_min_fee_controller(), None);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7000)")]
fn test_set_min_fee_fails_when_controller_not_set() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2100)")]
fn test_set_min_fee_controller_fails_when_owner_not_set() {
    let env = Env::default();
    let controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_min_fee_controller(&controller);
}

#[test]
fn test_set_min_fee_controller_sets_controller_and_emits_event() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller);
    assert_eq!(client.get_min_fee_controller(), Some(controller));
}

#[test]
fn test_set_min_fee_controller_can_be_updated() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller_1 = Address::generate(&env);
    let controller_2 = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller_1);
    client.set_min_fee_controller(&controller_1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller_1);
    assert_eq!(client.get_min_fee_controller(), Some(controller_1));

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller_2);
    client.set_min_fee_controller(&controller_2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller_2);
    assert_eq!(client.get_min_fee_controller(), Some(controller_2));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_controller_cannot_set_min_fee_after_transfer() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    let controller_1 = Address::generate(&env);
    client.set_min_fee_ctrl_unchkd(&controller_1);

    // Transfer to new controller
    let controller_2 = Address::generate(&env);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller_2);
    client.set_min_fee_controller(&controller_2);

    // Old controller tries to set min fee
    mock_set_min_fee_auth(&env, &contract_id, &controller_1, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);
}

#[test]
fn test_min_fee_persists_after_controller_transfer() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    let controller_1 = Address::generate(&env);
    client.set_min_fee_ctrl_unchkd(&controller_1);

    // Controller 1 sets a min fee
    let min_fee = MIN_FEE_MULTIPLIER / 100;
    mock_set_min_fee_auth(&env, &contract_id, &controller_1, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &min_fee);

    // Transfer controller
    let controller_2 = Address::generate(&env);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller_2);
    client.set_min_fee_controller(&controller_2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller_2);

    // Min fee still accessible
    assert_eq!(client.get_min_fee(&burn_token), min_fee);
}

#[test]
fn test_new_min_fee_controller_can_update_fee() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    let controller_1 = Address::generate(&env);
    client.set_min_fee_ctrl_unchkd(&controller_1);

    // Controller 1 sets a min fee
    let min_fee = MIN_FEE_MULTIPLIER / 100;
    mock_set_min_fee_auth(&env, &contract_id, &controller_1, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    // Transfer controller
    let controller_2 = Address::generate(&env);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller_2);
    client.set_min_fee_controller(&controller_2);

    // New controller can update fee
    let new_min_fee = MIN_FEE_MULTIPLIER / 50;
    mock_set_min_fee_auth(&env, &contract_id, &controller_2, &burn_token, &new_min_fee);
    client.set_min_fee(&burn_token, &new_min_fee);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &new_min_fee);
    assert_eq!(client.get_min_fee(&burn_token), new_min_fee);
}

#[test]
fn test_set_min_fee_controller_same_address_emits_event() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller);

    // Set same controller again - still emits event
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&controller);
    assert_eq!(client.get_min_fee_controller(), Some(controller));
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_min_fee_controller_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Don't mock auth - should fail
    client.set_min_fee_controller(&controller);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_owner_cannot_set_min_fee_controller() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Mock auth for non-owner
    mock_set_min_fee_controller_auth(&env, &contract_id, &non_owner, &controller);
    client.set_min_fee_controller(&controller);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_min_fee_requires_controller_auth() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&Address::generate(&env));
    client.set_min_fee_ctrl_unchkd(&controller);

    // Don't mock auth - should fail
    client.set_min_fee(&burn_token, &1);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_set_min_fee() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.set_min_fee_ctrl_unchkd(&controller);

    // Owner tries to set min fee (only controller can)
    mock_set_min_fee_auth(&env, &contract_id, &owner, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_controller_cannot_set_min_fee_controller() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let new_controller = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.set_min_fee_ctrl_unchkd(&controller);

    // Controller tries to set controller (only owner can)
    mock_set_min_fee_controller_auth(&env, &contract_id, &controller, &new_controller);
    client.set_min_fee_controller(&new_controller);
}

// =============================================================================
// Minimum Fee Configuration Tests
// =============================================================================

#[test]
fn test_set_min_fee_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let min_fee = MIN_FEE_MULTIPLIER / 100;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &min_fee);

    assert_eq!(client.get_min_fee(&burn_token), min_fee);
}

#[test]
fn test_set_min_fee_is_per_token() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token_a = Address::generate(&env);
    let burn_token_b = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let min_fee_a: i128 = 1;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token_a, &min_fee_a);
    client.set_min_fee(&burn_token_a, &min_fee_a);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token_a, &min_fee_a);

    let min_fee_b = MIN_FEE_MULTIPLIER / 10;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token_b, &min_fee_b);
    client.set_min_fee(&burn_token_b, &min_fee_b);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token_b, &min_fee_b);

    assert_eq!(client.get_min_fee(&burn_token_a), min_fee_a);
    assert_eq!(client.get_min_fee(&burn_token_b), min_fee_b);

    // Verify fee computation is isolated per token
    // token_a: 10_000 * 1 / 10_000_000 = 0 -> floor to 1
    assert_eq!(client.get_min_fee_amount(&burn_token_a, &10_000), 1);
    // token_b: 10_000 * 1_000_000 / 10_000_000 = 1_000
    assert_eq!(client.get_min_fee_amount(&burn_token_b, &10_000), 1_000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6201)")]
fn test_set_min_fee_rejects_min_fee_too_high() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    mock_set_min_fee_auth(
        &env,
        &contract_id,
        &controller,
        &burn_token,
        &MIN_FEE_MULTIPLIER,
    );
    client.set_min_fee(&burn_token, &MIN_FEE_MULTIPLIER);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6201)")]
fn test_set_min_fee_rejects_min_fee_above_multiplier() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let above_max = MIN_FEE_MULTIPLIER + 1;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &above_max);
    client.set_min_fee(&burn_token, &above_max);
}

#[test]
fn test_set_min_fee_same_value_emits_event() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&Address::generate(&env));
    client.set_min_fee_ctrl_unchkd(&controller);

    let min_fee = MIN_FEE_MULTIPLIER / 100;

    // Set min fee
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &min_fee);

    // Set same min fee again - still emits event
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &min_fee);
    assert_eq!(client.get_min_fee(&burn_token), min_fee);
}

#[test]
fn test_set_min_fee_can_be_updated() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&Address::generate(&env));
    client.set_min_fee_ctrl_unchkd(&controller);

    // Set initial fee
    let fee_1 = MIN_FEE_MULTIPLIER / 100;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &fee_1);
    client.set_min_fee(&burn_token, &fee_1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &fee_1);
    assert_eq!(client.get_min_fee(&burn_token), fee_1);

    // Update to different fee
    let fee_2 = MIN_FEE_MULTIPLIER / 10;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &fee_2);
    client.set_min_fee(&burn_token, &fee_2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &fee_2);
    assert_eq!(client.get_min_fee(&burn_token), fee_2);
}

// =============================================================================
// Minimum Fee Calculation Tests
// =============================================================================

#[test]
fn test_get_min_fee_returns_zero_when_not_set() {
    let env = Env::default();
    let burn_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_min_fee(&burn_token), 0);
}

#[test]
fn test_get_min_fee_amount_returns_zero_when_fee_not_set() {
    let env = Env::default();
    let burn_token = Address::generate(&env);
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // No controller / fee set -> should return 0
    assert_eq!(client.get_min_fee_amount(&burn_token, &1_000), 0);
}

#[test]
fn test_get_min_fee_amount_returns_zero_when_fee_is_zero() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &0);
    client.set_min_fee(&burn_token, &0);

    assert_eq!(client.get_min_fee_amount(&burn_token, &500), 0);
    assert_eq!(client.get_min_fee_amount(&burn_token, &1), 0);
}

#[test]
fn test_get_min_fee_amount_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let min_fee = MIN_FEE_MULTIPLIER / 100;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    // amount * min_fee / MIN_FEE_MULTIPLIER = 10_000 * 100_000 / 10_000_000 = 100
    assert_eq!(client.get_min_fee_amount(&burn_token, &10_000), 100);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6202)")]
fn test_get_min_fee_amount_rejects_amount_too_low() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);

    // amount <= 1 with non-zero fee should panic
    client.get_min_fee_amount(&burn_token, &1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6202)")]
fn test_get_min_fee_amount_rejects_zero_amount() {
    let env = Env::default();
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&Address::generate(&env));
    client.set_min_fee_ctrl_unchkd(&controller);

    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);

    // amount = 0 with non-zero fee should panic
    client.get_min_fee_amount(&burn_token, &0);
}

#[test]
fn test_get_min_fee_amount_rounds_up_minimum_one() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    // Very small fee -> fractional result
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &1);
    client.set_min_fee(&burn_token, &1);

    assert_eq!(client.get_min_fee_amount(&burn_token, &2), 1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6203)")]
fn test_get_min_fee_amount_overflow_panics() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    let min_fee = MIN_FEE_MULTIPLIER - 1;
    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
    client.set_min_fee(&burn_token, &min_fee);

    // Craft an amount that will overflow i128 when multiplied by the fee
    let huge_amount = i128::MAX / 2;
    client.get_min_fee_amount(&burn_token, &huge_amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6204)")]
fn test_set_min_fee_rejects_negative_fee() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let controller = Address::generate(&env);
    let burn_token = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
    client.set_min_fee_controller(&controller);

    mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &-1);
    client.set_min_fee(&burn_token, &-1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_set_min_fee_accepts_valid_range(min_fee in 0i128..MIN_FEE_MULTIPLIER) {
        let env = Env::default();
        let owner = Address::generate(&env);
        let controller = Address::generate(&env);
        let burn_token = Address::generate(&env);

        let contract_id = env.register(TestContract, ());
        let client = TestContractClient::new(&env, &contract_id);

        client.set_owner_unchecked(&owner);

        mock_set_min_fee_controller_auth(&env, &contract_id, &owner, &controller);
        client.set_min_fee_controller(&controller);

        mock_set_min_fee_auth(&env, &contract_id, &controller, &burn_token, &min_fee);
        client.set_min_fee(&burn_token, &min_fee);

        prop_assert_eq!(client.get_min_fee(&burn_token), min_fee);
    }
}
