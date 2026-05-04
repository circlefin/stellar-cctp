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

use super::Denylistable;
use crate::test_utils::denylistable::{
    mock_denylist_auth, mock_un_denylist_auth, mock_update_denylister_auth,
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
impl Denylistable for TestContract {
    fn get_denylister(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, super::DENYLISTER)
    }

    fn update_denylister(e: &Env, denylister: Address) {
        simple_role::set_role_and_emit_with_previous(
            e,
            super::DENYLISTER,
            &denylister,
            super::emit_denylister_changed,
        );
    }

    fn denylist(e: &Env, account: Address) {
        super::denylist(e, &account)
    }

    fn un_denylist(e: &Env, account: Address) {
        super::un_denylist(e, &account)
    }

    fn is_denylisted(e: &Env, account: Address) -> bool {
        super::is_denylisted(e, &account)
    }
}

#[contractimpl]
impl TestContract {
    pub fn update_denylister_unchecked(env: Env, denylister: Address) {
        simple_role::set_role_and_emit_with_previous_unchecked(
            &env,
            super::DENYLISTER,
            &denylister,
            super::emit_denylister_changed,
        );
    }

    pub fn enforce_denylister_auth(env: Env) {
        simple_role::enforce_role_auth(&env, super::DENYLISTER);
    }

    pub fn set_owner_unchecked(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }

    pub fn require_not_denylisted(env: Env, account: Address) {
        super::storage::require_not_denylisted(&env, &account);
    }
}

fn setup_denylister(
    env: &Env,
    contract_id: &Address,
    client: &TestContractClient,
) -> (Address, Address) {
    let owner = Address::generate(env);
    client.set_owner_unchecked(&owner);
    let denylister = Address::generate(env);
    mock_update_denylister_auth(env, contract_id, &owner, &denylister);
    client.update_denylister(&denylister);
    (denylister, owner)
}

// =============================================================================
// Denylister lifecycle tests
// =============================================================================

#[test]
fn test_get_denylister_returns_none_initially() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_denylister(), None);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_denylist_fails_when_denylister_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    client.denylist(&account);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_un_denylist_fails_when_denylister_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    client.un_denylist(&account);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_enforce_denylister_auth_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.enforce_denylister_auth();
}

#[test]
#[should_panic(expected = "#2100")]
fn test_update_denylister_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister(&denylister);
}

#[test]
fn test_update_denylister_sets_denylister_and_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let denylister = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &owner, &denylister);
    client.update_denylister(&denylister);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_denylister_changed(None, &denylister);

    assert_eq!(client.get_denylister(), Some(denylister));
}

#[test]
fn test_update_denylister_emits_event_with_previous() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let denylister1 = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &owner, &denylister1);
    client.update_denylister(&denylister1);

    let denylister2 = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &owner, &denylister2);
    client.update_denylister(&denylister2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_denylister_changed(Some(&denylister1), &denylister2);

    assert_eq!(client.get_denylister(), Some(denylister2));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_denylister_cannot_act_after_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (old_denylister, owner) = setup_denylister(&env, &contract_id, &client);

    // Transfer denylister role to a new address
    let new_denylister = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &owner, &new_denylister);
    client.update_denylister(&new_denylister);

    // Old denylister tries to denylist — should fail
    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &old_denylister, &account);
    client.denylist(&account);
}

#[test]
fn test_denylisted_persists_after_denylister_transfer() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (denylister, owner) = setup_denylister(&env, &contract_id, &client);

    // Denylist an address with the current denylister
    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylisted(&account);

    assert!(client.is_denylisted(&account));

    // Transfer denylister role
    let new_denylister = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &owner, &new_denylister);
    client.update_denylister(&new_denylister);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylister_changed(Some(&denylister), &new_denylister);

    // Denylisted address is still denylisted
    assert!(client.is_denylisted(&account));

    // New denylister can un-denylist
    mock_un_denylist_auth(&env, &contract_id, &new_denylister, &account);
    client.un_denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_un_denylisted(&account);

    assert!(!client.is_denylisted(&account));
}

// =============================================================================
// Authorization tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_denylister_requires_owner_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let denylister = Address::generate(&env);
    // No mock auth provided — should fail
    client.update_denylister(&denylister);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_denylist_requires_denylister_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    setup_denylister(&env, &contract_id, &client);

    let account = Address::generate(&env);
    // No mock auth provided — should fail
    client.denylist(&account);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_un_denylist_requires_denylister_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    setup_denylister(&env, &contract_id, &client);

    let account = Address::generate(&env);
    // No mock auth provided — should fail
    client.un_denylist(&account);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_denylister_fails_with_wrong_auth() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let wrong_caller = Address::generate(&env);
    let new_denylister = Address::generate(&env);

    mock_update_denylister_auth(&env, &contract_id, &wrong_caller, &new_denylister);
    client.update_denylister(&new_denylister);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_denylist() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (_, owner) = setup_denylister(&env, &contract_id, &client);

    // Owner provides auth as if they were the denylister — should fail
    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &owner, &account);
    client.denylist(&account);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_un_denylist() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (denylister, owner) = setup_denylister(&env, &contract_id, &client);

    // Denylist an address first
    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    // Owner provides auth as if they were the denylister — should fail
    mock_un_denylist_auth(&env, &contract_id, &owner, &account);
    client.un_denylist(&account);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_denylister_cannot_update_denylister() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (denylister, _) = setup_denylister(&env, &contract_id, &client);

    // Denylister tries to set a new denylister — should fail (owner-only)
    let new_denylister = Address::generate(&env);
    mock_update_denylister_auth(&env, &contract_id, &denylister, &new_denylister);
    client.update_denylister(&new_denylister);
}

// =============================================================================
// Denylist/un-denylist tests
// =============================================================================

#[test]
fn test_denylist_address() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);
    assert!(!client.is_denylisted(&account));

    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_denylisted(&account);

    assert!(client.is_denylisted(&account));
}

#[test]
fn test_un_denylist_address() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);
    assert!(client.is_denylisted(&account));

    mock_un_denylist_auth(&env, &contract_id, &denylister, &account);
    client.un_denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_un_denylisted(&account);

    assert!(!client.is_denylisted(&account));
}

#[test]
fn test_un_denylist_one_of_multiple() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylister_changed(None, &denylister);

    let account1 = Address::generate(&env);
    let account2 = Address::generate(&env);
    let account3 = Address::generate(&env);

    mock_denylist_auth(&env, &contract_id, &denylister, &account1);
    client.denylist(&account1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylisted(&account1);

    mock_denylist_auth(&env, &contract_id, &denylister, &account2);
    client.denylist(&account2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylisted(&account2);

    mock_denylist_auth(&env, &contract_id, &denylister, &account3);
    client.denylist(&account3);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylisted(&account3);

    mock_un_denylist_auth(&env, &contract_id, &denylister, &account2);
    client.un_denylist(&account2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_un_denylisted(&account2);

    assert!(client.is_denylisted(&account1));
    assert!(!client.is_denylisted(&account2));
    assert!(client.is_denylisted(&account3));
}

#[test]
fn test_is_denylisted_returns_false_for_non_denylisted() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    assert!(!client.is_denylisted(&account));
}

// =============================================================================
// require_not_denylisted tests
// =============================================================================

#[test]
fn test_require_not_denylisted_succeeds() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    client.require_not_denylisted(&account);
}

#[test]
#[should_panic(expected = "Error(Contract, #6100")]
fn test_require_not_denylisted_fails_when_denylisted() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);
    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    client.require_not_denylisted(&account);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_denylist_same_address_twice_emits_both_events() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);

    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_denylisted(&account);

    // Second denylist on the same address still emits
    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_denylisted(&account);

    assert!(client.is_denylisted(&account));
}

#[test]
fn test_un_denylist_same_address_twice_emits_both_events() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);

    mock_denylist_auth(&env, &contract_id, &denylister, &account);
    client.denylist(&account);
    assert!(client.is_denylisted(&account));

    mock_un_denylist_auth(&env, &contract_id, &denylister, &account);
    client.un_denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_un_denylisted(&account);

    assert!(!client.is_denylisted(&account));

    // Second un-denylist still emits
    mock_un_denylist_auth(&env, &contract_id, &denylister, &account);
    client.un_denylist(&account);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_un_denylisted(&account);

    assert!(!client.is_denylisted(&account));
}

#[test]
fn test_un_denylist_non_denylisted_address_still_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let denylister = Address::generate(&env);
    client.update_denylister_unchecked(&denylister);

    let account = Address::generate(&env);

    mock_un_denylist_auth(&env, &contract_id, &denylister, &account);
    client.un_denylist(&account);

    // Event is emitted even though the address was never denylisted
    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_un_denylisted(&account);

    assert!(!client.is_denylisted(&account));
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {

    #[test]
    fn test_prop_independent_addresses(seed1: u64, seed2: u64) {
        let env = Env::default();
        let denylister = Address::generate(&env);
        let account1 = Address::generate(&env);
        let account2 = Address::generate(&env);

        let contract_id = env.register(TestContract, ());
        let client = TestContractClient::new(&env, &contract_id);

        client.update_denylister_unchecked(&denylister);

        if seed1 % 2 == 0 {
            mock_denylist_auth(&env, &contract_id, &denylister, &account1);
            client.denylist(&account1);
        }

        if seed2 % 2 == 0 {
            mock_denylist_auth(&env, &contract_id, &denylister, &account2);
            client.denylist(&account2);
        }

        assert_eq!(client.is_denylisted(&account1), seed1 % 2 == 0);
        assert_eq!(client.is_denylisted(&account2), seed2 % 2 == 0);
    }
}
