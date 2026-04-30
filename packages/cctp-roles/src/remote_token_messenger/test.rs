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

use crate::test_utils::remote_token_messenger::{
    mock_add_remote_token_messenger_auth, mock_remove_remote_token_messenger_auth,
};
use crate::test_utils::CctpEventAssertions;
use event_assertion::EventAssertion;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::BytesN as _, Address, BytesN, Env,
};

use super::RemoteTokenMessenger;

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn set_owner_unchecked(env: Env, owner: Address) {
        common_roles::ownable::set_owner_unchecked(&env, &owner);
    }

    pub fn add_remote_tm_unchecked(env: Env, domain: u32, token_messenger: BytesN<32>) {
        super::add_remote_token_messenger_unchecked(&env, domain, &token_messenger);
    }

    pub fn require_remote_token_messenger(e: &Env, domain: u32, token_messenger: BytesN<32>) {
        super::require_remote_token_messenger(e, domain, &token_messenger);
    }
}

#[contractimpl]
impl RemoteTokenMessenger for TestContract {
    fn add_remote_token_messenger(e: &Env, domain: u32, token_messenger: BytesN<32>) {
        super::add_remote_token_messenger(e, domain, &token_messenger);
    }

    fn remove_remote_token_messenger(e: &Env, domain: u32) {
        super::remove_remote_token_messenger(e, domain);
    }

    fn get_remote_token_messenger(e: &Env, domain: u32) -> Option<BytesN<32>> {
        super::get_remote_token_messenger(e, domain)
    }
}

// =============================================================================
// Add Remote Token Messenger
// =============================================================================

#[test]
fn test_add_remote_token_messenger_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain, &token_messenger);

    assert_eq!(
        client.get_remote_token_messenger(&domain),
        Some(token_messenger)
    );
}

#[test]
fn test_add_remote_token_messenger_unchecked_success() {
    let env = Env::default();
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.add_remote_tm_unchecked(&domain, &token_messenger);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain, &token_messenger);

    assert_eq!(
        client.get_remote_token_messenger(&domain),
        Some(token_messenger)
    );
}

#[test]
#[should_panic(expected = "#6400")]
fn test_add_remote_token_messenger_fails_if_already_set() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    // Try to add again - should fail
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "#6400")]
fn test_add_remote_token_messenger_fails_if_different_address_on_same_domain() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger_1 = BytesN::<32>::random(&env);
    let token_messenger_2 = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger_1);
    client.add_remote_token_messenger(&domain, &token_messenger_1);

    // Try to add a different address on the same domain - should fail
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger_2);
    client.add_remote_token_messenger(&domain, &token_messenger_2);
}

#[test]
#[should_panic(expected = "#6402")]
fn test_add_remote_token_messenger_fails_if_zero_address() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let zero_messenger = BytesN::<32>::from_array(&env, &[0u8; 32]);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &zero_messenger);
    client.add_remote_token_messenger(&domain, &zero_messenger);
}

#[test]
fn test_add_remote_token_messenger_multiple_domains() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger_1 = BytesN::<32>::random(&env);
    let token_messenger_2 = BytesN::<32>::random(&env);
    let domain_1 = 1u32;
    let domain_2 = 2u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain_1, &token_messenger_1);
    client.add_remote_token_messenger(&domain_1, &token_messenger_1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain_1, &token_messenger_1);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain_2, &token_messenger_2);
    client.add_remote_token_messenger(&domain_2, &token_messenger_2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain_2, &token_messenger_2);

    assert_eq!(
        client.get_remote_token_messenger(&domain_1),
        Some(token_messenger_1)
    );
    assert_eq!(
        client.get_remote_token_messenger(&domain_2),
        Some(token_messenger_2)
    );
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2100)")]
fn test_add_remote_token_messenger_fails_when_owner_not_set() {
    let env = Env::default();
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.add_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_add_remote_token_messenger_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Don't mock auth - should fail
    client.add_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_owner_cannot_add_remote_token_messenger() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &non_owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2100)")]
fn test_remove_remote_token_messenger_fails_when_owner_not_set() {
    let env = Env::default();
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    // Add via unchecked, then try to remove without owner
    client.add_remote_tm_unchecked(&domain, &BytesN::<32>::random(&env));

    client.remove_remote_token_messenger(&domain);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_remove_remote_token_messenger_requires_owner_auth() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.add_remote_tm_unchecked(&domain, &token_messenger);

    // Don't mock auth - should fail
    client.remove_remote_token_messenger(&domain);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_owner_cannot_remove_remote_token_messenger() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);
    client.add_remote_tm_unchecked(&domain, &token_messenger);

    mock_remove_remote_token_messenger_auth(&env, &contract_id, &non_owner, domain);
    client.remove_remote_token_messenger(&domain);
}

// =============================================================================
// Remove Remote Token Messenger
// =============================================================================

#[test]
fn test_remove_remote_token_messenger_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Add first
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    // Remove
    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_remote_token_messenger_removed(domain, &token_messenger);

    assert_eq!(client.get_remote_token_messenger(&domain), None);
}

#[test]
#[should_panic(expected = "#6401")]
fn test_remove_remote_token_messenger_fails_if_not_set() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);
}

#[test]
#[should_panic(expected = "#6401")]
fn test_remove_remote_token_messenger_fails_if_already_removed() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);

    // Try to remove again - should fail
    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);
}

#[test]
fn test_can_re_add_after_removing() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let new_token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Add
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain, &token_messenger);
    assert_eq!(
        client.get_remote_token_messenger(&domain),
        Some(token_messenger.clone())
    );

    // Remove
    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_remote_token_messenger_removed(domain, &token_messenger);
    assert_eq!(client.get_remote_token_messenger(&domain), None);

    // Re-add with different address
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &new_token_messenger);
    client.add_remote_token_messenger(&domain, &new_token_messenger);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain, &new_token_messenger);
    assert_eq!(
        client.get_remote_token_messenger(&domain),
        Some(new_token_messenger)
    );
}

#[test]
fn test_remove_does_not_affect_other_domains() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger_1 = BytesN::<32>::random(&env);
    let token_messenger_2 = BytesN::<32>::random(&env);
    let domain_1 = 1u32;
    let domain_2 = 2u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    // Add to both domains
    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain_1, &token_messenger_1);
    client.add_remote_token_messenger(&domain_1, &token_messenger_1);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain_2, &token_messenger_2);
    client.add_remote_token_messenger(&domain_2, &token_messenger_2);

    // Remove domain 1
    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain_1);
    client.remove_remote_token_messenger(&domain_1);

    // Domain 1 is gone
    assert_eq!(client.get_remote_token_messenger(&domain_1), None);

    // Domain 2 is unaffected
    assert_eq!(
        client.get_remote_token_messenger(&domain_2),
        Some(token_messenger_2.clone())
    );
}

// =============================================================================
// Get Remote Token Messenger
// =============================================================================

#[test]
fn test_get_remote_token_messenger_returns_none_when_not_set() {
    let env = Env::default();
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_remote_token_messenger(&domain), None);
}

// =============================================================================
// Require Remote Token Messenger
// =============================================================================

#[test]
fn test_require_remote_token_messenger_success() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    // Should not panic
    client.require_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "#6403")]
fn test_require_remote_token_messenger_fails_when_not_set() {
    let env = Env::default();
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.require_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "#6403")]
fn test_require_remote_token_messenger_fails_for_wrong_address() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let wrong_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    client.require_remote_token_messenger(&domain, &wrong_messenger);
}

#[test]
#[should_panic(expected = "#6403")]
fn test_require_remote_token_messenger_fails_for_wrong_domain() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let wrong_domain = 2u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    client.require_remote_token_messenger(&wrong_domain, &token_messenger);
}

#[test]
#[should_panic(expected = "#6403")]
fn test_require_remote_token_messenger_fails_after_removal() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    mock_remove_remote_token_messenger_auth(&env, &contract_id, &owner, domain);
    client.remove_remote_token_messenger(&domain);

    client.require_remote_token_messenger(&domain, &token_messenger);
}

#[test]
#[should_panic(expected = "#6403")]
fn test_require_remote_token_messenger_fails_for_zero_address() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let token_messenger = BytesN::<32>::random(&env);
    let zero_messenger = BytesN::<32>::from_array(&env, &[0u8; 32]);
    let domain = 1u32;
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_owner_unchecked(&owner);

    mock_add_remote_token_messenger_auth(&env, &contract_id, &owner, domain, &token_messenger);
    client.add_remote_token_messenger(&domain, &token_messenger);

    client.require_remote_token_messenger(&domain, &zero_messenger);
}
