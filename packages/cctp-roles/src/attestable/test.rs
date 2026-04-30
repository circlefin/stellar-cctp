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

use crate::attestable::*;
use crate::test_utils::attestable::{
    bytes_with_byte_set, bytes_with_range_filled, bytes_with_trailing_byte,
    fixture_dupe_signatures, fixture_invalid_order, fixture_valid, malleate_secp256k1_sig,
    mock_disable_attester_auth, mock_enable_attester_auth, mock_set_signature_threshold_auth,
    mock_update_attester_manager_auth,
};
use crate::test_utils::CctpEventAssertions;
use event_assertion::EventAssertion;
use simple_role;
use stellar_access::ownable::set_owner;

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, vec, Address, Bytes, BytesN, Env, Vec,
};

//  attester address: 0x725b06f73ff761ef5390e39315e2bfbf60d33f96
const ATTESTER_ADDRESS_1: [u8; 20] = [
    0x72, 0x5b, 0x06, 0xf7, 0x3f, 0xf7, 0x61, 0xef, 0x53, 0x90, 0xe3, 0x93, 0x15, 0xe2, 0xbf, 0xbf,
    0x60, 0xd3, 0x3f, 0x96,
];

// attester address: 0x52ed4cbff8dce6a19748043f3240ec03c834bcef
const ATTESTER_ADDRESS_2: [u8; 20] = [
    0x52, 0xed, 0x4c, 0xbf, 0xf8, 0xdc, 0xe6, 0xa1, 0x97, 0x48, 0x04, 0x3f, 0x32, 0x40, 0xec, 0x03,
    0xc8, 0x34, 0xbc, 0xef,
];

// attester address: 0xaabbccddeeff00112233445566778899aabbccdd (not a valid signer of any fixture)
const ATTESTER_ADDRESS_3: [u8; 20] = [
    0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
    0xaa, 0xbb, 0xcc, 0xdd,
];

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {
    pub fn set_owner_unchecked(env: Env, owner: Address) {
        set_owner(&env, &owner);
    }

    pub fn update_attester_manager(env: Env, attester_manager: Address) {
        simple_role::set_role_and_emit_with_previous(
            &env,
            super::ATTESTER_MANAGER,
            &attester_manager,
            super::emit_attester_manager_updated,
        );
    }

    pub fn set_attester_mgr_unchkd(env: Env, attester_manager: Address) {
        simple_role::set_role_and_emit_with_previous_unchecked(
            &env,
            super::ATTESTER_MANAGER,
            &attester_manager,
            super::emit_attester_manager_updated,
        );
    }

    pub fn get_attester_manager(env: Env) -> Option<Address> {
        simple_role::try_get_role(&env, super::ATTESTER_MANAGER)
    }

    pub fn enable_attester(env: Env, attester_address: BytesN<20>) {
        super::enable_attester(&env, &attester_address);
    }

    pub fn disable_attester(env: Env, attester_address: BytesN<20>) {
        super::disable_attester(&env, &attester_address);
    }

    pub fn is_enabled_attester(env: Env, attester_address: BytesN<20>) -> bool {
        super::is_enabled_attester(&env, &attester_address)
    }

    pub fn get_enabled_attester(env: Env, index: u32) -> BytesN<20> {
        super::get_enabled_attester(&env, index)
    }

    pub fn set_signature_threshold(env: Env, threshold: u32) {
        super::set_signature_threshold(&env, threshold);
    }

    pub fn get_signature_threshold(env: Env) -> Option<u32> {
        super::get_signature_threshold(&env)
    }

    pub fn verify_attestation_signatures(env: Env, message: Bytes, attestation: Bytes) {
        super::verify_attestation_signatures(&env, &message, &attestation);
    }
}

struct TestContext {
    env: Env,
    contract_id: Address,
    attester_manager: Address,
    owner: Address,
}

fn setup_attester_manager(
    env: &Env,
    contract_id: &Address,
    client: &TestContractClient,
) -> (Address, Address) {
    let owner = Address::generate(env);
    client.set_owner_unchecked(&owner);
    let attester_manager = Address::generate(env);
    mock_update_attester_manager_auth(env, contract_id, &owner, &attester_manager);
    client.update_attester_manager(&attester_manager);
    (attester_manager, owner)
}

fn setup_attesters(
    env: &Env,
    contract_id: &Address,
    client: &TestContractClient,
    attester_manager: &Address,
    threshold: u32,
) -> Vec<BytesN<20>> {
    if threshold != 1 && threshold != 2 {
        panic!("threshold must be 1 or 2");
    }

    let attester1 = BytesN::from_array(env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(env, contract_id, attester_manager, &attester1);
    client.enable_attester(&attester1);

    let attesters = if threshold == 2 {
        let attester2 = BytesN::from_array(env, &ATTESTER_ADDRESS_2);
        mock_enable_attester_auth(env, contract_id, attester_manager, &attester2);
        client.enable_attester(&attester2);
        vec![env, attester1, attester2]
    } else {
        vec![env, attester1]
    };

    mock_set_signature_threshold_auth(env, contract_id, attester_manager, threshold);
    client.set_signature_threshold(&threshold);

    attesters
}

fn setup_contract(threshold: u32) -> TestContext {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let (attester_manager, owner) = setup_attester_manager(&env, &contract_id, &client);
    setup_attesters(&env, &contract_id, &client, &attester_manager, threshold);
    TestContext {
        env,
        contract_id,
        attester_manager,
        owner,
    }
}

// ================================
// Attester manager lifecycle tests
// ================================

#[test]
fn test_get_attester_manager_returns_none_initially() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_attester_manager(), None);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_enable_attester_fails_when_attester_manager_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let attester = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    client.enable_attester(&attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_disable_attester_fails_when_attester_manager_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let attester = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    client.disable_attester(&attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #7000)")]
fn test_set_signature_threshold_fails_when_attester_manager_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    client.set_signature_threshold(&1);
}

#[test]
#[should_panic(expected = "#2100")]
fn test_update_attester_manager_fails_when_owner_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let new_manager = Address::generate(&env);
    client.update_attester_manager(&new_manager);
}

#[test]
fn test_update_attester_manager_sets_manager_and_emits_event() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let manager = Address::generate(&env);
    mock_update_attester_manager_auth(&env, &contract_id, &owner, &manager);
    client.update_attester_manager(&manager);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&None, &manager);

    assert_eq!(client.get_attester_manager(), Some(manager.clone()));
}

#[test]
fn test_update_attester_manager_emits_event_with_previous() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.set_owner_unchecked(&owner);

    let manager1 = Address::generate(&env);
    mock_update_attester_manager_auth(&env, &contract_id, &owner, &manager1);
    client.update_attester_manager(&manager1);

    let manager2 = Address::generate(&env);
    mock_update_attester_manager_auth(&env, &contract_id, &owner, &manager2);
    client.update_attester_manager(&manager2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&Some(manager1), &manager2);

    assert_eq!(client.get_attester_manager(), Some(manager2.clone()));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_old_attester_manager_cannot_act_after_transfer() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);

    // Transfer attester manager role to a new address
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_manager);
    client.update_attester_manager(&new_manager);

    // Old manager tries to enable an attester — should fail
    let attester = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_3);
    mock_enable_attester_auth(&ctx.env, &ctx.contract_id, &ctx.attester_manager, &attester);
    client.enable_attester(&attester);
}

#[test]
fn test_enabled_attester_persists_after_attester_manager_transfer() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);

    let attester1 = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_1);
    assert!(client.is_enabled_attester(&attester1));

    // Transfer attester manager role
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_manager);
    client.update_attester_manager(&new_manager);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&Some(ctx.attester_manager.clone()), &new_manager);

    // Attester enabled by old manager is still enabled
    assert!(client.is_enabled_attester(&attester1));
}

#[test]
fn test_new_attester_manager_can_enable_attester() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);

    // Transfer attester manager role
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_manager);
    client.update_attester_manager(&new_manager);

    // New manager can enable a new attester
    let attester3 = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_3);
    mock_enable_attester_auth(&ctx.env, &ctx.contract_id, &new_manager, &attester3);
    client.enable_attester(&attester3);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester3);

    assert!(client.is_enabled_attester(&attester3));
}

#[test]
fn test_signature_threshold_persists_after_attester_manager_transfer() {
    let ctx = setup_contract(2);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);

    assert_eq!(client.get_signature_threshold(), Some(2));

    // Transfer attester manager role
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_manager);
    client.update_attester_manager(&new_manager);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&Some(ctx.attester_manager.clone()), &new_manager);

    // Threshold persists after transfer
    assert_eq!(client.get_signature_threshold(), Some(2));
}

#[test]
fn test_new_attester_manager_can_change_threshold() {
    let ctx = setup_contract(2);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);

    // Transfer attester manager role
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_manager);
    client.update_attester_manager(&new_manager);

    // New manager can change threshold
    mock_set_signature_threshold_auth(&ctx.env, &ctx.contract_id, &new_manager, 1);
    client.set_signature_threshold(&1);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(2), 1);

    assert_eq!(client.get_signature_threshold(), Some(1));
}

#[test]
fn test_disabled_attester_state_persists_and_can_be_reenabled_after_manager_change() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    // Lower threshold to 1 so we can disable an attester
    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(2), 1);

    // Disable an attester
    let attester_to_disable = BytesN::from_array(&env, &ATTESTER_ADDRESS_2);
    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &attester_to_disable);
    client.disable_attester(&attester_to_disable);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_disabled(&attester_to_disable);

    assert!(!client.is_enabled_attester(&attester_to_disable));

    // Change the attester manager
    let new_manager = Address::generate(&env);
    mock_update_attester_manager_auth(&env, &contract_id, &owner, &new_manager);
    client.update_attester_manager(&new_manager);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&Some(attester_manager), &new_manager);

    // Verify disabled state persists
    assert!(!client.is_enabled_attester(&attester_to_disable));

    // Verify new manager can re-enable the disabled attester
    mock_enable_attester_auth(&env, &contract_id, &new_manager, &attester_to_disable);
    client.enable_attester(&attester_to_disable);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester_to_disable);

    assert!(client.is_enabled_attester(&attester_to_disable));
}

#[test]
fn test_enabled_attester_state_persists_and_can_be_disabled_after_manager_change() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    // Enable a third attester
    let attester3 = BytesN::from_array(&env, &ATTESTER_ADDRESS_3);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester3);
    client.enable_attester(&attester3);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester3);

    assert!(client.is_enabled_attester(&attester3));

    // Lower threshold to 1 so we can disable attesters
    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(2), 1);

    // Change the attester manager
    let new_manager = Address::generate(&env);
    mock_update_attester_manager_auth(&env, &contract_id, &owner, &new_manager);
    client.update_attester_manager(&new_manager);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_manager_updated(&Some(attester_manager), &new_manager);

    // Verify enabled state persists
    assert!(client.is_enabled_attester(&attester3));

    // Verify new manager can disable the enabled attester
    mock_disable_attester_auth(&env, &contract_id, &new_manager, &attester3);
    client.disable_attester(&attester3);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_attester_disabled(&attester3);

    assert!(!client.is_enabled_attester(&attester3));
}

// =============================================================================
// Authorization Tests (no auth provided)
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_attester_manager_requires_owner_auth() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let new_manager = Address::generate(&ctx.env);
    client.update_attester_manager(&new_manager);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_enable_attester_requires_attester_manager_auth() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let attester = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_3);
    client.enable_attester(&attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_disable_attester_requires_attester_manager_auth() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let attester = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_1);
    client.disable_attester(&attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_signature_threshold_requires_attester_manager_auth() {
    let ctx = setup_contract(2);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    client.set_signature_threshold(&1);
}

// =============================================================================
// Authorization Tests (owner cannot perform attester manager actions)
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_enable_attester() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let attester = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_3);
    mock_enable_attester_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &attester);
    client.enable_attester(&attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_disable_attester() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let attester = BytesN::from_array(&ctx.env, &ATTESTER_ADDRESS_1);
    mock_disable_attester_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &attester);
    client.disable_attester(&attester);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_owner_cannot_set_signature_threshold() {
    let ctx = setup_contract(2);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    mock_set_signature_threshold_auth(&ctx.env, &ctx.contract_id, &ctx.owner, 1);
    client.set_signature_threshold(&1);
}

// =============================================================================
// Authorization Tests (attester manager cannot perform owner actions)
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_attester_manager_cannot_update_attester_manager() {
    let ctx = setup_contract(1);
    let client = TestContractClient::new(&ctx.env, &ctx.contract_id);
    let new_manager = Address::generate(&ctx.env);
    mock_update_attester_manager_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.attester_manager,
        &new_manager,
    );
    client.update_attester_manager(&new_manager);
}

// ================================
// Enable/disable attester tests
// ================================

#[test]
fn test_enable_disable_attester() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester1 = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester1);
    client.enable_attester(&attester1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester1);

    assert!(client.is_enabled_attester(&attester1));
    assert_eq!(client.get_enabled_attester(&0_u32), attester1);

    // Need to go above threshold before we are allowed to disable an attester
    let attester2 = BytesN::from_array(&env, &ATTESTER_ADDRESS_2);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester2);
    client.enable_attester(&attester2);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester2);

    assert!(client.is_enabled_attester(&attester2));
    assert_eq!(client.get_enabled_attester(&1_u32), attester2);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(0), 1);

    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &attester2);
    client.disable_attester(&attester2);

    let mut events = EventAssertion::new(&env, contract_id);
    events.assert_event_count(1);
    events.assert_attester_disabled(&attester2);

    assert!(!client.is_enabled_attester(&attester2));
}

#[test]
#[should_panic(expected = "Error(Contract, #6008)")]
fn test_enable_attester_cannot_be_zero_address() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let zero_attester = BytesN::from_array(&env, &[0u8; 20]);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &zero_attester);
    client.enable_attester(&zero_attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #6005)")]
fn test_enable_attester_cannot_be_already_enabled() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester);
    client.enable_attester(&attester);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester);
    client.enable_attester(&attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #6006)")]
fn test_disable_attester_cannot_be_already_disabled() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    // Lower threshold to get around the threshold check
    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);

    let not_enabled_attester = BytesN::from_array(&env, &ATTESTER_ADDRESS_3);
    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &not_enabled_attester);
    client.disable_attester(&not_enabled_attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #6009)")]
fn test_disable_attester_cannot_leave_zero_enabled_attesters() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester);
    client.enable_attester(&attester);

    // Disabling the only enabled attester should fail.
    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &attester);
    client.disable_attester(&attester);
}

#[test]
#[should_panic(expected = "Error(Contract, #6009)")]
fn test_disable_attester_cannot_drop_below_signature_threshold() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    let attester_1 = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);

    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &attester_1);
    client.disable_attester(&attester_1);
}

#[test]
#[should_panic(expected = "Error(Contract, #6012)")]
fn test_disable_attester_fails_when_signature_threshold_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester1 = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester1);
    client.enable_attester(&attester1);

    let attester2 = BytesN::from_array(&env, &ATTESTER_ADDRESS_2);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester2);
    client.enable_attester(&attester2);

    // Attempt to disable attester without signature threshold being set
    mock_disable_attester_auth(&env, &contract_id, &attester_manager, &attester1);
    client.disable_attester(&attester1);
}

#[test]
#[should_panic(expected = "Error(Contract, #6007)")]
fn test_get_enabled_attester_index_out_of_bounds() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(1);
    let client = TestContractClient::new(&env, &contract_id);

    client.get_enabled_attester(&1_u32);
}

// ================================
// Signature threshold tests
// ================================

#[test]
fn test_set_and_get_signature_threshold() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    assert_eq!(client.get_signature_threshold(), None);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester1 = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester1);
    client.enable_attester(&attester1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_attester_enabled(&attester1);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);

    let mut events = EventAssertion::new(&env, contract_id.clone());
    events.assert_event_count(1);
    events.assert_signature_threshold_updated(Some(0), 1);

    assert_eq!(client.get_signature_threshold(), Some(1));
}

#[test]
#[should_panic(expected = "Error(Contract, #6004)")]
fn test_set_signature_threshold_cannot_be_zero() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 0);
    client.set_signature_threshold(&0);
}

#[test]
#[should_panic(expected = "Error(Contract, #6010)")]
fn test_set_signature_threshold_cannot_exceed_enabled_attesters() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(1);
    let client = TestContractClient::new(&env, &contract_id);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 2);
    client.set_signature_threshold(&2);
}

#[test]
#[should_panic(expected = "Error(Contract, #6011)")]
fn test_set_signature_threshold_cannot_be_same_value() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(1);
    let client = TestContractClient::new(&env, &contract_id);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 1);
    client.set_signature_threshold(&1);
}

// ================================
// Attestation verification tests
// ================================

#[test]
#[should_panic(expected = "Error(Contract, #6012)")]
fn test_verify_attestation_fails_when_signature_threshold_not_set() {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    let client = TestContractClient::new(&env, &contract_id);

    let (attester_manager, _) = setup_attester_manager(&env, &contract_id, &client);

    let attester1 = BytesN::from_array(&env, &ATTESTER_ADDRESS_1);
    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester1);
    client.enable_attester(&attester1);

    let fixture = fixture_valid(&env);

    // Attempt to verify attestation without signature threshold being set
    client.verify_attestation_signatures(&fixture.message, &fixture.attestation);
}

#[test]
fn test_verify_attestation_multiple_signatures() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    let fixture = fixture_valid(&env);

    client.verify_attestation_signatures(&fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6001)")]
fn test_verify_attestation_invalid_order() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    let fixture = fixture_invalid_order(&env);

    client.verify_attestation_signatures(&fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6001)")]
fn test_verify_attestation_duplicate_signatures() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);

    let fixture = fixture_dupe_signatures(&env);

    client.verify_attestation_signatures(&fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6002)")]
fn test_verify_attestation_recovered_signer_not_enabled_attester() {
    let TestContext {
        env,
        contract_id,
        attester_manager,
        owner: _,
    } = setup_contract(1);
    let client = TestContractClient::new(&env, &contract_id);

    let fixture = fixture_valid(&env);

    // Enable attester 3 (but not attester 2 who signed the fixture).
    let attester3 = BytesN::from_array(&env, &ATTESTER_ADDRESS_3);

    mock_enable_attester_auth(&env, &contract_id, &attester_manager, &attester3);
    client.enable_attester(&attester3);

    mock_set_signature_threshold_auth(&env, &contract_id, &attester_manager, 2);
    client.set_signature_threshold(&2);

    // Verification should fail because attester 2 (who signed the second signature) is not enabled.
    client.verify_attestation_signatures(&fixture.message, &fixture.attestation);
}

#[test]
#[should_panic(expected = "Error(Contract, #6000)")]
fn test_verify_attestation_length_too_short_by_one() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    let len = fixture.attestation.len();
    let shorter = fixture.attestation.slice(0..(len - 1));

    client.verify_attestation_signatures(&fixture.message, &shorter);
}

#[test]
#[should_panic(expected = "Error(Contract, #6000)")]
fn test_verify_attestation_length_too_long_by_one() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    let longer = bytes_with_trailing_byte(&env, &fixture.attestation, 0);

    client.verify_attestation_signatures(&fixture.message, &longer);
}

#[test]
#[should_panic(expected = "Error(Contract, #6000)")]
fn test_verify_attestation_length_single_signature_when_threshold_is_two() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    // expected length is SIGNATURE_LENGTH * threshold.
    let one_sig = fixture.attestation.slice(0..SIGNATURE_LENGTH);

    client.verify_attestation_signatures(&fixture.message, &one_sig);
}

#[test]
#[should_panic(expected = "Error(Contract, #6013)")]
fn test_verify_attestation_signature_recovery_failed_invalid_v() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    // Mutate the first signature's v byte (last byte of the 65-byte signature).
    let bad_attestation = bytes_with_byte_set(&env, &fixture.attestation, 64, 30);

    client.verify_attestation_signatures(&fixture.message, &bad_attestation);
}

#[test]
#[should_panic(expected = "invalid ECDSA sinature")]
fn test_verify_attestation_signature_recovery_failed_invalid_signature_s_value() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    // Zero out `s` in the first signature
    let bad_attestation = bytes_with_range_filled(&env, &fixture.attestation, 32, 64, 0);

    client.verify_attestation_signatures(&fixture.message, &bad_attestation);
}

#[test]
#[should_panic(expected = "invalid ECDSA sinature")]
fn test_verify_attestation_signature_recovery_failed_invalid_signature_r_value() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    // Set `r` to 0xff..ff (>= secp256k1 curve order), which is invalid.
    let bad_attestation = bytes_with_range_filled(&env, &fixture.attestation, 0, 32, 0xff);

    client.verify_attestation_signatures(&fixture.message, &bad_attestation);
}

#[test]
#[should_panic(expected = "ECDSA signature 's' part is not normalized to low form")]
fn test_verify_attestation_rejects_same_signer_twice_via_malleability() {
    let TestContext {
        env,
        contract_id,
        attester_manager: _,
        owner: _,
    } = setup_contract(2);
    let client = TestContractClient::new(&env, &contract_id);
    let fixture = fixture_valid(&env);

    let sig0 = fixture.attestation.slice(0..SIGNATURE_LENGTH);
    let sig0_malleated = malleate_secp256k1_sig(&env, &sig0);

    let mut att = Bytes::new(&env);
    att.append(&sig0);
    att.append(&sig0_malleated);

    client.verify_attestation_signatures(&fixture.message, &att);
}

// ============================================================================
// recover_secp256k1_public_key tests
// ============================================================================

#[test]
#[should_panic(expected = "Error(Object, IndexBounds)")]
fn test_recover_secp256k1_public_key_fails_when_signature_too_short_for_r() {
    let env = Env::default();
    let digest = env.crypto().keccak256(&Bytes::from_array(&env, &[0u8; 32]));

    // Only 31 bytes - slice(0..32) will fail with host IndexBounds error
    let short_sig = Bytes::from_array(&env, &[0u8; 31]);

    recover_secp256k1_public_key(&env, &digest, &short_sig);
}

#[test]
#[should_panic(expected = "Error(Object, IndexBounds)")]
fn test_recover_secp256k1_public_key_fails_when_signature_too_short_for_s() {
    let env = Env::default();
    let digest = env.crypto().keccak256(&Bytes::from_array(&env, &[0u8; 32]));

    // Only 63 bytes - slice(32..64) will fail with host IndexBounds error
    let short_sig = Bytes::from_array(&env, &[0u8; 63]);

    recover_secp256k1_public_key(&env, &digest, &short_sig);
}

#[test]
#[should_panic(expected = "Error(Contract, #6003)")]
fn test_recover_secp256k1_public_key_fails_when_signature_missing_v_byte() {
    let env = Env::default();
    let digest = env.crypto().keccak256(&Bytes::from_array(&env, &[0u8; 32]));

    // Exactly 64 bytes - has r and s but signature.get(64) returns None
    let short_sig = Bytes::from_array(&env, &[0u8; 64]);

    recover_secp256k1_public_key(&env, &digest, &short_sig);
}

#[test]
fn test_recover_secp256k1_public_key_with_normalized_v_byte() {
    let env = Env::default();
    let fixture = fixture_valid(&env);
    let digest = env.crypto().keccak256(&fixture.message);

    // Get the first signature (65 bytes)
    let sig = fixture.attestation.slice(0..SIGNATURE_LENGTH);

    // Get the original v byte (should be 27 or 28)
    let original_v = sig.get(64).unwrap();
    assert!(original_v == 27 || original_v == 28);

    // Create a signature with normalized v (0 or 1 instead of 27 or 28)
    let normalized_v = original_v - 27;
    let normalized_sig = bytes_with_byte_set(&env, &sig, 64, normalized_v);

    // Should recover the same public key
    let recovered = recover_secp256k1_public_key(&env, &digest, &normalized_sig);

    // Verify it's a valid 64-byte public key (non-zero)
    assert!(recovered.to_array().iter().any(|&b| b != 0));
}

#[test]
#[should_panic(expected = "Error(Contract, #6013)")]
fn test_recover_secp256k1_public_key_fails_when_recovery_id_invalid_v2() {
    let env = Env::default();
    let fixture = fixture_valid(&env);
    let digest = env.crypto().keccak256(&fixture.message);

    // Get the first signature (65 bytes)
    let sig = fixture.attestation.slice(0..SIGNATURE_LENGTH);

    // Set v=2, which is an invalid raw recovery ID (must be 0 or 1)
    let bad_sig = bytes_with_byte_set(&env, &sig, 64, 2);

    recover_secp256k1_public_key(&env, &digest, &bad_sig);
}

#[test]
#[should_panic(expected = "Error(Contract, #6013)")]
fn test_recover_secp256k1_public_key_fails_when_recovery_id_invalid_v30() {
    let env = Env::default();
    let fixture = fixture_valid(&env);
    let digest = env.crypto().keccak256(&fixture.message);

    // Get the first signature (65 bytes)
    let sig = fixture.attestation.slice(0..SIGNATURE_LENGTH);

    // Set v=30, which normalizes to 30-27=3, an invalid recovery ID
    let bad_sig = bytes_with_byte_set(&env, &sig, 64, 30);

    recover_secp256k1_public_key(&env, &digest, &bad_sig);
}
