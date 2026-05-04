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

use crate::contract::{
    TokenMessengerMinterV2Contract, TokenMessengerMinterV2ContractClient,
    TokenMessengerMinterV2ContractInitParams,
};
use crate::storage::TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD;
use crate::test_utils::{
    create_test_burn_message, create_test_burn_message_with_expiration, create_test_token,
    mock_approve_auth, mock_deposit_for_burn_auth,
    mock_deposit_for_burn_with_decimal_conversion_auth, mock_deposit_for_burn_with_hook_auth,
    mock_handle_recv_finalized_message_auth, mock_handle_recv_unfinalized_message_auth,
    mock_sac_mint_auth, setup_contract, setup_deposit_test, setup_deposit_test_with_decimals,
    setup_receive_test, setup_receive_test_with_decimals, BurnMessageV2,
    FailingMockMessageTransmitter, MessageV2, StorageTestStub, StorageTestStubClient,
    MESSAGE_BODY_VERSION, UPGRADE_V2_WASM,
};
use cctp_roles::test_utils::denylistable::mock_denylist_auth;
use cctp_roles::test_utils::fee_recipient::mock_set_fee_recipient_auth;
use cctp_roles::test_utils::min_fee_controller::{
    mock_set_min_fee_auth, mock_set_min_fee_controller_auth,
};
use cctp_roles::test_utils::remote_token_messenger::{
    mock_add_remote_token_messenger_auth, mock_remove_remote_token_messenger_auth,
};
use cctp_roles::test_utils::token_controller::{
    mock_link_token_pair_auth, mock_remove_swap_minter_config_auth,
    mock_set_max_burn_amount_per_message_auth, mock_set_swap_minter_config_auth,
    mock_set_token_controller_auth, mock_set_token_decimal_config_auth,
    mock_unlink_token_pair_auth,
};
use cctp_roles::test_utils::CctpEventAssertions;
use common_roles::test_utils::pausable::mock_pause_auth;
use common_roles::test_utils::CommonEventAssertions;
use event_assertion::EventAssertion;
use rstest::rstest;
use soroban_sdk::address_payload::AddressPayload;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Bytes, BytesN, Env, U256,
};

// =============================================================================
// Constructor Tests
// =============================================================================

#[test]
fn test_constructor_initializes_state_and_emits_events() {
    let ctx = setup_contract();
    let client = ctx.client();

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());

    events.assert_event_count(8);
    events.assert_ownership_transfer_completed(&ctx.owner);
    events.assert_pauser_changed(&ctx.pauser);
    events.assert_rescuer_changed(&ctx.rescuer);
    events.assert_set_token_controller(&ctx.token_controller);
    events.assert_admin_changed(None, &ctx.admin);
    events.assert_fee_recipient_set(&ctx.fee_recipient);
    events.assert_min_fee_controller_set(&ctx.min_fee_controller);
    events.assert_denylister_changed(None, &ctx.denylister);

    assert_eq!(client.get_owner(), Some(ctx.owner.clone()));
    assert_eq!(client.get_pauser(), Some(ctx.pauser.clone()));
    assert_eq!(client.get_rescuer(), Some(ctx.rescuer.clone()));
    assert_eq!(
        client.get_token_controller(),
        Some(ctx.token_controller.clone())
    );
    assert_eq!(client.get_admin(), Some(ctx.admin.clone()));
    assert_eq!(client.get_fee_recipient(), Some(ctx.fee_recipient.clone()));
    assert_eq!(client.get_denylister(), Some(ctx.denylister.clone()));
    assert_eq!(
        client.get_local_message_transmitter(),
        ctx.message_transmitter.clone()
    );
    assert_eq!(client.get_message_body_version(), MESSAGE_BODY_VERSION);
    assert!(!client.paused());
}

#[test]
fn test_constructor_initializes_remote_token_messengers() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let message_transmitter = Address::generate(&env);

    let remote_tm_1 = BytesN::<32>::random(&env);
    let remote_tm_2 = BytesN::<32>::random(&env);
    let domain_1 = 1u32;
    let domain_2 = 2u32;

    let contract_id = env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer,
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient,
            min_fee_controller,
            denylister,
            message_transmitter,
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, domain_1, domain_2],
            remote_token_messengers: vec![&env, remote_tm_1.clone(), remote_tm_2.clone()],
        },),
    );

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);

    // Verify remote token messengers were set
    assert_eq!(
        client.get_remote_token_messenger(&domain_1),
        Some(remote_tm_1.clone())
    );
    assert_eq!(
        client.get_remote_token_messenger(&domain_2),
        Some(remote_tm_2.clone())
    );
    assert_eq!(
        client.get_remote_token_messenger(&domain_1),
        Some(remote_tm_1.clone())
    );
    assert_eq!(
        client.get_remote_token_messenger(&domain_2),
        Some(remote_tm_2.clone())
    );

    // Verify non-existent domain returns None
    assert_eq!(client.get_remote_token_messenger(&999), None);
}

#[test]
#[should_panic(expected = "remote_domains and remote_token_messengers must have the same length")]
fn test_constructor_fails_mismatched_remote_token_messenger_arrays() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let message_transmitter = Address::generate(&env);

    let remote_tm_1 = BytesN::<32>::random(&env);

    // Provide 2 domains but only 1 token messenger
    env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            token_controller,
            admin,
            fee_recipient,
            min_fee_controller,
            denylister,
            message_transmitter,
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, 1u32, 2u32],
            remote_token_messengers: vec![&env, remote_tm_1],
        },),
    );
}

#[test]
#[should_panic(expected = "remote_domains and remote_token_messengers must have the same length")]
fn test_constructor_fails_one_domain_two_messengers() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let message_transmitter = Address::generate(&env);

    let remote_tm_1 = BytesN::<32>::random(&env);
    let remote_tm_2 = BytesN::<32>::random(&env);

    // Provide 1 domain but 2 token messengers
    env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner,
            pauser,
            rescuer,
            token_controller,
            admin,
            fee_recipient,
            min_fee_controller,
            denylister,
            message_transmitter,
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, 1u32],
            remote_token_messengers: vec![&env, remote_tm_1, remote_tm_2],
        },),
    );
}

#[test]
fn test_constructor_passes_zero_domains_and_zero_messengers() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let min_fee_controller = Address::generate(&env);
    let denylister = Address::generate(&env);
    let message_transmitter = Address::generate(&env);

    // Provide zero domains and zero token messengers - should pass
    let contract_id = env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser,
            rescuer,
            token_controller,
            admin,
            fee_recipient,
            min_fee_controller,
            denylister,
            message_transmitter,
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env],
            remote_token_messengers: vec![&env],
        },),
    );

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);

    // Verify contract initialized correctly
    assert_eq!(client.get_owner(), Some(owner));
    // Verify no remote token messengers exist
    assert_eq!(client.get_remote_token_messenger(&1), None);
}

// =============================================================================
// Role Implementation Tests
// =============================================================================

#[test]
fn test_contract_is_ownable() {
    let ctx = setup_contract();
    let client = ctx.client();

    common_roles::assert_contract_is_ownable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_contract_is_pausable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_pausable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.pauser,
        &ctx.owner
    );
}

#[test]
fn test_contract_is_rescuable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_rescuable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
fn test_contract_is_manageable() {
    let ctx = setup_contract();
    let client = ctx.client();
    common_roles::assert_contract_is_manageable!(
        &ctx.env,
        &client,
        &ctx.contract_id,
        &ctx.owner,
        &ctx.admin,
        UPGRADE_V2_WASM
    );
}

#[test]
fn test_contract_is_denylistable() {
    let ctx = setup_contract();
    let client = ctx.client();

    cctp_roles::assert_contract_is_denylistable!(&ctx.env, &client, &ctx.contract_id, &ctx.owner);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_deposit_for_burn_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    client.deposit_for_burn(
        &caller,
        &1,
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &Address::generate(&ctx.env),
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_deposit_for_burn_with_hook_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    client.deposit_for_burn_with_hook(
        &caller,
        &1,
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &Address::generate(&ctx.env),
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        &0,
        &0,
        &Bytes::from_array(&ctx.env, &[]),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_handle_recv_finalized_message_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    client.handle_recv_finalized_message(
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &0,
        &Bytes::from_array(&ctx.env, &[]),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1000)")]
fn test_handle_recv_unfinalized_message_fails_when_paused() {
    let ctx = setup_contract();
    let client = ctx.client();

    mock_pause_auth(&ctx.env, &ctx.contract_id, &ctx.pauser);
    client.pause();

    client.handle_recv_unfinalized_message(
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &0,
        &Bytes::from_array(&ctx.env, &[]),
    );
}

// =============================================================================
// Authorization Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_deposit_for_burn_requires_caller_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    client.deposit_for_burn(
        &caller,
        &1,
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &Address::generate(&ctx.env),
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &0,
        &0,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6100)")]
fn test_deposit_for_burn_fails_when_caller_is_denylisted() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    mock_denylist_auth(&ctx.env, &ctx.contract_id, &ctx.denylister, &ctx.caller);
    client.denylist(&ctx.caller);

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 100_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6100)")]
fn test_deposit_for_burn_with_hook_fails_when_caller_is_denylisted() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    mock_denylist_auth(&ctx.env, &ctx.contract_id, &ctx.denylister, &ctx.caller);
    client.denylist(&ctx.caller);

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 100_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;
    let hook_data = Bytes::from_array(&ctx.env, &[1, 2, 3, 4]);

    mock_deposit_for_burn_with_hook_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &hook_data,
    );

    client.deposit_for_burn_with_hook(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
        &hook_data,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_deposit_for_burn_with_hook_requires_caller_auth() {
    let ctx = setup_contract();
    let client = ctx.client();
    let caller = Address::generate(&ctx.env);

    client.deposit_for_burn_with_hook(
        &caller,
        &1,
        &0,
        &BytesN::from_array(&ctx.env, &[0; 32]),
        &Address::generate(&ctx.env),
        &BytesN::from_array(&ctx.env, &[0u8; 32]),
        &0,
        &0,
        &Bytes::from_array(&ctx.env, &[]),
    );
}

// =============================================================================
// MinFeeController Tests
// =============================================================================

#[test]
fn test_min_fee_controller_methods() {
    let ctx = setup_contract();
    let client = ctx.client();

    // get_min_fee_controller returns the value set during constructor
    assert_eq!(
        client.get_min_fee_controller(),
        Some(ctx.min_fee_controller.clone())
    );

    // set_min_fee_controller can update to a new controller
    let new_controller = Address::generate(&ctx.env);
    mock_set_min_fee_controller_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_controller);
    client.set_min_fee_controller(&new_controller);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_controller_set(&new_controller);

    assert_eq!(
        client.get_min_fee_controller(),
        Some(new_controller.clone())
    );

    // set_min_fee and get_min_fee
    let burn_token = Address::generate(&ctx.env);
    let min_fee: i128 = 100;
    mock_set_min_fee_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &burn_token,
        &min_fee,
    );
    client.set_min_fee(&burn_token, &min_fee);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_min_fee_set(&burn_token, &min_fee);

    assert_eq!(client.get_min_fee(&burn_token), min_fee);
}

/// Parameterized test for get_min_fee_amount with decimal normalization.
///
/// The fee path is: local amount -> normalize (strip dust) -> canonical -> fee calc -> local fee.
/// With local_decimals=7, canonical_decimals=6, min_fee=100, MIN_FEE_MULTIPLIER=10_000_000:
///   canonical_fee = canonical_amount * 100 / 10_000_000 (min 1 if non-zero min_fee configured)
///   local_fee = canonical_fee * 10
#[rstest]
// Clean amounts (no dust)
#[case::clean_amount(100_000_000, 1000)]
#[case::small_clean_amount(50_000, 10)]
// Dust is stripped before fee calculation — same result as clean
#[case::dust_large(100_000_009, 1000)]
#[case::dust_small(50_009, 10)]
// Boundary: smallest valid amount (20 local = 2 canonical, since canonical <= 1 panics)
// fee = 2 * 100 / 10_000_000 = 0 -> min floor of 1 canonical = 10 local
#[case::min_valid_amount(20, 10)]
#[case::min_valid_with_dust(29, 10)]
// Larger amounts
#[case::one_usdc(10_000_000, 100)]
#[case::one_usdc_with_dust(10_000_005, 100)]
fn test_get_min_fee_amount(#[case] amount: i128, #[case] expected_fee: i128) {
    let ctx = setup_contract();
    let client = ctx.client();

    let burn_token = Address::generate(&ctx.env);
    let min_fee: i128 = 100;

    // Set up min fee controller and fee
    let new_controller = Address::generate(&ctx.env);
    mock_set_min_fee_controller_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_controller);
    client.set_min_fee_controller(&new_controller);
    mock_set_min_fee_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &burn_token,
        &min_fee,
    );
    client.set_min_fee(&burn_token, &min_fee);

    // Set up decimal config (7 local / 6 canonical)
    mock_set_token_decimal_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.token_controller,
        &burn_token,
        7,
        6,
    );
    client.set_token_decimal_config(&burn_token, &7, &6);

    assert_eq!(
        client.get_min_fee_amount(&burn_token, &amount),
        expected_fee
    );
}

/// Parameterized test for get_min_fee_amount error cases.
///
/// Amounts that normalize to canonical <= 1 or are negative should panic.
#[rstest]
// Pure dust: normalizes to 0 canonical
#[case::pure_dust(9)]
// Normalizes to exactly 1 canonical (amount <= 1 check in min_fee_controller)
#[case::canonical_one(10)]
// Zero amount
#[case::zero(0)]
// Negative amounts
#[case::negative(-1)]
#[case::negative_large(-100_000_000)]
#[should_panic(expected = "HostError: Error(Contract, #6202)")] // MinFeeControllerError::AmountTooLow
fn test_get_min_fee_amount_rejects_invalid_amounts(#[case] amount: i128) {
    let ctx = setup_contract();
    let client = ctx.client();

    let burn_token = Address::generate(&ctx.env);
    let min_fee: i128 = 100;

    let new_controller = Address::generate(&ctx.env);
    mock_set_min_fee_controller_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_controller);
    client.set_min_fee_controller(&new_controller);
    mock_set_min_fee_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &burn_token,
        &min_fee,
    );
    client.set_min_fee(&burn_token, &min_fee);

    mock_set_token_decimal_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.token_controller,
        &burn_token,
        7,
        6,
    );
    client.set_token_decimal_config(&burn_token, &7, &6);

    // Should panic for all invalid amounts
    client.get_min_fee_amount(&burn_token, &amount);
}

// =============================================================================
// TokenController Tests
// =============================================================================

#[test]
fn test_token_controller_methods() {
    let ctx = setup_contract();
    let client = ctx.client();

    // get_token_controller returns the controller set during init
    assert_eq!(
        client.get_token_controller(),
        Some(ctx.token_controller.clone())
    );

    // set_token_controller
    let new_controller = Address::generate(&ctx.env);
    mock_set_token_controller_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_controller);
    client.set_token_controller(&new_controller);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_set_token_controller(&new_controller);

    assert_eq!(client.get_token_controller(), Some(new_controller.clone()));

    // link_token_pair and get_local_token
    let local_token = Address::generate(&ctx.env);
    let remote_domain = 1_u32;
    let remote_token = BytesN::<32>::random(&ctx.env);
    mock_link_token_pair_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_linked(&local_token, remote_domain, &remote_token);

    assert_eq!(
        client.get_local_token(&remote_domain, &remote_token),
        Some(local_token.clone())
    );

    // unlink_token_pair
    mock_unlink_token_pair_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.unlink_token_pair(&local_token, &remote_domain, &remote_token);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_pair_unlinked(&local_token, remote_domain, &remote_token);

    assert_eq!(client.get_local_token(&remote_domain, &remote_token), None);

    // set_max_burn_amount_per_message and get_max_burn_amount_per_message
    let burn_limit = 1_000_000_i128;
    mock_set_max_burn_amount_per_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &local_token,
        burn_limit,
    );
    client.set_max_burn_amount_per_message(&local_token, &burn_limit);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_set_burn_limit_per_message(&local_token, burn_limit);

    assert_eq!(
        client.get_max_burn_amount_per_message(&local_token),
        Some(burn_limit)
    );

    // set_token_decimal_config and get_token_decimal_config
    mock_set_token_decimal_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &new_controller,
        &local_token,
        7,
        6,
    );
    client.set_token_decimal_config(&local_token, &7, &6);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_token_decimal_config_added(&local_token, 7, 6);

    let config = client.get_token_decimal_config(&local_token);
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.local_decimals, 7);
    assert_eq!(config.canonical_decimals, 6);
}

#[test]
fn test_swap_minter_config_methods() {
    let ctx = setup_contract();
    let client = ctx.client();

    let local_token = Address::generate(&ctx.env);
    let swap_minter = Address::generate(&ctx.env);
    let allow_asset = Address::generate(&ctx.env);

    // get_swap_minter_config returns None when not set
    assert_eq!(client.get_swap_minter_config(&local_token), None);

    // set_swap_minter_config
    mock_set_swap_minter_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.token_controller,
        &local_token,
        &swap_minter,
        &allow_asset,
    );
    client.set_swap_minter_config(&local_token, &swap_minter, &allow_asset);

    // get_swap_minter_config returns the configured values
    let config = client.get_swap_minter_config(&local_token);
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.swap_minter, swap_minter);
    assert_eq!(config.allow_asset, allow_asset);

    // remove_swap_minter_config
    mock_remove_swap_minter_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.token_controller,
        &local_token,
    );
    client.remove_swap_minter_config(&local_token);
    assert_eq!(client.get_swap_minter_config(&local_token), None);
}

// =============================================================================
// RemoteTokenMessenger Tests
// =============================================================================

#[test]
fn test_add_and_remove_remote_token_messenger() {
    let ctx = setup_contract();
    let client = ctx.client();

    let domain = 5_u32;
    let token_messenger = BytesN::<32>::random(&ctx.env);

    // get_remote_token_messenger returns None when not set
    assert_eq!(client.get_remote_token_messenger(&domain), None);

    // add_remote_token_messenger
    mock_add_remote_token_messenger_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.owner,
        domain,
        &token_messenger,
    );
    client.add_remote_token_messenger(&domain, &token_messenger);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_remote_token_messenger_added(domain, &token_messenger);

    // get_remote_token_messenger returns the configured value
    assert_eq!(
        client.get_remote_token_messenger(&domain),
        Some(token_messenger.clone())
    );

    // remove_remote_token_messenger
    mock_remove_remote_token_messenger_auth(&ctx.env, &ctx.contract_id, &ctx.owner, domain);
    client.remove_remote_token_messenger(&domain);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_remote_token_messenger_removed(domain, &token_messenger);

    // get_remote_token_messenger returns None after removal
    assert_eq!(client.get_remote_token_messenger(&domain), None);
}

// =============================================================================
// FeeRecipient Tests
// =============================================================================

#[test]
fn test_fee_recipient_initialized_in_constructor() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_fee_recipient(), Some(ctx.fee_recipient));
}

#[test]
fn test_set_fee_recipient_success() {
    let ctx = setup_contract();
    let client = ctx.client();

    let new_fee_recipient = Address::generate(&ctx.env);
    mock_set_fee_recipient_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_fee_recipient);
    client.set_fee_recipient(&new_fee_recipient);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_event_count(1);
    events.assert_fee_recipient_set(&new_fee_recipient);

    assert_eq!(client.get_fee_recipient(), Some(new_fee_recipient.clone()));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_set_fee_recipient_fails_without_owner_auth() {
    let ctx = setup_contract();
    let client = ctx.client();

    let new_fee_recipient = Address::generate(&ctx.env);
    // Don't mock auth - should fail
    client.set_fee_recipient(&new_fee_recipient);
}

#[test]
fn test_fee_recipient_can_be_updated() {
    let ctx = setup_contract();
    let client = ctx.client();

    // First update
    let new_fee_recipient_1 = Address::generate(&ctx.env);
    mock_set_fee_recipient_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_fee_recipient_1);
    client.set_fee_recipient(&new_fee_recipient_1);
    assert_eq!(client.get_fee_recipient(), Some(new_fee_recipient_1));

    // Second update
    let new_fee_recipient_2 = Address::generate(&ctx.env);
    mock_set_fee_recipient_auth(&ctx.env, &ctx.contract_id, &ctx.owner, &new_fee_recipient_2);
    client.set_fee_recipient(&new_fee_recipient_2);
    assert_eq!(client.get_fee_recipient(), Some(new_fee_recipient_2));
}

// =============================================================================
// Storage Tests
// =============================================================================

#[test]
fn test_get_local_message_transmitter_returns_value_set_in_constructor() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(
        client.get_local_message_transmitter(),
        ctx.message_transmitter
    );
}

#[test]
fn test_get_message_body_version_returns_value_set_in_constructor() {
    let ctx = setup_contract();
    let client = ctx.client();

    assert_eq!(client.get_message_body_version(), MESSAGE_BODY_VERSION);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7100)")]
fn test_get_local_message_transmitter_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestStub, ());
    let client = StorageTestStubClient::new(&env, &contract_id);

    client.get_local_msg_transmitter();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7101)")]
fn test_get_message_body_version_fails_when_not_set() {
    let env = Env::default();
    let contract_id = env.register(StorageTestStub, ());
    let client = StorageTestStubClient::new(&env, &contract_id);

    client.get_msg_body_version();
}

// =============================================================================
// deposit_for_burn Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7102)")] // AmountMustBeNonzero
fn test_deposit_for_burn_fails_with_zero_amount() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 0_i128;
    let max_fee = 0_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount, // zero amount
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7102)")] // AmountMustBeNonzero
fn test_deposit_for_burn_fails_with_negative_amount() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = -100_i128;
    let max_fee = 0_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount, // negative amount
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7103)")] // MintRecipientMustBeNonzero
fn test_deposit_for_burn_fails_with_zero_mint_recipient() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let zero_recipient = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let max_fee = 0_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &zero_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &zero_recipient, // zero recipient
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7124)")] // MaxFeeMustBeNonNegative
fn test_deposit_for_burn_fails_with_negative_max_fee() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let max_fee = -1_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7104)")] // MaxFeeMustBeLessThanAmount
fn test_deposit_for_burn_fails_when_max_fee_equals_amount() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let max_fee = 1000_i128; // max_fee == amount
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7104)")] // MaxFeeMustBeLessThanAmount
fn test_deposit_for_burn_fails_when_max_fee_exceeds_amount() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let max_fee = 2000_i128; // max_fee > amount
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7104)")] // MaxFeeMustBeLessThanAmount
fn test_deposit_for_burn_fails_when_canonical_max_fee_equals_canonical_amount() {
    // This tests the edge case where local amounts pass validation but canonical amounts fail.
    let ctx = setup_deposit_test_with_decimals(7, 6);
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    // Local amounts: amount=11, max_fee=10
    // In local decimals: max_fee (10) < amount (11)
    // canonical_max_fee (1) >= canonical_amount (1) - should fail
    let amount = 11_i128;
    let max_fee = 10_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_with_decimal_conversion_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7104)")] // MaxFeeMustBeLessThanAmount
fn test_deposit_for_burn_fails_when_amount_normalization_makes_canonical_fee_equal_amount() {
    // This tests the edge case where amount normalization (dust removal) causes amount to equal max_fee
    let ctx = setup_deposit_test_with_decimals(7, 6);
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    // Local amounts: amount=105, max_fee=100
    // In local decimals: max_fee (100) < amount (105)
    // canonical_max_fee (10) >= canonical_burn_amount (10) - should fail
    let amount = 105_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_with_decimal_conversion_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7106)")] // NoTokenMessengerForDomain
fn test_deposit_for_burn_fails_with_unregistered_domain() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let unregistered_domain = 999_u32;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        unregistered_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &unregistered_domain, // unregistered domain
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6303)")] // TokenControllerError::BurnTokenNotSupported
fn test_deposit_for_burn_fails_with_unsupported_burn_token() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let unsupported_token = Address::generate(&ctx.env);
    let amount = 1000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    // Set decimal config for the unsupported token (but no burn limit)
    mock_set_token_decimal_config_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.token_controller,
        &unsupported_token,
        6,
        6,
    );
    client.set_token_decimal_config(&unsupported_token, &6, &6);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &unsupported_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &unsupported_token, // unsupported token (no burn limit configured)
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6304)")] // TokenControllerError::BurnAmountExceedsLimit
fn test_deposit_for_burn_fails_when_amount_exceeds_limit() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 2_000_000_i128; // exceeds limit
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    // burn limit is 1_000_000, so try to burn more
    client.deposit_for_burn(
        &ctx.caller,
        &amount, // exceeds limit
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")] // Balance error from SAC
fn test_deposit_for_burn_fails_with_insufficient_balance() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let poor_caller = Address::generate(&ctx.env);

    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let expiration_ledger = ctx.env.ledger().sequence() + 1000;
    mock_approve_auth(
        &ctx.env,
        &ctx.burn_token,
        &poor_caller,
        &ctx.contract_id,
        i128::MAX,
        expiration_ledger,
    );
    token_client.approve(
        &poor_caller,
        &ctx.contract_id,
        &i128::MAX,
        &expiration_ledger,
    );

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &poor_caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &poor_caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7107)")] // HookDataEmpty
fn test_deposit_for_burn_with_hook_fails_with_empty_hook_data() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let empty_hook_data = Bytes::new(&ctx.env);
    let amount = 1000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 0_u32;

    mock_deposit_for_burn_with_hook_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &empty_hook_data,
    );

    client.deposit_for_burn_with_hook(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
        &empty_hook_data, // empty hook data
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7105)")] // InsufficientMaxFee
fn test_deposit_for_burn_fails_when_max_fee_below_min_fee() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    // Configure min_fee_controller and set a min_fee
    let min_fee_controller_addr = Address::generate(&ctx.env);
    mock_set_min_fee_controller_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.owner,
        &min_fee_controller_addr,
    );
    client.set_min_fee_controller(&min_fee_controller_addr);

    // Set min_fee to 100 (out of 10_000_000 = 0.001% = 10 basis points / 1000)
    // For amount 100_000, min_fee_amount = 100_000 * 100 / 10_000_000 = 1
    let min_fee: i128 = 100;
    mock_set_min_fee_auth(
        &ctx.env,
        &ctx.contract_id,
        &min_fee_controller_addr,
        &ctx.burn_token,
        &min_fee,
    );
    client.set_min_fee(&ctx.burn_token, &min_fee);

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 100_000_i128;
    let max_fee = 0_i128;
    let min_finality_threshold = 0_u32;

    // Mock auth for the deposit_for_burn call (even though it will fail validation)
    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    // Try with max_fee = 0, which is less than min_fee_amount = 1
    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee, // max_fee < min_fee_amount
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "MessageTransmitter send_message failed")]
fn test_deposit_for_burn_fails_when_message_transmitter_fails() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let denylister = Address::generate(&env);
    let caller = Address::generate(&env);

    // Register a FAILING mock message transmitter
    let message_transmitter = env.register(FailingMockMessageTransmitter, ());

    let destination_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);

    let min_fee_controller = Address::generate(&env);

    let contract_id = env.register(
        TokenMessengerMinterV2Contract,
        (TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer,
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, destination_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );

    let burn_token = create_test_token(&env, &owner);

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);
    let burn_limit = 1_000_000_i128;
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &token_controller,
        &burn_token,
        burn_limit,
    );
    client.set_max_burn_amount_per_message(&burn_token, &burn_limit);

    mock_set_token_decimal_config_auth(&env, &contract_id, &token_controller, &burn_token, 6, 6);
    client.set_token_decimal_config(&burn_token, &6, &6);

    let token_admin_client = StellarAssetClient::new(&env, &burn_token);
    let mint_amount = 10_000_000_i128;
    mock_sac_mint_auth(&env, &burn_token, &owner, &caller, mint_amount);
    token_admin_client.mint(&caller, &mint_amount);

    // Approve the contract to spend caller's tokens
    let token_client = TokenClient::new(&env, &burn_token);
    let expiration_ledger = env.ledger().sequence() + 1000;
    mock_approve_auth(
        &env,
        &burn_token,
        &caller,
        &contract_id,
        i128::MAX,
        expiration_ledger,
    );
    token_client.approve(&caller, &contract_id, &i128::MAX, &expiration_ledger);

    let mint_recipient = BytesN::<32>::random(&env);
    let destination_caller = BytesN::from_array(&env, &[0u8; 32]);
    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    mock_deposit_for_burn_auth(
        &env,
        &contract_id,
        &caller,
        amount,
        destination_domain,
        &mint_recipient,
        &burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    // This should fail because the MessageTransmitter.send_message panics
    client.deposit_for_burn(
        &caller,
        &amount,
        &destination_domain,
        &mint_recipient,
        &burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

// =============================================================================
// deposit_for_burn Success Tests
// =============================================================================

#[test]
fn test_deposit_for_burn_success() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Verify DepositForBurn event
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    // Verify caller's balance decreased
    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);

    // Verify contract doesn't hold the tokens (they were burned, not just transferred)
    let contract_balance = token_client.balance(&ctx.contract_id);
    assert_eq!(contract_balance, 0);
}

#[test]
fn test_deposit_for_burn_emits_correct_event() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Verify DepositForBurn event was emitted with correct data
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &Bytes::new(&ctx.env), // empty hook_data for deposit_for_burn
    );
}

#[test]
fn test_deposit_for_burn_with_hook_emits_correct_event() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::<32>::random(&ctx.env);
    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;
    let hook_data = Bytes::from_array(&ctx.env, &[1, 2, 3, 4, 5]);

    mock_deposit_for_burn_with_hook_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &hook_data,
    );

    client.deposit_for_burn_with_hook(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
        &hook_data,
    );

    // Verify DepositForBurn event was emitted with correct data including hook_data
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &hook_data,
    );
}

#[test]
fn test_deposit_for_burn_with_hook_success() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::<32>::random(&ctx.env);
    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;
    let hook_data = Bytes::from_array(&ctx.env, &[1, 2, 3, 4]);

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_with_hook_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &hook_data,
    );

    client.deposit_for_burn_with_hook(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
        &hook_data,
    );

    // Verify DepositForBurn event with hook_data
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &hook_data,
    );

    // Verify caller's balance decreased
    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);

    // Verify contract doesn't hold the tokens (they were burned, not just transferred)
    let contract_balance = token_client.balance(&ctx.contract_id);
    assert_eq!(contract_balance, 0);
}

#[test]
fn test_deposit_for_burn_success_with_zero_max_fee() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 10_000_i128;
    let max_fee = 0_i128; // Zero fee should be valid when no min_fee is configured
    let min_finality_threshold = 500_u32;

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Verify DepositForBurn event
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);
}

#[test]
fn test_deposit_for_burn_success_with_min_fee_configured() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    // Configure min_fee_controller and set a min_fee
    let min_fee_controller_addr = Address::generate(&ctx.env);
    mock_set_min_fee_controller_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.owner,
        &min_fee_controller_addr,
    );
    client.set_min_fee_controller(&min_fee_controller_addr);

    let min_fee: i128 = 100;
    mock_set_min_fee_auth(
        &ctx.env,
        &ctx.contract_id,
        &min_fee_controller_addr,
        &ctx.burn_token,
        &min_fee,
    );
    client.set_min_fee(&ctx.burn_token, &min_fee);

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 100_000_i128;
    // min_fee_amount = 100_000 * 100 / 10_000_000 = 1
    let max_fee = 10_i128; // >= min_fee_amount
    let min_finality_threshold = 500_u32;

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Verify DepositForBurn event
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);
}

#[test]
fn test_deposit_for_burn_at_exact_burn_limit() {
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 1_000_000_i128; // exactly at the limit
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    // Mint more tokens to the caller to cover the full amount
    let token_admin_client = StellarAssetClient::new(&ctx.env, &ctx.burn_token);
    mock_sac_mint_auth(&ctx.env, &ctx.burn_token, &ctx.owner, &ctx.caller, amount);
    token_admin_client.mint(&ctx.caller, &amount);

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    // Execute deposit_for_burn at exact limit
    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Verify DepositForBurn event
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        amount,
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        max_fee,
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);
}

// =============================================================================
// handle_recv_finalized_message Success Tests
// =============================================================================

#[test]
fn test_handle_recv_finalized_message_success_no_fee() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    // Use a random bytes32 as mint_recipient (represents a contract address on destination)
    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_finalized_message should return true");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed, // full amount, no fee
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
fn test_handle_recv_finalized_message_success_with_fee() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 50_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_finalized_message should return true");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
fn test_handle_recv_finalized_message_emits_event() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 50_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    // Verify the function succeeded
    assert!(result, "handle_recv_finalized_message should return true");

    // Verify MintAndWithdraw event was emitted
    // Convert recipient_bytes32 to Address for event verification
    use soroban_sdk::address_payload::AddressPayload;
    let mint_recipient_payload = AddressPayload::ContractIdHash(recipient_bytes32.clone());
    let mint_recipient_address = Address::from_payload(&ctx.env, mint_recipient_payload);

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed, // amount minted to recipient
        &ctx.local_token,
        fee_executed,
    );
}

// =============================================================================
// handle_recv_unfinalized_message Success Tests
// =============================================================================

#[test]
fn test_handle_recv_unfinalized_message_success() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 50_i128;
    let finality_threshold = TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_unfinalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_unfinalized_message should return true");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
fn test_handle_recv_unfinalized_message_success_above_threshold() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 1000u32; // above minimum threshold

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_unfinalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_unfinalized_message should return true");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

// =============================================================================
// handle_recv_unfinalized_message Finality Threshold Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7116)")] // UnsupportedFinalityThreshold
fn test_handle_recv_unfinalized_message_fails_below_min_threshold() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 499u32; // below minimum (500)

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_unfinalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7116)")] // UnsupportedFinalityThreshold
fn test_handle_recv_unfinalized_message_fails_zero_threshold() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 0u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_unfinalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Message Transmitter Caller Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_handle_recv_finalized_message_fails_without_message_transmitter_auth() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    // Intentionally NOT mocking MessageTransmitter auth - this should fail
    // because only the MessageTransmitter contract is authorized to call this method
    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_handle_recv_unfinalized_message_fails_without_message_transmitter_auth() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    // Intentionally NOT mocking MessageTransmitter auth - this should fail
    // because only the MessageTransmitter contract is authorized to call this method
    client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Remote Token Messenger Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6403)")] // RemoteTokenMessengerNotRegistered
fn test_handle_recv_finalized_message_fails_unregistered_sender() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    // Use unregistered sender
    let unregistered_sender = BytesN::<32>::random(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &unregistered_sender,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &unregistered_sender, // not registered
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6403)")] // RemoteTokenMessengerNotRegistered
fn test_handle_recv_finalized_message_fails_wrong_domain() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;
    let wrong_domain = 999u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        wrong_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    // Use wrong domain (not domain 1)
    client.handle_recv_finalized_message(
        &wrong_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6403)")] // RemoteTokenMessengerNotRegistered
fn test_handle_recv_finalized_message_fails_zero_sender() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    let zero_sender = BytesN::from_array(&ctx.env, &[0u8; 32]);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &zero_sender,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &zero_sender,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Burn Message Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7111)")] // InvalidBurnMessageFormat
fn test_handle_recv_finalized_message_fails_empty_message() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let empty_message = Bytes::new(&ctx.env);
    let finality_threshold = 2000u32;

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &empty_message,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &empty_message,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7111)")] // InvalidBurnMessageFormat
fn test_handle_recv_finalized_message_fails_short_message() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    // Message too short (less than minimum 228 bytes)
    let short_message = Bytes::from_array(&ctx.env, &[0u8; 100]);
    let finality_threshold = 2000u32;

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &short_message,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &short_message,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7112)")] // InvalidBurnMessageVersion
fn test_handle_recv_finalized_message_fails_wrong_version() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    // Create a burn message with wrong version
    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    let burn_message = BurnMessageV2 {
        version: 999, // wrong version (expected is 1)
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: U256::from_u32(&ctx.env, 10_000),
        message_sender,
        max_fee: U256::from_u32(&ctx.env, 100),
        fee_executed: U256::from_u32(&ctx.env, 0),
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Fee Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7113)")] // FeeEqualsOrExceedsAmount
fn test_handle_recv_finalized_message_fails_fee_equals_amount() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 10_000_i128;
    let fee_executed = 10_000_i128; // fee equals amount
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7113)")] // FeeEqualsOrExceedsAmount
fn test_handle_recv_finalized_message_fails_fee_exceeds_amount() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 20_000_i128;
    let fee_executed = 15_000_i128; // fee exceeds amount
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7114)")] // FeeExceedsMaxFee
fn test_handle_recv_finalized_message_fails_fee_exceeds_max_fee() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 50_i128;
    let fee_executed = 100_i128; // fee exceeds max_fee
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
fn test_handle_recv_finalized_message_success_zero_fee() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128; // zero fee is valid
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result);

    // Verify MintAndWithdraw event with zero fee
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );

    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    // 4 events: approve, burn, mint (recipient), mint_and_withdraw (no fee path)
    events.assert_event_count(4);
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount, // full amount, no fee deducted
        &ctx.local_token,
        0, // fee_collected: 0
    );
}

#[test]
fn test_handle_recv_finalized_message_success_fee_equals_max_fee() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 100_i128; // fee == max_fee (boundary: check is >, not >=)
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "fee == max_fee should succeed (check is > not >=)");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7122)")] // AmountOverflow
fn test_handle_recv_finalized_message_fails_amount_exceeds_i128_max() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    // Create a burn message with amount > i128::MAX using raw BurnMessage
    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    // U256 value that exceeds i128::MAX (2^128)
    let overflow_amount = U256::from_u128(&ctx.env, u128::MAX);

    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: overflow_amount,
        message_sender,
        max_fee: U256::from_u32(&ctx.env, 0),
        fee_executed: U256::from_u32(&ctx.env, 0),
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7122)")] // AmountOverflow
fn test_handle_recv_finalized_message_fails_max_fee_exceeds_i128_max() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: U256::from_u128(&ctx.env, 10_000),
        message_sender,
        max_fee: U256::from_u128(&ctx.env, u128::MAX), // exceeds i128::MAX
        fee_executed: U256::from_u32(&ctx.env, 0),
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7122)")] // AmountOverflow
fn test_handle_recv_finalized_message_fails_fee_executed_exceeds_i128_max() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: U256::from_u128(&ctx.env, 10_000),
        message_sender,
        max_fee: U256::from_u32(&ctx.env, 100),
        fee_executed: U256::from_u128(&ctx.env, u128::MAX), // exceeds i128::MAX
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7119)")] // DecimalConversionFailed
fn test_handle_recv_finalized_message_fails_to_local_amount_overflow() {
    // local=7, canonical=6 (multiplier = 10)
    // With canonical_amount = i128::MAX, to_local_amount will overflow: i128::MAX * 10
    let ctx = setup_receive_test_with_decimals(7, 6);
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32,
        U256::from_u128(&ctx.env, i128::MAX as u128), // valid as U256, overflows in to_local_amount
        0,
        0,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7119)")] // DecimalConversionFailed
fn test_handle_recv_finalized_message_fails_to_local_max_fee_overflow() {
    // local=7, canonical=6 (multiplier = 10)
    let ctx = setup_receive_test_with_decimals(7, 6);
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    // amount is fine, but max_fee = i128::MAX overflows in to_local_amount
    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: U256::from_u128(&ctx.env, 10_000),
        message_sender,
        max_fee: U256::from_u128(&ctx.env, i128::MAX as u128),
        fee_executed: U256::from_u32(&ctx.env, 0),
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7119)")] // DecimalConversionFailed
fn test_handle_recv_finalized_message_fails_to_local_fee_overflow() {
    // local=7, canonical=6 (multiplier = 10)
    let ctx = setup_receive_test_with_decimals(7, 6);
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);
    let finality_threshold = 2000u32;

    let message_sender = BytesN::<32>::random(&ctx.env);
    let hook_data = Bytes::new(&ctx.env);

    // amount and max_fee are small (convert fine), but fee_executed = i128::MAX overflows
    // in to_local_amount (fee conversion is 3rd, after amount and max_fee conversions succeed)
    let burn_message = BurnMessageV2 {
        version: MESSAGE_BODY_VERSION,
        burn_token: ctx.remote_token.clone(),
        mint_recipient: recipient_bytes32,
        amount: U256::from_u128(&ctx.env, 10_000),
        message_sender,
        max_fee: U256::from_u32(&ctx.env, 100),
        fee_executed: U256::from_u128(&ctx.env, i128::MAX as u128),
        expiration_block: U256::from_u32(&ctx.env, 0),
        hook_data,
    };

    let message_body = burn_message.serialize(&ctx.env);

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Token Pair Validation Tests
// =============================================================================

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7115)")] // MintTokenNotSupported
fn test_handle_recv_finalized_message_fails_unlinked_token() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    // Use an unlinked remote token
    let unlinked_remote_token = BytesN::<32>::random(&ctx.env);

    let message_body = create_test_burn_message(
        &ctx.env,
        unlinked_remote_token, // not linked to any local token
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

// =============================================================================
// Message Expiration Tests
// =============================================================================

#[test]
fn test_handle_recv_finalized_message_success_no_expiration() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;
    let expiration_block = U256::from_u32(&ctx.env, 0); // 0 means no expiration

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "Message with no expiration should succeed");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
fn test_handle_recv_finalized_message_success_future_expiration() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    // Set expiration to a future ledger
    let current_ledger = ctx.env.ledger().sequence();
    let expiration_block = U256::from_u32(&ctx.env, current_ledger + 100); // expires 100 ledgers in the future

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "Message with future expiration should succeed");

    // Verify MintAndWithdraw event
    let mint_recipient_address = Address::from_payload(
        &ctx.env,
        AddressPayload::ContractIdHash(recipient_bytes32.clone()),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_mint_and_withdraw(
        &mint_recipient_address,
        amount - fee_executed,
        &ctx.local_token,
        fee_executed,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7117)")] // MessageExpired
fn test_handle_recv_finalized_message_fails_expired_message() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    // Advance ledger to a non-zero value so we can set an expired block
    ctx.env.ledger().with_mut(|l| l.sequence_number = 1000);

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    // Set expiration to a past ledger (current is 1000, expire at 999)
    let expiration_block = U256::from_u32(&ctx.env, 999);

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7117)")] // MessageExpired
fn test_handle_recv_finalized_message_fails_expiration_at_current_ledger() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    // Advance ledger to a non-zero value
    ctx.env.ledger().with_mut(|l| l.sequence_number = 1000);

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    // Set expiration to exactly the current ledger (boundary case - should fail)
    // expiration_block <= current_ledger means expired
    let expiration_block = U256::from_u32(&ctx.env, 1000);

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7117)")] // MessageExpired
fn test_handle_recv_unfinalized_message_fails_expired_message() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    // Advance ledger to a non-zero value so we can set an expired block
    ctx.env.ledger().with_mut(|l| l.sequence_number = 1000);

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 50_i128;
    let finality_threshold = TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD;

    // Set expiration to a past ledger (current is 1000, expire at 999)
    let expiration_block = U256::from_u32(&ctx.env, 999);

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_unfinalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_unfinalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7111)")] // InvalidBurnMessageV2Format
fn test_handle_recv_finalized_message_fails_expiration_block_exceeds_u32_max() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    let expiration_block = U256::from_u128(&ctx.env, (u32::MAX as u128) + 1);

    let message_body = create_test_burn_message_with_expiration(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
        expiration_block,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
fn test_handle_recv_finalized_message_mint_recipient_is_token_address() {
    let ctx = setup_receive_test();
    let client = ctx.client();

    let local_token_bytes32 = match ctx.local_token.to_payload() {
        Some(AddressPayload::ContractIdHash(hash)) => hash,
        _ => panic!("Expected ContractIdHash for local_token"),
    };

    let amount = 10_000_i128;
    let max_fee = 100_i128;
    let fee_executed = 0_i128;
    let finality_threshold = 2000u32;

    // Create burn message with the local_token (SAC) as the mint_recipient
    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        local_token_bytes32.clone(),
        U256::from_u128(&ctx.env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_finalized_message should return true");

    // Verify the SAC (local_token) now holds the minted tokens and they were not explicitly burned
    let token_client = TokenClient::new(&ctx.env, &ctx.local_token);
    let sac_balance = token_client.balance(&ctx.local_token);

    assert_eq!(sac_balance, amount, "SAC should hold the minted tokens");
}

// =============================================================================
// Decimal Conversion Tests
// =============================================================================

#[test]
fn test_deposit_for_burn_with_decimal_conversion() {
    // Stellar USDC (7 decimals) → CCTP (6 decimals)
    // User deposits 123457 (7 decimals) = 0.0123457 USDC
    // After normalization: 123450 (7 decimals) = 0.012345 USDC (dust of 7 stays with user)
    // Remote amount: 12345 (6 decimals) = 0.012345 USDC
    let ctx = setup_deposit_test_with_decimals(7, 6);
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 123457_i128; // User's original amount in 7 decimals (within burn limit of 1M)
    let max_fee = 1000_i128; // in local decimals
    let min_finality_threshold = 500_u32;

    // Check initial token balance
    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    // Mock auth for deposit_for_burn
    mock_deposit_for_burn_with_decimal_conversion_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount, // User specifies full amount with dust
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Event emits remote amounts for cross-chain consistency
    let remote_burn_amount = 12345_i128; // remote decimals (6)
    let local_burn_amount = 123450_i128; // local decimals (7), normalized
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        remote_burn_amount, // remote_burn_amount: 12345
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        100, // remote_max_fee: 1000 / 10 = 100
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    // Verify the burn message sent to message_transmitter contains the correct REMOTE amount
    let msg_events = EventAssertion::new(&ctx.env, ctx.message_transmitter.clone());
    let full_message = msg_events.expect_message_sent();

    // Extract the message body (burn message) from the full CCTP message
    let burn_message = MessageV2::get_message_body(&full_message);

    // Parse and verify the burn message contains the correct REMOTE amounts
    let burn_message_amount = BurnMessageV2::get_amount(&ctx.env, &burn_message)
        .expect("Failed to parse burn message amount");
    let burn_message_max_fee = BurnMessageV2::get_max_fee(&ctx.env, &burn_message)
        .expect("Failed to parse burn message max_fee");

    // The burn message should contain REMOTE amounts (6 decimals)
    assert_eq!(
        burn_message_amount,
        U256::from_u128(&ctx.env, 12345),
        "Burn message should contain remote amount (12345), not local amount (123450)"
    );
    assert_eq!(
        burn_message_max_fee,
        U256::from_u128(&ctx.env, 100),
        "Burn message should contain remote max_fee (100), not local max_fee (1000)"
    );

    // Now verify caller's balance: should only be reduced by the normalized LOCAL amount (123450)
    // Dust (7) should remain with the user
    let final_balance = token_client.balance(&ctx.caller);
    let dust = amount - local_burn_amount;
    assert_eq!(dust, 7);
    assert_eq!(final_balance, initial_balance - local_burn_amount);
}

#[test]
fn test_deposit_for_burn_decimal_conversion_preserves_dust() {
    // Test that dust (amounts that can't be represented in remote decimals) stays with user
    let ctx = setup_deposit_test_with_decimals(7, 6);
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 999999_i128; // Maximum dust scenario (within burn limit of 1M)
    let max_fee = 1000_i128;
    let min_finality_threshold = 500_u32;

    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    // Normalized amount: 999990 (dust of 9 stays with user)
    mock_deposit_for_burn_with_decimal_conversion_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // Event should emit canonical (remote) amount, not local amount with dust
    // local 999990 → canonical 99999 (/ 10)
    // local max_fee 1000 → canonical 100 (/ 10)
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    events.assert_deposit_for_burn(
        &ctx.burn_token,
        99999, // canonical amount: 999990 / 10
        &ctx.caller,
        &mint_recipient,
        ctx.destination_domain,
        &ctx.remote_token_messenger,
        &destination_caller,
        100, // canonical max_fee: 1000 / 10
        min_finality_threshold,
        &Bytes::new(&ctx.env),
    );

    let final_balance = token_client.balance(&ctx.caller);
    // Dust of 9 should remain with user
    assert_eq!(final_balance, initial_balance - 999990);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7118)")] // BurnAmountTooSmall
fn test_deposit_for_burn_fails_when_amount_normalizes_to_zero() {
    // Test: Amount is so small that after normalization it becomes 0
    let ctx = setup_deposit_test_with_decimals(7, 6);
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 9_i128; // Less than 10, so normalizes to 0 (for 7→6 decimal conversion)
    let max_fee = 1_i128;
    let min_finality_threshold = 500_u32;

    // Mock auth - will fail before transfer due to BurnAmountTooSmall
    mock_deposit_for_burn_with_decimal_conversion_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount,
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7120)")] // TokenDecimalConfigNotSet
fn test_deposit_for_burn_fails_without_decimal_config() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let denylister = Address::generate(&env);
    let caller = Address::generate(&env);

    let message_transmitter = env.register(crate::test_utils::MockMessageTransmitter, ());

    let destination_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);

    let min_fee_controller = Address::generate(&env);

    let contract_id = env.register(
        crate::contract::TokenMessengerMinterV2Contract,
        (crate::contract::TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer,
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, destination_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );

    let burn_token = crate::test_utils::create_test_token(&env, &owner);

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);

    // Configure burn limit but NOT decimal config
    mock_set_max_burn_amount_per_message_auth(
        &env,
        &contract_id,
        &token_controller,
        &burn_token,
        1_000_000_i128,
    );
    client.set_max_burn_amount_per_message(&burn_token, &1_000_000_i128);

    let token_admin_client = StellarAssetClient::new(&env, &burn_token);
    crate::test_utils::mock_sac_mint_auth(&env, &burn_token, &owner, &caller, 10_000_000_i128);
    token_admin_client.mint(&caller, &10_000_000_i128);

    // Approve the contract to spend caller's tokens
    let token_client = TokenClient::new(&env, &burn_token);
    let expiration_ledger = env.ledger().sequence() + 1000;
    crate::test_utils::mock_approve_auth(
        &env,
        &burn_token,
        &caller,
        &contract_id,
        i128::MAX,
        expiration_ledger,
    );
    token_client.approve(&caller, &contract_id, &i128::MAX, &expiration_ledger);

    let mint_recipient = BytesN::<32>::random(&env);
    let destination_caller = BytesN::from_array(&env, &[0u8; 32]);
    let amount = 123456_i128;
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    mock_deposit_for_burn_auth(
        &env,
        &contract_id,
        &caller,
        amount,
        destination_domain,
        &mint_recipient,
        &burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &caller,
        &amount,
        &destination_domain,
        &mint_recipient,
        &burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );
}

#[test]
fn test_receive_with_decimal_conversion() {
    // Receiving USDC from another chain (6 decimals) → Stellar (7 decimals)
    // Remote amount: 123456 (6 decimals) = 0.123456 USDC
    // Local amount: 1234560 (7 decimals) = 0.123456 USDC
    let ctx = setup_receive_test_with_decimals(7, 6);
    let client = ctx.client();

    let recipient_bytes32 = BytesN::<32>::random(&ctx.env);

    // Message contains amounts in CANONICAL decimals (6)
    let canonical_amount = 123456_i128; // 0.123456 USDC in 6 decimals
    let max_fee = 100_i128;
    let fee_executed = 10_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &ctx.env,
        ctx.remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&ctx.env, canonical_amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.message_transmitter,
        ctx.remote_domain,
        &ctx.remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    let result = client.handle_recv_finalized_message(
        &ctx.remote_domain,
        &ctx.remote_token_messenger,
        &finality_threshold,
        &message_body,
    );

    assert!(result, "handle_recv_finalized_message should return true");

    // Event assertion must happen immediately after the emitting call
    let recipient = Address::from_payload(
        &ctx.env,
        soroban_sdk::address_payload::AddressPayload::ContractIdHash(recipient_bytes32),
    );
    let mut events = EventAssertion::new(&ctx.env, ctx.contract_id.clone());
    // 6 events: approve, burn, mint (recipient), burn, mint (fee_recipient), mint_and_withdraw
    events.assert_event_count(6);
    events.assert_mint_and_withdraw(
        &recipient,
        canonical_amount - fee_executed, // 123456 - 10 = 123446 (canonical, NOT local 1234460)
        &ctx.local_token,
        fee_executed, // 10 (canonical, NOT local 100)
    );

    // Verify the minted amount is in LOCAL decimals (7)
    // canonical_amount (123456) → local_amount (1234560)
    // canonical_fee (10) → local_fee (100)
    // minted = local_amount - local_fee = 1234560 - 100 = 1234460
    let token_client = TokenClient::new(&ctx.env, &ctx.local_token);
    let recipient_balance = token_client.balance(&recipient);

    let expected_minted = (canonical_amount - fee_executed) * 10; // 123446 * 10 = 1234460
    assert_eq!(recipient_balance, expected_minted);

    // Fee recipient should receive local_fee
    let fee_recipient_balance = token_client.balance(&ctx.fee_recipient);
    let expected_fee = fee_executed * 10; // 10 * 10 = 100
    assert_eq!(fee_recipient_balance, expected_fee);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7120)")] // TokenDecimalConfigNotSet
fn test_receive_fails_without_decimal_config() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let denylister = Address::generate(&env);

    let message_transmitter = env.register(crate::test_utils::MockMessageTransmitter, ());

    let remote_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);
    let min_fee_controller = Address::generate(&env);

    let contract_id = env.register(
        crate::contract::TokenMessengerMinterV2Contract,
        (crate::contract::TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer,
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, remote_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );

    let local_token = crate::test_utils::create_test_token(&env, &contract_id);
    let remote_token = BytesN::<32>::random(&env);

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);

    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &token_controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    let recipient_bytes32 = BytesN::<32>::random(&env);

    let amount = 123456_i128;
    let max_fee = 100_i128;
    let fee_executed = 10_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &env,
        remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &env,
        &contract_id,
        &message_transmitter,
        remote_domain,
        &remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    // This should fail because no decimal config is set
    client.handle_recv_finalized_message(
        &remote_domain,
        &remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7121)")] // SwapMinterConfigNotSet
fn test_receive_fails_without_swap_minter_config() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let pauser = Address::generate(&env);
    let rescuer = Address::generate(&env);
    let token_controller = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    let denylister = Address::generate(&env);

    let message_transmitter = env.register(crate::test_utils::MockMessageTransmitter, ());

    let remote_domain = 1u32;
    let remote_token_messenger = BytesN::<32>::random(&env);
    let min_fee_controller = Address::generate(&env);

    let contract_id = env.register(
        crate::contract::TokenMessengerMinterV2Contract,
        (crate::contract::TokenMessengerMinterV2ContractInitParams {
            owner: owner.clone(),
            pauser: pauser.clone(),
            rescuer,
            token_controller: token_controller.clone(),
            admin: admin.clone(),
            fee_recipient: fee_recipient.clone(),
            min_fee_controller: min_fee_controller.clone(),
            denylister: denylister.clone(),
            message_transmitter: message_transmitter.clone(),
            message_body_version: MESSAGE_BODY_VERSION,
            remote_domains: vec![&env, remote_domain],
            remote_token_messengers: vec![&env, remote_token_messenger.clone()],
        },),
    );

    let local_token = crate::test_utils::create_test_token(&env, &contract_id);
    let remote_token = BytesN::<32>::random(&env);

    let client = TokenMessengerMinterV2ContractClient::new(&env, &contract_id);

    // Link token pair
    mock_link_token_pair_auth(
        &env,
        &contract_id,
        &token_controller,
        &local_token,
        remote_domain,
        &remote_token,
    );
    client.link_token_pair(&local_token, &remote_domain, &remote_token);

    // Set decimal config (required before swap minter config check)
    mock_set_token_decimal_config_auth(&env, &contract_id, &token_controller, &local_token, 6, 6);
    client.set_token_decimal_config(&local_token, &6, &6);

    // NOTE: We intentionally do NOT set swap_minter_config

    let recipient_bytes32 = BytesN::<32>::random(&env);

    let amount = 123456_i128;
    let max_fee = 100_i128;
    let fee_executed = 10_i128;
    let finality_threshold = 2000u32;

    let message_body = create_test_burn_message(
        &env,
        remote_token.clone(),
        recipient_bytes32.clone(),
        U256::from_u128(&env, amount as u128),
        max_fee,
        fee_executed,
    );

    mock_handle_recv_finalized_message_auth(
        &env,
        &contract_id,
        &message_transmitter,
        remote_domain,
        &remote_token_messenger,
        finality_threshold,
        &message_body,
    );

    // This should fail because no swap minter config is set
    client.handle_recv_finalized_message(
        &remote_domain,
        &remote_token_messenger,
        &finality_threshold,
        &message_body,
    );
}

#[test]
fn test_deposit_for_burn_with_equal_decimals() {
    // Default decimal config (6,6) is set by setup_deposit_test
    let ctx = setup_deposit_test();
    let client = ctx.client();

    let mint_recipient = BytesN::<32>::random(&ctx.env);
    let destination_caller = BytesN::from_array(&ctx.env, &[0u8; 32]);
    let amount = 123456_i128; // Within burn limit of 1M
    let max_fee = 100_i128;
    let min_finality_threshold = 500_u32;

    let token_client = TokenClient::new(&ctx.env, &ctx.burn_token);
    let initial_balance = token_client.balance(&ctx.caller);

    mock_deposit_for_burn_auth(
        &ctx.env,
        &ctx.contract_id,
        &ctx.caller,
        amount, // No normalization with equal decimals
        ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        max_fee,
        min_finality_threshold,
    );

    client.deposit_for_burn(
        &ctx.caller,
        &amount,
        &ctx.destination_domain,
        &mint_recipient,
        &ctx.burn_token,
        &destination_caller,
        &max_fee,
        &min_finality_threshold,
    );

    // With equal decimals, full amount should be burned
    let final_balance = token_client.balance(&ctx.caller);
    assert_eq!(final_balance, initial_balance - amount);
}
