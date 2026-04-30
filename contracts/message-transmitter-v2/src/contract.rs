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
use cctp_interfaces::{MessageHandlerClient, Receiver, Relayer};
use cctp_roles::{attestable, Attestable};
use cctp_utils::MessageV2;
use common_roles::{
    manageable, ownable, pausable, rescuable, simple_role, Manageable, Ownable, Pausable, Rescuable,
};
use soroban_sdk::{
    address_payload::AddressPayload, contract, contractimpl, contracttype, panic_with_error,
    Address, Bytes, BytesN, Env, Vec,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{only_owner, when_not_paused, Upgradeable};
use stellar_utils::storage::ttl::{
    extend_instance_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};
use stellar_utils::{address_to_bytes32, is_zero_bytes};

use crate::storage::{self, validate_received_message, FINALITY_THRESHOLD_FINALIZED};
use crate::MessageTransmitterError;

#[allow(dead_code)]
#[derive(Upgradeable)]
#[contract]
pub struct MessageTransmitterV2Contract;

#[derive(Clone)]
#[contracttype]
pub struct MessageTransmitterV2ContractInitParams {
    pub owner: Address,
    pub pauser: Address,
    pub rescuer: Address,
    pub attester_manager: Address,
    pub attesters: Vec<BytesN<20>>,
    pub signature_threshold: u32,
    pub max_message_body_size: u32,
    pub admin: Address,
    pub local_domain: u32,
    pub version: u32,
}

#[contractimpl]
impl MessageTransmitterV2Contract {
    pub fn __constructor(e: Env, params: MessageTransmitterV2ContractInitParams) {
        ownable::set_owner_unchecked(&e, &params.owner);
        simple_role::set_role_and_emit_unchecked(
            &e,
            pausable::PAUSER,
            &params.pauser,
            pausable::emit_pauser_changed,
        );
        simple_role::set_role_and_emit_unchecked(
            &e,
            rescuable::RESCUER,
            &params.rescuer,
            rescuable::emit_rescuer_changed,
        );
        simple_role::set_role_and_emit_with_previous_unchecked(
            &e,
            attestable::ATTESTER_MANAGER,
            &params.attester_manager,
            attestable::emit_attester_manager_updated,
        );

        if params.attesters.is_empty() {
            panic_with_error!(e, MessageTransmitterError::NoAttesters);
        }

        // Enable all attesters
        for attester in params.attesters.iter() {
            attestable::enable_attester_unchecked(&e, &attester);
        }
        attestable::set_signature_threshold_unchecked(&e, params.signature_threshold);
        manageable::set_admin_unchecked(&e, &params.admin);

        e.storage().instance().set(
            &storage::MessageTransmitterStorageKey::LocalDomain,
            &params.local_domain,
        );
        e.storage().instance().set(
            &storage::MessageTransmitterStorageKey::Version,
            &params.version,
        );
        storage::set_max_message_body_size_unchecked(&e, params.max_message_body_size);

        // Claim the zero nonce to prevent it from being used
        let zero_nonce = BytesN::<32>::from_array(&e, &[0u8; 32]);
        storage::set_nonce_used(&e, &zero_nonce);

        extend_instance_ttl(&e, e.storage().max_ttl(), e.storage().max_ttl());
    }

    /// Returns the local domain identifier for this chain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The local domain identifier.
    pub fn get_local_domain(e: &Env) -> u32 {
        storage::get_local_domain(e)
    }

    /// Returns the message format version.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The message format version.
    pub fn get_version(e: &Env) -> u32 {
        storage::get_version(e)
    }

    /// Returns the maximum allowed message body size.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The maximum allowed message body size.
    pub fn get_max_message_body_size(e: &Env) -> u32 {
        storage::get_max_message_body_size(e)
    }

    /// Sets the maximum allowed message body size.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `max_message_body_size` - The maximum allowed message body size.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the
    ///   contract owner fails.
    ///
    /// # Events
    ///
    /// * topics - `["max_message_body_size_updated"]`
    /// * data - `[new_max_message_body_size: u32]`
    #[only_owner]
    pub fn set_max_message_body_size(e: &Env, max_message_body_size: u32) {
        storage::set_max_message_body_size_unchecked(e, max_message_body_size);
    }

    /// Checks if a nonce has been used.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `nonce` - The nonce to check.
    ///
    /// # Returns
    ///
    /// `true` if the nonce has been used, `false` otherwise.
    pub fn is_nonce_used(e: &Env, nonce: BytesN<32>) -> bool {
        storage::is_nonce_used(e, &nonce)
    }
}

#[contractimpl]
impl Relayer for MessageTransmitterV2Contract {
    /// Sends a message to the destination domain and recipient.
    ///
    /// Formats the message, and emits a `MessageSent` event with message information.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller (message sender).
    /// * `destination_domain` - The destination domain identifier.
    /// * `recipient` - The recipient address on destination chain as BytesN<32>.
    /// * `destination_caller` - Caller on the destination domain as BytesN<32>.
    /// * `min_finality_threshold` - The minimum finality threshold at which the message must be attested to.
    /// * `message_body` - Contents of the message (bytes).
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – `caller` authorization fails.
    /// * [`MessageTransmitterError::DestinationIsLocalDomain`] – Cannot send to local domain.
    /// * [`MessageTransmitterError::MessageBodyTooLarge`] – Message body exceeds max size.
    /// * [`MessageTransmitterError::RecipientIsZero`] – Recipient cannot be zero.
    /// * [`MessageTransmitterError::AddressTypeNotRecognized`] – Address type not recognized (unable to convert to bytes32 with Address::to_payload).
    ///
    /// # Events
    ///
    /// * topics - `["message_sent"]`
    /// * data - `[message: Bytes]`
    #[when_not_paused]
    fn send_message(
        e: &Env,
        caller: Address,
        destination_domain: u32,
        recipient: BytesN<32>,
        destination_caller: BytesN<32>,
        min_finality_threshold: u32,
        message_body: Bytes,
    ) {
        caller.require_auth();
        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        let local_domain = storage::get_local_domain(e);
        let version = storage::get_version(e);
        let max_message_body_size = storage::get_max_message_body_size(e);

        if destination_domain == local_domain {
            panic_with_error!(e, MessageTransmitterError::DestinationIsLocalDomain);
        }

        if message_body.len() > max_message_body_size {
            panic_with_error!(e, MessageTransmitterError::MessageBodyTooLarge);
        }

        if is_zero_bytes(&recipient) {
            panic_with_error!(e, MessageTransmitterError::RecipientIsZero);
        }

        let sender = address_to_bytes32(&caller).unwrap_or_else(|| {
            panic_with_error!(e, MessageTransmitterError::AddressTypeNotRecognized)
        });

        let message = MessageV2::format_for_relay(
            e,
            version,
            local_domain,
            destination_domain,
            sender,
            recipient,
            destination_caller,
            min_finality_threshold,
            message_body,
        );

        crate::emit_message_sent(e, &message);
    }
}

#[contractimpl]
impl Receiver for MessageTransmitterV2Contract {
    /// Receives a message. Messages can only be broadcast once for a given nonce.
    /// The message body of a valid message is passed to the specified recipient
    /// for further processing.
    ///
    /// # Attestation Format
    ///
    /// A valid attestation is the concatenated 65-byte signature(s) of exactly
    /// `thresholdSignature` signatures, in increasing order of attester address.
    /// ***If the attester addresses recovered from signatures are not in
    /// increasing order, signature verification will fail.***
    /// If incorrect number of signatures or duplicate signatures are supplied,
    /// signature verification will fail.
    ///
    /// # Message Format
    ///
    /// Field                        Bytes      Type       Index
    /// version                      4          uint32     0
    /// sourceDomain                 4          uint32     4
    /// destinationDomain            4          uint32     8
    /// nonce                        32         bytes32    12
    /// sender                       32         bytes32    44
    /// recipient                    32         bytes32    76
    /// destinationCaller            32         bytes32    108
    /// minFinalityThreshold         4          uint32     140
    /// finalityThresholdExecuted    4          uint32     144
    /// messageBody                  dynamic    bytes      148
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller.
    /// * `message` - The message raw bytes to receive.
    /// * `attestation` - Concatenated 65-byte signature(s) of the message, in increasing
    ///   order of the attester address recovered from signatures.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – `caller` authorization fails.
    /// * [`MessageTransmitterError::InvalidMessageFormat`] – Message is malformed or too short.
    /// * [`MessageTransmitterError::InvalidDestinationDomain`] – Destination domain does not match local domain.
    /// * [`MessageTransmitterError::InvalidDestinationCaller`] – Caller is not the authorized destination caller.
    /// * [`MessageTransmitterError::InvalidMessageVersion`] – Message version does not match.
    /// * [`MessageTransmitterError::NonceAlreadyUsed`] – Nonce has already been used.
    /// * [`MessageTransmitterError::HandleReceiveMessageFailed`] – Message handler returned false.
    ///
    /// # Events
    ///
    /// * topics - `["message_received", caller: Address, nonce: BytesN<32>, finality_threshold_executed: u32]`
    /// * data - `[source_domain: u32, sender: BytesN<32>, message_body: Bytes]`
    #[when_not_paused]
    fn receive_message(e: &Env, caller: Address, message: Bytes, attestation: Bytes) -> bool {
        caller.require_auth();
        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        let validated = validate_received_message(e, &caller, &message, &attestation);

        storage::set_nonce_used(e, &validated.nonce);

        // We're inferring a ContractIdHash address type because the recipient must be a contract that implements the MessageHandler interface.
        let recipient_payload = AddressPayload::ContractIdHash(validated.recipient.clone());
        let recipient_address = Address::from_payload(e, recipient_payload);
        let handler_client = MessageHandlerClient::new(e, &recipient_address);

        let success = if validated.finality_threshold_executed >= FINALITY_THRESHOLD_FINALIZED {
            handler_client.handle_recv_finalized_message(
                &validated.source_domain,
                &validated.sender,
                &validated.finality_threshold_executed,
                &validated.message_body,
            )
        } else {
            handler_client.handle_recv_unfinalized_message(
                &validated.source_domain,
                &validated.sender,
                &validated.finality_threshold_executed,
                &validated.message_body,
            )
        };

        if !success {
            panic_with_error!(e, MessageTransmitterError::HandleReceiveMessageFailed);
        }

        crate::emit_message_received(
            e,
            &caller,
            validated.source_domain,
            &validated.nonce,
            &validated.sender,
            validated.finality_threshold_executed,
            &validated.message_body,
        );

        true
    }
}

#[contractimpl]
impl Pausable for MessageTransmitterV2Contract {
    fn get_pauser(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, pausable::PAUSER)
    }
    fn update_pauser(e: &Env, new_pauser: Address) {
        simple_role::set_role_and_emit(
            e,
            pausable::PAUSER,
            &new_pauser,
            pausable::emit_pauser_changed,
        );
    }
    fn pause(e: &Env) {
        pausable::pause(e);
    }
    fn unpause(e: &Env) {
        pausable::unpause(e);
    }
    fn paused(e: &Env) -> bool {
        pausable::paused(e)
    }
}

#[contractimpl]
impl Ownable for MessageTransmitterV2Contract {
    fn get_owner(e: &Env) -> Option<Address> {
        ownable::get_owner(e)
    }
    fn get_pending_owner(e: &Env) -> Option<Address> {
        ownable::get_pending_owner(e)
    }
    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32) {
        ownable::transfer_ownership(e, &new_owner, expires_in_ledgers);
    }
    fn accept_ownership(e: &Env) {
        ownable::accept_ownership(e);
    }
}

#[contractimpl]
impl Rescuable for MessageTransmitterV2Contract {
    fn get_rescuer(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, rescuable::RESCUER)
    }
    fn update_rescuer(e: &Env, new_rescuer: Address) {
        simple_role::set_role_and_emit(
            e,
            rescuable::RESCUER,
            &new_rescuer,
            rescuable::emit_rescuer_changed,
        );
    }
    fn rescue_sep41(e: &Env, token_contract: Address, to: Address, amount: i128) {
        rescuable::rescue_sep41(e, &token_contract, &to, &amount)
    }
}

#[contractimpl]
impl Attestable for MessageTransmitterV2Contract {
    fn get_attester_manager(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, attestable::ATTESTER_MANAGER)
    }
    fn update_attester_manager(e: &Env, new_attester_manager: Address) {
        simple_role::set_role_and_emit_with_previous(
            e,
            attestable::ATTESTER_MANAGER,
            &new_attester_manager,
            attestable::emit_attester_manager_updated,
        );
    }
    fn enable_attester(e: &Env, attester: BytesN<20>) {
        attestable::enable_attester(e, &attester)
    }
    fn disable_attester(e: &Env, attester: BytesN<20>) {
        attestable::disable_attester(e, &attester)
    }
    fn get_enabled_attester(e: &Env, index: u32) -> BytesN<20> {
        attestable::get_enabled_attester(e, index)
    }
    fn get_num_enabled_attesters(e: &Env) -> u32 {
        attestable::get_num_enabled_attesters(e)
    }
    fn is_enabled_attester(e: &Env, attester: BytesN<20>) -> bool {
        attestable::is_enabled_attester(e, &attester)
    }
    fn get_signature_threshold(e: &Env) -> Option<u32> {
        attestable::get_signature_threshold(e)
    }
    fn set_signature_threshold(e: &Env, new_signature_threshold: u32) {
        attestable::set_signature_threshold(e, new_signature_threshold)
    }
}

#[contractimpl]
impl Manageable for MessageTransmitterV2Contract {
    fn get_admin(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, manageable::ADMIN)
    }

    fn get_pending_admin(e: &Env) -> Option<Address> {
        manageable::get_pending_admin(e)
    }

    fn transfer_admin(e: &Env, new_admin: Address, expires_in_ledgers: u32) {
        manageable::transfer_admin(e, &new_admin, expires_in_ledgers)
    }

    fn accept_admin(e: &Env) {
        manageable::accept_admin(e)
    }
}

impl UpgradeableInternal for MessageTransmitterV2Contract {
    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }
}
