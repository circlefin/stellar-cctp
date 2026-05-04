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

use crate::message::{validate_cctp_message, ValidatedMessageData};
use crate::storage;
use crate::CctpForwarderError;
use common_roles::{
    manageable, ownable, pausable, rescuable, simple_role, Manageable, Ownable, Pausable,
    Rescuable, ADMIN,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token, Address, Bytes, Env,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{when_not_paused, Upgradeable};
use stellar_utils::storage::ttl::{
    extend_instance_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

#[allow(dead_code)]
#[derive(Upgradeable)]
#[contract]
pub struct CctpForwarderContract;

/// Initialization parameters for the CctpForwarder contract.
#[derive(Clone)]
#[contracttype]
pub struct CctpForwarderContractInitParams {
    /// The contract owner address
    pub owner: Address,
    /// The pauser address
    pub pauser: Address,
    /// The rescuer address
    pub rescuer: Address,
    /// The admin address (handles upgrades)
    pub admin: Address,
    /// The MessageTransmitter contract address
    pub message_transmitter: Address,
    /// The TokenMessengerMinter contract address
    pub token_messenger_minter: Address,
    /// The expected message version
    pub expected_message_version: u32,
    /// The expected burn message version
    pub expected_burn_message_version: u32,
}

#[contractimpl]
impl CctpForwarderContract {
    /// Initializes the CctpForwarder contract.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `params` - Initialization parameters including roles and contract addresses.
    ///
    /// # Events
    ///
    /// Emits role configuration events for all roles.
    pub fn __constructor(e: Env, params: CctpForwarderContractInitParams) {
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
        manageable::set_admin_unchecked(&e, &params.admin);

        e.storage().instance().set(
            &storage::CctpForwarderStorageKey::MessageTransmitter,
            &params.message_transmitter,
        );

        e.storage().instance().set(
            &storage::CctpForwarderStorageKey::TokenMessengerMinter,
            &params.token_messenger_minter,
        );

        e.storage().instance().set(
            &storage::CctpForwarderStorageKey::ExpectedMessageVersion,
            &params.expected_message_version,
        );
        e.storage().instance().set(
            &storage::CctpForwarderStorageKey::ExpectedBurnMessageVersion,
            &params.expected_burn_message_version,
        );

        extend_instance_ttl(&e, e.storage().max_ttl(), e.storage().max_ttl());
    }

    /// Mints tokens via CCTP and forwards them to the forward recipient.
    ///
    /// This function calls `receive_message` on the MessageTransmitter to mint tokens
    /// to this contract, then forwards them to the recipient specified in the hook data.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `message` - The CCTP message bytes.
    /// * `attestation` - The attestation bytes for the message.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * [`CctpForwarderError::InvalidMessageFormat`] – The message format is invalid.
    /// * [`CctpForwarderError::UnsupportedMessageVersion`] – The message version is not supported.
    /// * [`CctpForwarderError::InvalidBurnMessageFormat`] – The burn message format is invalid.
    /// * [`CctpForwarderError::UnsupportedBurnMessageVersion`] – The burn message version is not supported.
    /// * [`CctpForwarderError::InvalidMintRecipient`] – The mintRecipient is not this contract.
    /// * [`CctpForwarderError::InvalidRecipient`] – The recipient is not the TokenMessengerMinter.
    /// * [`CctpForwarderError::HookDataTooShort`] – The hook data is too short.
    /// * [`CctpForwarderError::InvalidForwardRecipient`] – The forward recipient strkey is invalid.
    ///
    /// # Events
    ///
    /// * topics - `["mint_and_forward"]`
    /// * data - `[forward_recipient: MuxedAddress, token: Address, amount: i128]`
    #[when_not_paused]
    pub fn mint_and_forward(e: &Env, message: Bytes, attestation: Bytes) {
        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        let ValidatedMessageData {
            source_domain,
            burn_token,
            forward_recipient,
        } = validate_cctp_message(e, &message);

        let local_token = storage::get_local_token(e, source_domain, &burn_token);
        let contract_address = e.current_contract_address();

        let recipient_address = forward_recipient.address();
        if recipient_address == local_token || recipient_address == contract_address {
            panic_with_error!(e, CctpForwarderError::InvalidForwardRecipient);
        }
        let amount_minted =
            storage::mint_through_cctp(e, &contract_address, &local_token, &message, &attestation);

        let token_client = token::TokenClient::new(e, &local_token);
        token_client.transfer(&contract_address, &forward_recipient, &amount_minted);

        crate::emit_mint_and_forward(e, &forward_recipient, &local_token, amount_minted);
    }

    /// Returns the configured MessageTransmitter address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The MessageTransmitter contract address.
    pub fn get_message_transmitter(e: &Env) -> Address {
        storage::get_message_transmitter(e)
    }

    /// Returns the configured TokenMessengerMinter address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The TokenMessengerMinter contract address.
    pub fn get_token_messenger_minter(e: &Env) -> Address {
        storage::get_token_messenger_minter(e)
    }

    /// Returns the expected message version.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The expected message version.
    pub fn get_expected_message_version(e: &Env) -> u32 {
        storage::get_expected_msg_version(e)
    }

    /// Returns the expected burn message version.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The expected burn message version.
    pub fn get_expected_burn_msg_version(e: &Env) -> u32 {
        storage::get_expected_burn_msg_version(e)
    }
}

#[contractimpl]
impl Pausable for CctpForwarderContract {
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
impl Ownable for CctpForwarderContract {
    fn get_owner(e: &Env) -> Option<Address> {
        ownable::get_owner(e)
    }
    fn transfer_ownership(e: &Env, new_owner: Address, expires_in_ledgers: u32) {
        ownable::transfer_ownership(e, &new_owner, expires_in_ledgers);
    }
    fn accept_ownership(e: &Env) {
        ownable::accept_ownership(e);
    }
    fn get_pending_owner(e: &Env) -> Option<Address> {
        ownable::get_pending_owner(e)
    }
}

#[contractimpl]
impl Rescuable for CctpForwarderContract {
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
impl Manageable for CctpForwarderContract {
    fn get_admin(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, ADMIN)
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

impl UpgradeableInternal for CctpForwarderContract {
    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }
}
