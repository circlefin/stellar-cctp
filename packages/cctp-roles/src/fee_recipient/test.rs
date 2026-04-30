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

use super::FeeRecipient;
use crate::test_utils::fee_recipient::mock_set_fee_recipient_auth;
use crate::test_utils::CctpEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};
use stellar_access::ownable::set_owner;

#[contract]
struct TestContract;

#[contractimpl]
impl FeeRecipient for TestContract {
    fn get_fee_recipient(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::FEE_RECIPIENT)
    }

    fn set_fee_recipient(e: &Env, new_fee_recipient: Address) {
        simple_role::set_role_and_emit(
            e,
            super::FEE_RECIPIENT,
            &new_fee_recipient,
            super::emit_fee_recipient_set,
        );
    }
}

#[contractimpl]
impl TestContract {
    pub fn set_fee_recipient_unchkd(env: Env, fee_recipient: Address) {
        simple_role::set_role_and_emit_unchecked(
            &env,
            super::FEE_RECIPIENT,
            &fee_recipient,
            super::emit_fee_recipient_set,
        );
    }

    pub fn set_owner_unchecked(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }
}

// =============================================================================
// Fee Recipient Management Tests
// =============================================================================

#[test]
fn test_get_fee_recipient_returns_none_initially() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_fee_recipient(), None);
}

#[test]
fn test_set_fee_recipient_unchecked_success() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let fee_recipient = Address::generate(&env);

    client.set_fee_recipient_unchkd(&fee_recipient);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient.clone()));
}

#[test]
fn test_set_fee_recipient() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_set_fee_recipient_auth(&env, &contract_id, &owner, &fee_recipient);
    client.set_fee_recipient(&fee_recipient);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient.clone()));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_fee_recipient_fails_without_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Don't mock auth - should fail
    client.set_fee_recipient(&fee_recipient);
}

#[test]
fn test_set_fee_recipient_can_be_updated() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let fee_recipient_1 = Address::generate(&env);
    let fee_recipient_2 = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Set initial fee recipient
    mock_set_fee_recipient_auth(&env, &contract_id, &owner, &fee_recipient_1);
    client.set_fee_recipient(&fee_recipient_1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient_1);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient_1));

    // Update to new fee recipient
    mock_set_fee_recipient_auth(&env, &contract_id, &owner, &fee_recipient_2);
    client.set_fee_recipient(&fee_recipient_2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient_2);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient_2));
}

// =============================================================================
// Error Path Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2100)")]
fn test_set_fee_recipient_fails_when_owner_not_set() {
    let env = Env::default();
    let fee_recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // No owner set - should fail with OwnerNotSet
    client.set_fee_recipient(&fee_recipient);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_owner_cannot_set_fee_recipient() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Mock auth for non-owner address
    mock_set_fee_recipient_auth(&env, &contract_id, &non_owner, &fee_recipient);
    client.set_fee_recipient(&fee_recipient);
}

// =============================================================================
// Idempotency Tests
// =============================================================================

#[test]
fn test_set_fee_recipient_same_address_emits_event() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Set fee recipient
    mock_set_fee_recipient_auth(&env, &contract_id, &owner, &fee_recipient);
    client.set_fee_recipient(&fee_recipient);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient.clone()));

    // Set same fee recipient again - still emits event
    mock_set_fee_recipient_auth(&env, &contract_id, &owner, &fee_recipient);
    client.set_fee_recipient(&fee_recipient);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&fee_recipient);
    assert_eq!(client.get_fee_recipient(), Some(fee_recipient));
}
