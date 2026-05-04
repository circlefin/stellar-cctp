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

//! Event assertion extensions for cctp-roles.
//!
//! This module provides the `CctpEventAssertions` trait which adds
//! CCTP-specific event assertion methods to `EventAssertion`.
//!
//! # Example
//!
//! ```ignore
//! use event_assertion::EventAssertion;
//! use cctp_roles::test_utils::CctpEventAssertions;
//!
//! let mut events = EventAssertion::new(&env, contract.address.clone());
//! events.assert_attester_enabled(&attester);
//! events.assert_message_sent();
//! ```

use event_assertion::EventAssertion;
use soroban_sdk::{Address, Bytes, BytesN, IntoVal, MuxedAddress, Symbol, TryIntoVal, Val};

/// Extension trait for CCTP-specific event assertions.
///
/// Import this trait to get access to attestable, denylistable, fee_recipient,
/// min_fee_controller, remote_token_messenger, token_controller, message transmitter,
/// and token messenger event assertion methods on `EventAssertion`.
pub trait CctpEventAssertions {
    // Attestable events
    fn assert_signature_threshold_updated(
        &mut self,
        expected_old_threshold: Option<u32>,
        expected_new_threshold: u32,
    );
    fn assert_attester_enabled(&mut self, expected_attester: &BytesN<20>);
    fn assert_attester_disabled(&mut self, expected_attester: &BytesN<20>);
    fn assert_attester_manager_updated(
        &mut self,
        expected_previous_manager: &Option<Address>,
        expected_new_manager: &Address,
    );

    // Denylistable events
    fn assert_denylister_changed(
        &mut self,
        expected_old_denylister: Option<&Address>,
        expected_new_denylister: &Address,
    );
    fn assert_denylisted(&mut self, expected_account: &Address);
    fn assert_un_denylisted(&mut self, expected_account: &Address);

    // Fee recipient events
    fn assert_fee_recipient_set(&mut self, expected_fee_recipient: &Address);

    // Min fee controller events
    fn assert_min_fee_controller_set(&mut self, expected_controller: &Address);
    fn assert_min_fee_set(&mut self, expected_burn_token: &Address, expected_min_fee: &i128);

    // Remote token messenger events
    fn assert_remote_token_messenger_added(
        &mut self,
        expected_domain: u32,
        expected_token_messenger: &BytesN<32>,
    );
    fn assert_remote_token_messenger_removed(
        &mut self,
        expected_domain: u32,
        expected_token_messenger: &BytesN<32>,
    );

    // Token controller events
    fn assert_set_token_controller(&mut self, expected_token_controller: &Address);
    fn assert_token_pair_linked(
        &mut self,
        local_token: &Address,
        remote_domain: u32,
        remote_token: &BytesN<32>,
    );
    fn assert_token_pair_unlinked(
        &mut self,
        local_token: &Address,
        remote_domain: u32,
        remote_token: &BytesN<32>,
    );
    fn assert_set_burn_limit_per_message(&mut self, local_token: &Address, burn_limit: i128);
    fn assert_token_decimal_config_added(
        &mut self,
        token: &Address,
        local_decimals: u32,
        canonical_decimals: u32,
    );
    fn assert_swap_minter_config_set(
        &mut self,
        token: &Address,
        swap_minter: &Address,
        allow_asset: &Address,
    );
    fn assert_swap_minter_config_removed(
        &mut self,
        token: &Address,
        swap_minter: &Address,
        allow_asset: &Address,
    );

    // Message transmitter events
    fn assert_max_message_body_size_updated(&mut self, expected_size: &u32);
    fn assert_message_sent(&mut self);
    fn assert_message_received(&mut self);
    fn extract_message_sent(&self) -> Option<Bytes>;
    fn expect_message_sent(&self) -> Bytes;

    // CCTP forwarder events
    fn assert_mint_and_forward(
        &mut self,
        forward_recipient: &MuxedAddress,
        token: &Address,
        amount: i128,
    );

    // Token messenger events
    #[allow(clippy::too_many_arguments)]
    fn assert_deposit_for_burn(
        &mut self,
        burn_token: &Address,
        amount: i128,
        depositor: &Address,
        mint_recipient: &BytesN<32>,
        destination_domain: u32,
        destination_token_messenger: &BytesN<32>,
        destination_caller: &BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
        hook_data: &Bytes,
    );
    fn assert_mint_and_withdraw(
        &mut self,
        mint_recipient: &Address,
        amount: i128,
        mint_token: &Address,
        fee_collected: i128,
    );
}

impl<'a> CctpEventAssertions for EventAssertion<'a> {
    fn assert_signature_threshold_updated(
        &mut self,
        expected_old_threshold: Option<u32>,
        expected_new_threshold: u32,
    ) {
        let event = self.find_event_by_symbol("signature_threshold_updated");

        assert!(
            event.is_some(),
            "signature_threshold_updated event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "signature_threshold_updated event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "signature_threshold_updated event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "signature_threshold_updated"),
            "Event topic should be 'signature_threshold_updated'"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let old_threshold_val = event_data
            .get(Symbol::new(self.env(), "old_signature_threshold"))
            .expect("signature_threshold_updated event missing old_signature_threshold field");
        let old_threshold: Option<u32> = old_threshold_val.try_into_val(self.env()).ok();

        let new_threshold_val = event_data
            .get(Symbol::new(self.env(), "new_signature_threshold"))
            .expect("signature_threshold_updated event missing new_signature_threshold field");
        let new_threshold: u32 = new_threshold_val.into_val(self.env());

        assert_eq!(
            old_threshold, expected_old_threshold,
            "signature_threshold_updated event has wrong old_signature_threshold"
        );
        assert_eq!(
            new_threshold, expected_new_threshold,
            "signature_threshold_updated event has wrong new_signature_threshold"
        );
    }

    fn assert_attester_enabled(&mut self, expected_attester: &BytesN<20>) {
        let events = self.find_all_events_by_symbol("attester_enabled");
        let event = events.into_iter().find(|(contract, topics, _)| {
            if contract != self.contract() || topics.len() < 2 {
                return false;
            }
            let event_attester: BytesN<20> = topics.get_unchecked(1).into_val(self.env());
            &event_attester == expected_attester
        });

        assert!(
            event.is_some(),
            "attester_enabled event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "attester_enabled event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "attester_enabled event should have 2 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "attester_enabled"),
            "Event topic should be 'attester_enabled'"
        );

        let event_attester: BytesN<20> = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_attester, expected_attester,
            "attester_enabled event has wrong attester"
        );
    }

    fn assert_attester_disabled(&mut self, expected_attester: &BytesN<20>) {
        let event = self.find_event_by_symbol("attester_disabled");

        assert!(
            event.is_some(),
            "attester_disabled event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "attester_disabled event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "attester_disabled event should have 2 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "attester_disabled"),
            "Event topic should be 'attester_disabled'"
        );

        let event_attester: BytesN<20> = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_attester, expected_attester,
            "attester_disabled event has wrong attester"
        );
    }

    fn assert_attester_manager_updated(
        &mut self,
        expected_previous_manager: &Option<Address>,
        expected_new_manager: &Address,
    ) {
        let event = self.find_event_by_symbol("attester_manager_updated");

        assert!(
            event.is_some(),
            "attester_manager_updated event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "attester_manager_updated event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            3,
            "attester_manager_updated event should have 3 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "attester_manager_updated"),
            "Event topic should be 'attester_manager_updated'"
        );

        let event_previous_manager: Option<Address> = topics.get_unchecked(1).into_val(self.env());
        let event_new_manager: Address = topics.get_unchecked(2).into_val(self.env());

        assert_eq!(
            &event_previous_manager, expected_previous_manager,
            "attester_manager_updated event has wrong previous_attester_manager"
        );
        assert_eq!(
            &event_new_manager, expected_new_manager,
            "attester_manager_updated event has wrong new_attester_manager"
        );
    }

    fn assert_denylister_changed(
        &mut self,
        expected_old_denylister: Option<&Address>,
        expected_new_denylister: &Address,
    ) {
        let event = self.find_event_by_symbol("denylister_changed");

        assert!(
            event.is_some(),
            "denylister_changed event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "denylister_changed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            3,
            "denylister_changed event should have 3 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "denylister_changed"),
            "Event topic should be denylister_changed"
        );

        let event_old_denylister: Option<Address> = topics.get_unchecked(1).into_val(self.env());

        match (expected_old_denylister, event_old_denylister) {
            (Some(expected), Some(actual)) => {
                assert_eq!(
                    expected, &actual,
                    "denylister_changed event has wrong old_denylister"
                )
            }
            (None, None) => (),
            _ => panic!("denylister_changed event old_denylister mismatch"),
        }

        let event_new_denylister: Address = topics.get_unchecked(2).into_val(self.env());
        assert_eq!(
            &event_new_denylister, expected_new_denylister,
            "denylister_changed event has wrong new_denylister"
        );
    }

    fn assert_denylisted(&mut self, expected_account: &Address) {
        let event = self.find_event_by_symbol("denylisted");

        assert!(event.is_some(), "denylisted event not found in event log");

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "denylisted event from wrong contract"
        );

        assert_eq!(topics.len(), 2, "denylisted event should have 2 topics");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "denylisted"),
            "Event topic should be 'denylisted'"
        );

        let event_account: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_account, expected_account,
            "denylisted event has wrong account"
        );
    }

    fn assert_un_denylisted(&mut self, expected_account: &Address) {
        let event = self.find_event_by_symbol("un_denylisted");

        assert!(
            event.is_some(),
            "un_denylisted event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "un_denylisted event from wrong contract"
        );

        assert_eq!(topics.len(), 2, "un_denylisted event should have 2 topics");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "un_denylisted"),
            "Event topic should be 'un_denylisted'"
        );

        let event_account: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_account, expected_account,
            "un_denylisted event has wrong account"
        );
    }

    fn assert_fee_recipient_set(&mut self, expected_fee_recipient: &Address) {
        let event = self.find_event_by_symbol("fee_recipient_set");

        assert!(
            event.is_some(),
            "fee_recipient_set event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "fee_recipient_set event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "fee_recipient_set event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "fee_recipient_set"),
            "Event topic should be fee_recipient_set"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_fee_recipient: Address = event_data
            .get(Symbol::new(self.env(), "fee_recipient"))
            .expect("fee_recipient_set event missing fee_recipient field")
            .into_val(self.env());
        assert_eq!(
            &event_fee_recipient, expected_fee_recipient,
            "fee_recipient_set event has wrong fee recipient"
        );
    }

    fn assert_min_fee_controller_set(&mut self, expected_controller: &Address) {
        let event = self.find_event_by_symbol("min_fee_controller_set");

        assert!(
            event.is_some(),
            "min_fee_controller_set event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "min_fee_controller_set event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "min_fee_controller_set event should have 2 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "min_fee_controller_set"),
            "Event topic should be min_fee_controller_set"
        );

        let event_controller: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_controller, expected_controller,
            "min_fee_controller_set event has wrong controller"
        );
    }

    fn assert_min_fee_set(&mut self, expected_burn_token: &Address, expected_min_fee: &i128) {
        let event = self.find_event_by_symbol("min_fee_set");

        assert!(event.is_some(), "min_fee_set event not found in event log");

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "min_fee_set event from wrong contract"
        );

        assert_eq!(topics.len(), 2, "min_fee_set event should have 2 topics");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "min_fee_set"),
            "Event topic should be min_fee_set"
        );

        let event_burn_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_burn_token, expected_burn_token,
            "min_fee_set event has wrong burn_token"
        );

        let event_data: soroban_sdk::Map<Symbol, i128> = data.into_val(self.env());
        let event_min_fee = event_data
            .get(Symbol::new(self.env(), "min_fee"))
            .expect("min_fee_set event missing min_fee field");
        assert_eq!(
            &event_min_fee, expected_min_fee,
            "min_fee_set event has wrong min_fee"
        );
    }

    fn assert_remote_token_messenger_added(
        &mut self,
        expected_domain: u32,
        expected_token_messenger: &BytesN<32>,
    ) {
        let event = self.find_event_by_symbol("remote_token_messenger_added");

        assert!(
            event.is_some(),
            "remote_token_messenger_added event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "remote_token_messenger_added event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "remote_token_messenger_added event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "remote_token_messenger_added"),
            "Event topic should be 'remote_token_messenger_added'"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_domain: u32 = event_data
            .get(Symbol::new(self.env(), "domain"))
            .expect("remote_token_messenger_added event missing domain field")
            .into_val(self.env());
        assert_eq!(
            event_domain, expected_domain,
            "remote_token_messenger_added event has wrong domain"
        );

        let event_token_messenger: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "token_messenger"))
            .expect("remote_token_messenger_added event missing token_messenger field")
            .into_val(self.env());
        assert_eq!(
            &event_token_messenger, expected_token_messenger,
            "remote_token_messenger_added event has wrong token_messenger"
        );
    }

    fn assert_remote_token_messenger_removed(
        &mut self,
        expected_domain: u32,
        expected_token_messenger: &BytesN<32>,
    ) {
        let event = self.find_event_by_symbol("remote_token_messenger_removed");

        assert!(
            event.is_some(),
            "remote_token_messenger_removed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "remote_token_messenger_removed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "remote_token_messenger_removed event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "remote_token_messenger_removed"),
            "Event topic should be 'remote_token_messenger_removed'"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_domain: u32 = event_data
            .get(Symbol::new(self.env(), "domain"))
            .expect("remote_token_messenger_removed event missing domain field")
            .into_val(self.env());
        assert_eq!(
            event_domain, expected_domain,
            "remote_token_messenger_removed event has wrong domain"
        );

        let event_token_messenger: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "token_messenger"))
            .expect("remote_token_messenger_removed event missing token_messenger field")
            .into_val(self.env());
        assert_eq!(
            &event_token_messenger, expected_token_messenger,
            "remote_token_messenger_removed event has wrong token_messenger"
        );
    }

    fn assert_set_token_controller(&mut self, expected_token_controller: &Address) {
        let event = self.find_event_by_symbol("set_token_controller");

        assert!(
            event.is_some(),
            "set_token_controller event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "set_token_controller event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "set_token_controller event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "set_token_controller"),
            "Event topic should be set_token_controller"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        let event_token_controller: Address = event_data
            .get(Symbol::new(self.env(), "token_controller"))
            .expect("set_token_controller event missing token_controller field")
            .into_val(self.env());
        assert_eq!(
            &event_token_controller, expected_token_controller,
            "set_token_controller event has wrong address"
        );
    }

    fn assert_token_pair_linked(
        &mut self,
        local_token: &Address,
        remote_domain: u32,
        remote_token: &BytesN<32>,
    ) {
        let event = self.find_event_by_symbol("token_pair_linked");

        assert!(
            event.is_some(),
            "token_pair_linked event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "token_pair_linked event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "token_pair_linked event should have 1 topic"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_local_token: Address = event_data
            .get(Symbol::new(self.env(), "local_token"))
            .expect("token_pair_linked event missing local_token field")
            .into_val(self.env());
        assert_eq!(
            &event_local_token, local_token,
            "token_pair_linked event has wrong local_token"
        );

        let event_remote_domain: u32 = event_data
            .get(Symbol::new(self.env(), "remote_domain"))
            .expect("token_pair_linked event missing remote_domain field")
            .into_val(self.env());
        assert_eq!(
            event_remote_domain, remote_domain,
            "token_pair_linked event has wrong remote_domain"
        );

        let event_remote_token: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "remote_token"))
            .expect("token_pair_linked event missing remote_token field")
            .into_val(self.env());
        assert_eq!(
            &event_remote_token, remote_token,
            "token_pair_linked event has wrong remote_token"
        );
    }

    fn assert_token_pair_unlinked(
        &mut self,
        local_token: &Address,
        remote_domain: u32,
        remote_token: &BytesN<32>,
    ) {
        let event = self.find_event_by_symbol("token_pair_unlinked");

        assert!(
            event.is_some(),
            "token_pair_unlinked event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "token_pair_unlinked event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "token_pair_unlinked event should have 1 topic"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_local_token: Address = event_data
            .get(Symbol::new(self.env(), "local_token"))
            .expect("token_pair_unlinked event missing local_token field")
            .into_val(self.env());
        assert_eq!(
            &event_local_token, local_token,
            "token_pair_unlinked event has wrong local_token"
        );

        let event_remote_domain: u32 = event_data
            .get(Symbol::new(self.env(), "remote_domain"))
            .expect("token_pair_unlinked event missing remote_domain field")
            .into_val(self.env());
        assert_eq!(
            event_remote_domain, remote_domain,
            "token_pair_unlinked event has wrong remote_domain"
        );

        let event_remote_token: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "remote_token"))
            .expect("token_pair_unlinked event missing remote_token field")
            .into_val(self.env());
        assert_eq!(
            &event_remote_token, remote_token,
            "token_pair_unlinked event has wrong remote_token"
        );
    }

    fn assert_set_burn_limit_per_message(&mut self, local_token: &Address, burn_limit: i128) {
        let event = self.find_event_by_symbol("set_burn_limit_per_message");

        assert!(
            event.is_some(),
            "set_burn_limit_per_message event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "set_burn_limit_per_message event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "set_burn_limit_per_message event should have 2 topics"
        );

        let event_local_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_local_token, local_token,
            "set_burn_limit_per_message event has wrong local_token"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        let event_burn_limit: i128 = event_data
            .get(Symbol::new(self.env(), "burn_limit_per_message"))
            .expect("set_burn_limit_per_message event missing burn_limit_per_message field")
            .into_val(self.env());
        assert_eq!(
            event_burn_limit, burn_limit,
            "set_burn_limit_per_message event has wrong burn_limit_per_message"
        );
    }

    fn assert_token_decimal_config_added(
        &mut self,
        token: &Address,
        local_decimals: u32,
        canonical_decimals: u32,
    ) {
        let event = self.find_event_by_symbol("token_decimal_config_added");

        assert!(
            event.is_some(),
            "token_decimal_config_added event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "token_decimal_config_added event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "token_decimal_config_added event should have 2 topics"
        );

        let event_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_token, token,
            "token_decimal_config_added event has wrong token"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        let config_val = event_data
            .get(Symbol::new(self.env(), "token_decimal_config"))
            .expect("token_decimal_config_added event missing token_decimal_config field");
        let config_map: soroban_sdk::Map<Symbol, u32> = config_val.into_val(self.env());

        let event_local_decimals = config_map
            .get(Symbol::new(self.env(), "local_decimals"))
            .expect("token_decimal_config missing local_decimals field");
        let event_canonical_decimals = config_map
            .get(Symbol::new(self.env(), "canonical_decimals"))
            .expect("token_decimal_config missing canonical_decimals field");

        assert_eq!(
            event_local_decimals, local_decimals,
            "token_decimal_config_added event has wrong local_decimals"
        );
        assert_eq!(
            event_canonical_decimals, canonical_decimals,
            "token_decimal_config_added event has wrong canonical_decimals"
        );
    }

    fn assert_swap_minter_config_set(
        &mut self,
        token: &Address,
        swap_minter: &Address,
        allow_asset: &Address,
    ) {
        let event = self.find_event_by_symbol("swap_minter_config_set");

        assert!(
            event.is_some(),
            "swap_minter_config_set event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "swap_minter_config_set event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "swap_minter_config_set event should have 2 topics"
        );

        let event_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_token, token,
            "swap_minter_config_set event has wrong token"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        let config_val = event_data
            .get(Symbol::new(self.env(), "swap_minter_config"))
            .expect("swap_minter_config_set event missing swap_minter_config field");
        let config_map: soroban_sdk::Map<Symbol, Address> = config_val.into_val(self.env());

        let event_swap_minter = config_map
            .get(Symbol::new(self.env(), "swap_minter"))
            .expect("swap_minter_config missing swap_minter field");
        let event_allow_asset = config_map
            .get(Symbol::new(self.env(), "allow_asset"))
            .expect("swap_minter_config missing allow_asset field");

        assert_eq!(
            &event_swap_minter, swap_minter,
            "swap_minter_config_set event has wrong swap_minter"
        );
        assert_eq!(
            &event_allow_asset, allow_asset,
            "swap_minter_config_set event has wrong allow_asset"
        );
    }

    fn assert_swap_minter_config_removed(
        &mut self,
        token: &Address,
        swap_minter: &Address,
        allow_asset: &Address,
    ) {
        let event = self.find_event_by_symbol("swap_minter_config_removed");

        assert!(
            event.is_some(),
            "swap_minter_config_removed event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "swap_minter_config_removed event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            2,
            "swap_minter_config_removed event should have 2 topics"
        );

        let event_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_token, token,
            "swap_minter_config_removed event has wrong token"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());
        let config_val = event_data
            .get(Symbol::new(self.env(), "swap_minter_config"))
            .expect("swap_minter_config_removed event missing swap_minter_config field");
        let config_map: soroban_sdk::Map<Symbol, Address> = config_val.into_val(self.env());

        let event_swap_minter = config_map
            .get(Symbol::new(self.env(), "swap_minter"))
            .expect("swap_minter_config missing swap_minter field");
        let event_allow_asset = config_map
            .get(Symbol::new(self.env(), "allow_asset"))
            .expect("swap_minter_config missing allow_asset field");

        assert_eq!(
            &event_swap_minter, swap_minter,
            "swap_minter_config_removed event has wrong swap_minter"
        );
        assert_eq!(
            &event_allow_asset, allow_asset,
            "swap_minter_config_removed event has wrong allow_asset"
        );
    }

    fn assert_max_message_body_size_updated(&mut self, expected_size: &u32) {
        let event = self.find_event_by_symbol("max_message_body_size_updated");

        assert!(
            event.is_some(),
            "max_message_body_size_updated event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "max_message_body_size_updated event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "max_message_body_size_updated event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "max_message_body_size_updated"),
            "Event topic should be 'max_message_body_size_updated'"
        );

        let event_data: soroban_sdk::Map<Symbol, u32> = data.into_val(self.env());
        let event_size = event_data
            .get(Symbol::new(self.env(), "new_max_message_body_size"))
            .expect("max_message_body_size_updated event missing new_max_message_body_size field");
        assert_eq!(
            &event_size, expected_size,
            "max_message_body_size_updated event has wrong size"
        );
    }

    fn assert_message_sent(&mut self) {
        let event = self.find_event_by_symbol("message_sent");

        assert!(event.is_some(), "message_sent event not found in event log");

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "message_sent event from wrong contract"
        );

        assert_eq!(topics.len(), 1, "message_sent event should have 1 topic");

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "message_sent"),
            "Event topic should be 'message_sent'"
        );

        let event_data: soroban_sdk::Map<Symbol, Bytes> = data.into_val(self.env());
        assert!(
            event_data.get(Symbol::new(self.env(), "message")).is_some(),
            "message_sent event missing message field"
        );
    }

    fn assert_message_received(&mut self) {
        let event = self.find_event_by_symbol("message_received");

        assert!(
            event.is_some(),
            "message_received event not found in event log"
        );

        let (contract, topics, _data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "message_received event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            4,
            "message_received event should have 4 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "message_received"),
            "First topic should be 'message_received'"
        );
    }

    fn extract_message_sent(&self) -> Option<Bytes> {
        let event = self.find_event_by_symbol("message_sent");
        event.map(|(_, _, data)| {
            let event_data: soroban_sdk::Map<Symbol, Bytes> = data.into_val(self.env());
            event_data
                .get(Symbol::new(self.env(), "message"))
                .expect("message_sent event missing message field")
        })
    }

    fn expect_message_sent(&self) -> Bytes {
        self.extract_message_sent()
            .expect("MessageSent event not found in event log")
    }

    fn assert_mint_and_forward(
        &mut self,
        forward_recipient: &MuxedAddress,
        token: &Address,
        amount: i128,
    ) {
        let event = self.find_event_by_symbol("mint_and_forward");

        assert!(
            event.is_some(),
            "mint_and_forward event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "mint_and_forward event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            1,
            "mint_and_forward event should have 1 topic"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "mint_and_forward"),
            "Event topic should be 'mint_and_forward'"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_forward_recipient: MuxedAddress = event_data
            .get(Symbol::new(self.env(), "forward_recipient"))
            .expect("mint_and_forward event missing forward_recipient field")
            .into_val(self.env());
        assert_eq!(
            event_forward_recipient, *forward_recipient,
            "mint_and_forward event has wrong forward_recipient"
        );

        let event_token: Address = event_data
            .get(Symbol::new(self.env(), "token"))
            .expect("mint_and_forward event missing token field")
            .into_val(self.env());
        assert_eq!(
            event_token, *token,
            "mint_and_forward event has wrong token"
        );

        let event_amount: i128 = event_data
            .get(Symbol::new(self.env(), "amount"))
            .expect("mint_and_forward event missing amount field")
            .into_val(self.env());
        assert_eq!(
            event_amount, amount,
            "mint_and_forward event has wrong amount"
        );
    }

    fn assert_deposit_for_burn(
        &mut self,
        burn_token: &Address,
        amount: i128,
        depositor: &Address,
        mint_recipient: &BytesN<32>,
        destination_domain: u32,
        destination_token_messenger: &BytesN<32>,
        destination_caller: &BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
        hook_data: &Bytes,
    ) {
        let event = self.find_event_by_symbol("deposit_for_burn");

        assert!(
            event.is_some(),
            "deposit_for_burn event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "deposit_for_burn event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            4,
            "deposit_for_burn event should have 4 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "deposit_for_burn"),
            "Event topic should be deposit_for_burn"
        );

        let event_burn_token: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_burn_token, burn_token,
            "deposit_for_burn event has wrong burn_token"
        );

        let event_depositor: Address = topics.get_unchecked(2).into_val(self.env());
        assert_eq!(
            &event_depositor, depositor,
            "deposit_for_burn event has wrong depositor"
        );

        let event_min_finality: u32 = topics.get_unchecked(3).into_val(self.env());
        assert_eq!(
            event_min_finality, min_finality_threshold,
            "deposit_for_burn event has wrong min_finality_threshold"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_amount: i128 = event_data
            .get(Symbol::new(self.env(), "amount"))
            .expect("deposit_for_burn event missing amount field")
            .into_val(self.env());
        assert_eq!(
            event_amount, amount,
            "deposit_for_burn event has wrong amount"
        );

        let event_mint_recipient: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "mint_recipient"))
            .expect("deposit_for_burn event missing mint_recipient field")
            .into_val(self.env());
        assert_eq!(
            &event_mint_recipient, mint_recipient,
            "deposit_for_burn event has wrong mint_recipient"
        );

        let event_destination_domain: u32 = event_data
            .get(Symbol::new(self.env(), "destination_domain"))
            .expect("deposit_for_burn event missing destination_domain field")
            .into_val(self.env());
        assert_eq!(
            event_destination_domain, destination_domain,
            "deposit_for_burn event has wrong destination_domain"
        );

        let event_destination_token_messenger: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "destination_token_messenger"))
            .expect("deposit_for_burn event missing destination_token_messenger field")
            .into_val(self.env());
        assert_eq!(
            &event_destination_token_messenger, destination_token_messenger,
            "deposit_for_burn event has wrong destination_token_messenger"
        );

        let event_destination_caller: BytesN<32> = event_data
            .get(Symbol::new(self.env(), "destination_caller"))
            .expect("deposit_for_burn event missing destination_caller field")
            .into_val(self.env());
        assert_eq!(
            &event_destination_caller, destination_caller,
            "deposit_for_burn event has wrong destination_caller"
        );

        let event_max_fee: i128 = event_data
            .get(Symbol::new(self.env(), "max_fee"))
            .expect("deposit_for_burn event missing max_fee field")
            .into_val(self.env());
        assert_eq!(
            event_max_fee, max_fee,
            "deposit_for_burn event has wrong max_fee"
        );

        let event_hook_data: Bytes = event_data
            .get(Symbol::new(self.env(), "hook_data"))
            .expect("deposit_for_burn event missing hook_data field")
            .into_val(self.env());
        assert_eq!(
            &event_hook_data, hook_data,
            "deposit_for_burn event has wrong hook_data"
        );
    }

    fn assert_mint_and_withdraw(
        &mut self,
        mint_recipient: &Address,
        amount: i128,
        mint_token: &Address,
        fee_collected: i128,
    ) {
        let event = self.find_event_by_symbol("mint_and_withdraw");

        assert!(
            event.is_some(),
            "mint_and_withdraw event not found in event log"
        );

        let (contract, topics, data) = event.unwrap();
        assert_eq!(
            contract,
            *self.contract(),
            "mint_and_withdraw event from wrong contract"
        );

        assert_eq!(
            topics.len(),
            3,
            "mint_and_withdraw event should have 3 topics"
        );

        let topic_symbol: Symbol = topics.get_unchecked(0).into_val(self.env());
        assert_eq!(
            topic_symbol,
            Symbol::new(self.env(), "mint_and_withdraw"),
            "Event topic should be mint_and_withdraw"
        );

        let event_mint_recipient: Address = topics.get_unchecked(1).into_val(self.env());
        assert_eq!(
            &event_mint_recipient, mint_recipient,
            "mint_and_withdraw event has wrong mint_recipient"
        );

        let event_mint_token: Address = topics.get_unchecked(2).into_val(self.env());
        assert_eq!(
            &event_mint_token, mint_token,
            "mint_and_withdraw event has wrong mint_token"
        );

        let event_data: soroban_sdk::Map<Symbol, Val> = data.into_val(self.env());

        let event_amount: i128 = event_data
            .get(Symbol::new(self.env(), "amount"))
            .expect("mint_and_withdraw event missing amount field")
            .into_val(self.env());
        assert_eq!(
            event_amount, amount,
            "mint_and_withdraw event has wrong amount"
        );

        let event_fee_collected: i128 = event_data
            .get(Symbol::new(self.env(), "fee_collected"))
            .expect("mint_and_withdraw event missing fee_collected field")
            .into_val(self.env());
        assert_eq!(
            event_fee_collected, fee_collected,
            "mint_and_withdraw event has wrong fee_collected"
        );
    }
}
