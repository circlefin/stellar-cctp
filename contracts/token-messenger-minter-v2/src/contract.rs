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

// Allowing more than 7 arguments so we can adhere to the CCTP deposit_for_burn and deposit_for_burn_with_hook specifications.
#![allow(clippy::too_many_arguments)]

use cctp_interfaces::{MessageHandler, TokenMessenger};
use cctp_roles::{
    denylistable, fee_recipient, min_fee_controller, remote_token_messenger, token_controller,
    Denylistable, FeeRecipient, MinFeeController, RemoteTokenMessenger, SwapMinterConfig,
    TokenController, TokenDecimalConfig,
};
use cctp_utils::to_local_amount;
use common_roles::{
    manageable, ownable, pausable, rescuable, simple_role, Manageable, Ownable, Pausable, Rescuable,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Bytes, BytesN, Env, Vec,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::{when_not_paused, Upgradeable};
use stellar_utils::storage::ttl::{
    extend_instance_ttl, DEFAULT_EXTEND_AMOUNT, DEFAULT_TTL_THRESHOLD,
};

use crate::deposit::deposit_for_burn_impl;
use crate::receive::handle_receive_message_impl;
use crate::storage::{self, TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD};
use crate::TokenMessengerMinterError;

#[allow(dead_code)]
#[derive(Upgradeable)]
#[contract]
pub struct TokenMessengerMinterV2Contract;

#[derive(Clone)]
#[contracttype]
pub struct TokenMessengerMinterV2ContractInitParams {
    pub owner: Address,
    pub pauser: Address,
    pub rescuer: Address,
    pub token_controller: Address,
    pub admin: Address,
    pub fee_recipient: Address,
    pub min_fee_controller: Address,
    pub denylister: Address,
    pub message_transmitter: Address,
    pub message_body_version: u32,
    pub remote_domains: Vec<u32>,
    pub remote_token_messengers: Vec<BytesN<32>>,
}

#[contractimpl]
impl TokenMessengerMinterV2Contract {
    /// Initializes the TokenMessengerMinterV2 contract.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `params` - Initialization parameters including:
    ///   - `owner`: The contract owner address.
    ///   - `pauser`: The address authorized to pause/unpause the contract.
    ///   - `rescuer`: The address authorized to rescue tokens.
    ///   - `token_controller`: The address authorized to manage token configurations.
    ///   - `admin`: The contract admin address.
    ///   - `fee_recipient`: The address that receives fees.
    ///   - `min_fee_controller`: The address that controls min fee configuration.
    ///   - `denylister`: The address authorized to manage the denylist.
    ///   - `message_transmitter`: The local MessageTransmitter contract address.
    ///   - `message_body_version`: The version for burn message bodies.
    ///   - `remote_domains`: List of remote domain identifiers.
    ///   - `remote_token_messengers`: Corresponding remote token messenger addresses.
    ///
    /// # Errors
    ///
    /// * Panics if `remote_domains` and `remote_token_messengers` have different lengths.
    /// * [`RemoteTokenMessengerError::ZeroAddress`] – If any token_messenger is zero.
    /// * [`RemoteTokenMessengerError::TokenMessengerAlreadySet`] – If a duplicate domain is provided.
    pub fn __constructor(e: Env, params: TokenMessengerMinterV2ContractInitParams) {
        // Initialize roles
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
        simple_role::set_role_and_emit_unchecked(
            &e,
            token_controller::TOKEN_CONTROLLER,
            &params.token_controller,
            token_controller::emit_set_token_controller,
        );
        manageable::set_admin_unchecked(&e, &params.admin);
        simple_role::set_role_and_emit_unchecked(
            &e,
            fee_recipient::FEE_RECIPIENT,
            &params.fee_recipient,
            fee_recipient::emit_fee_recipient_set,
        );
        simple_role::set_role_and_emit_unchecked(
            &e,
            min_fee_controller::MIN_FEE_CONTROLLER,
            &params.min_fee_controller,
            min_fee_controller::emit_min_fee_controller_set,
        );
        simple_role::set_role_and_emit_with_previous_unchecked(
            &e,
            denylistable::DENYLISTER,
            &params.denylister,
            denylistable::emit_denylister_changed,
        );

        // Initialize contract configuration
        e.storage().instance().set(
            &storage::TokenMessengerMinterStorageKey::LocalMessageTransmitter,
            &params.message_transmitter,
        );
        e.storage().instance().set(
            &storage::TokenMessengerMinterStorageKey::MessageBodyVersion,
            &params.message_body_version,
        );

        // Initialize remote token messengers
        assert_eq!(
            params.remote_domains.len(),
            params.remote_token_messengers.len(),
            "remote_domains and remote_token_messengers must have the same length"
        );

        for i in 0..params.remote_domains.len() {
            let domain = params.remote_domains.get(i).unwrap();
            let token_messenger = params.remote_token_messengers.get(i).unwrap();
            remote_token_messenger::add_remote_token_messenger_unchecked(
                &e,
                domain,
                &token_messenger,
            );
        }

        extend_instance_ttl(&e, e.storage().max_ttl(), e.storage().max_ttl());
    }

    /// Returns the address of the local MessageTransmitter contract.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The address of the local MessageTransmitter contract.
    ///
    /// # Errors
    ///
    /// * [`TokenMessengerMinterError::LocalMessageTransmitterNotSet`] – If not set.
    #[allow(dead_code)]
    pub fn get_local_message_transmitter(e: &Env) -> Address {
        storage::get_local_message_transmitter(e)
    }

    /// Returns the message body version used for burn messages.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    ///
    /// # Returns
    ///
    /// The message body version.
    ///
    /// # Errors
    ///
    /// * [`TokenMessengerMinterError::MessageBodyVersionNotSet`] – If not set.
    #[allow(dead_code)]
    pub fn get_message_body_version(e: &Env) -> u32 {
        storage::get_message_body_version(e)
    }
}

#[contractimpl]
impl TokenMessenger for TokenMessengerMinterV2Contract {
    /// Deposits and burns tokens from sender to be minted on destination domain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller depositing tokens.
    /// * `amount` - Amount of tokens to burn (must be non-zero).
    /// * `destination_domain` - Destination domain to receive message on.
    /// * `mint_recipient` - Address of mint recipient on destination domain (as bytes32).
    /// * `burn_token` - Token to burn `amount` of, on local domain.
    /// * `destination_caller` - Authorized caller on the destination domain (as bytes32).
    ///   If zero, any address can broadcast the message.
    /// * `max_fee` - Maximum fee to pay on the destination domain, in units of burn_token.
    /// * `min_finality_threshold` - The minimum finality at which the burn message will be attested.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – `caller` authorization fails.
    /// * [`DenylistError::AccountDenylisted`] – If caller is on the denylist.
    /// * [`TokenMessengerMinterError::AmountMustBeNonzero`] – If amount is zero.
    /// * [`TokenMessengerMinterError::MintRecipientMustBeNonzero`] – If mint_recipient is zero.
    /// * [`TokenMessengerMinterError::MaxFeeMustBeLessThanAmount`] – If max_fee >= amount.
    /// * [`TokenMessengerMinterError::InsufficientMaxFee`] – If max_fee < min fee for amount.
    /// * [`TokenMessengerMinterError::NoTokenMessengerForDomain`] – If no token messenger registered.
    /// * [`TokenMessengerMinterError::BurnTokenNotSupported`] – If burn limit is not set.
    /// * [`TokenMessengerMinterError::BurnAmountExceedsLimit`] – If amount exceeds burn limit.
    ///
    /// # Events
    ///
    /// * topics - `["deposit_for_burn", burn_token: Address, depositor: Address, min_finality_threshold: u32]`
    /// * data - `[amount, mint_recipient, destination_domain, destination_token_messenger, destination_caller, max_fee, hook_data]`
    #[when_not_paused]
    fn deposit_for_burn(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
    ) {
        caller.require_auth();
        denylistable::require_not_denylisted(e, &caller);

        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        let empty_hook_data = Bytes::new(e);
        deposit_for_burn_impl(
            e,
            &caller,
            amount,
            destination_domain,
            &mint_recipient,
            &burn_token,
            &destination_caller,
            max_fee,
            min_finality_threshold,
            &empty_hook_data,
        );
    }

    /// Deposits and burns tokens from sender to be minted on destination domain,
    /// with optional hook data for execution on the destination chain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `caller` - The address of the caller depositing tokens.
    /// * `amount` - Amount of tokens to burn (must be non-zero).
    /// * `destination_domain` - Destination domain to receive message on.
    /// * `mint_recipient` - Address of mint recipient on destination domain (as bytes32).
    /// * `burn_token` - Token to burn `amount` of, on local domain.
    /// * `destination_caller` - Authorized caller on the destination domain (as bytes32).
    ///   If zero, any address can broadcast the message.
    /// * `max_fee` - Maximum fee to pay on the destination domain, in units of burn_token.
    /// * `min_finality_threshold` - The minimum finality at which the burn message will be attested.
    /// * `hook_data` - Hook data to append to burn message for interpretation on destination domain.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – `caller` authorization fails.
    /// * [`DenylistError::AccountDenylisted`] – If caller is on the denylist.
    /// * [`TokenMessengerMinterError::HookDataEmpty`] – If hook_data is empty.
    /// * [`TokenMessengerMinterError::AmountMustBeNonzero`] – If amount is zero.
    /// * [`TokenMessengerMinterError::MintRecipientMustBeNonzero`] – If mint_recipient is zero.
    /// * [`TokenMessengerMinterError::MaxFeeMustBeLessThanAmount`] – If max_fee >= amount.
    /// * [`TokenMessengerMinterError::InsufficientMaxFee`] – If max_fee < min fee for amount.
    /// * [`TokenMessengerMinterError::NoTokenMessengerForDomain`] – If no token messenger registered.
    /// * [`TokenMessengerMinterError::BurnTokenNotSupported`] – If burn limit is not set.
    /// * [`TokenMessengerMinterError::BurnAmountExceedsLimit`] – If amount exceeds burn limit.
    ///
    /// # Events
    ///
    /// * topics - `["deposit_for_burn", burn_token: Address, depositor: Address, min_finality_threshold: u32]`
    /// * data - `[amount, mint_recipient, destination_domain, destination_token_messenger, destination_caller, max_fee, hook_data]`
    #[when_not_paused]
    fn deposit_for_burn_with_hook(
        e: &Env,
        caller: Address,
        amount: i128,
        destination_domain: u32,
        mint_recipient: BytesN<32>,
        burn_token: Address,
        destination_caller: BytesN<32>,
        max_fee: i128,
        min_finality_threshold: u32,
        hook_data: Bytes,
    ) {
        caller.require_auth();
        denylistable::require_not_denylisted(e, &caller);

        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        if hook_data.is_empty() {
            panic_with_error!(e, TokenMessengerMinterError::HookDataEmpty);
        }

        deposit_for_burn_impl(
            e,
            &caller,
            amount,
            destination_domain,
            &mint_recipient,
            &burn_token,
            &destination_caller,
            max_fee,
            min_finality_threshold,
            &hook_data,
        );
    }
}

#[contractimpl]
impl MessageHandler for TokenMessengerMinterV2Contract {
    /// Handles an incoming finalized message received by the local MessageTransmitter,
    /// and takes the appropriate action. For a burn message, mints the associated token
    /// to the requested recipient on the local domain.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `source_domain` - The domain where the message originated from.
    /// * `sender` - The sender of the message (remote TokenMessenger).
    /// * `finality_threshold_executed` - The level of finality at which the message was attested to.
    /// * `message_body` - The message body bytes (burn message).
    ///
    /// # Returns
    ///
    /// `true` if successful.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the local MessageTransmitter fails.
    /// * [`RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered`] – Sender is not a registered remote token messenger.
    /// * [`TokenMessengerMinterError::InvalidBurnMessageFormat`] – Burn message format is invalid.
    /// * [`TokenMessengerMinterError::InvalidBurnMessageVersion`] – Burn message version does not match.
    /// * [`TokenMessengerMinterError::MessageExpired`] – Message has expired and must be re-signed.
    /// * [`TokenMessengerMinterError::FeeEqualsOrExceedsAmount`] – Fee equals or exceeds the amount.
    /// * [`TokenMessengerMinterError::FeeExceedsMaxFee`] – Fee exceeds the max fee.
    /// * [`TokenMessengerMinterError::MintTokenNotSupported`] – No local token is linked for the remote token/domain.
    ///
    /// # Events
    ///
    /// * topics - `["mint_and_withdraw", mint_recipient: Address, mint_token: Address]`
    /// * data - `[amount: i128, fee_collected: i128]`
    #[when_not_paused]
    fn handle_recv_finalized_message(
        e: &Env,
        source_domain: u32,
        sender: BytesN<32>,
        _finality_threshold_executed: u32,
        message_body: Bytes,
    ) -> bool {
        // Verify the caller is the local MessageTransmitter
        let message_transmitter = storage::get_local_message_transmitter(e);
        message_transmitter.require_auth();
        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        // Validate sender is a registered remote token messenger for the domain
        remote_token_messenger::require_remote_token_messenger(e, source_domain, &sender);

        handle_receive_message_impl(e, source_domain, &message_body)
    }

    /// Handles an incoming unfinalized message received by the local MessageTransmitter,
    /// and takes the appropriate action. For a burn message, mints the associated token
    /// to the requested recipient on the local domain, less fees.
    /// Fees are separately minted to the currently set `fee_recipient` address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `source_domain` - The domain where the message originated from.
    /// * `sender` - The sender of the message (remote TokenMessenger).
    /// * `finality_threshold_executed` - The level of finality at which the message was attested to.
    /// * `message_body` - The message body bytes (burn message).
    ///
    /// # Returns
    ///
    /// `true` if successful.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Contract, #1000)` – Contract is paused (`EnforcedPaused`).
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the local MessageTransmitter fails.
    /// * [`RemoteTokenMessengerError::RemoteTokenMessengerNotRegistered`] – Sender is not a registered remote token messenger.
    /// * [`TokenMessengerMinterError::UnsupportedFinalityThreshold`] – Finality threshold is below minimum (500).
    /// * [`TokenMessengerMinterError::InvalidBurnMessageFormat`] – Burn message format is invalid.
    /// * [`TokenMessengerMinterError::InvalidBurnMessageVersion`] – Burn message version does not match.
    /// * [`TokenMessengerMinterError::MessageExpired`] – Message has expired and must be re-signed.
    /// * [`TokenMessengerMinterError::FeeEqualsOrExceedsAmount`] – Fee equals or exceeds the amount.
    /// * [`TokenMessengerMinterError::FeeExceedsMaxFee`] – Fee exceeds the max fee.
    /// * [`TokenMessengerMinterError::MintTokenNotSupported`] – No local token is linked for the remote token/domain.
    ///
    /// # Events
    ///
    /// * topics - `["mint_and_withdraw", mint_recipient: Address, mint_token: Address]`
    /// * data - `[amount: i128, fee_collected: i128]`
    #[when_not_paused]
    fn handle_recv_unfinalized_message(
        e: &Env,
        source_domain: u32,
        sender: BytesN<32>,
        finality_threshold_executed: u32,
        message_body: Bytes,
    ) -> bool {
        // Verify the caller is the local MessageTransmitter
        let message_transmitter = storage::get_local_message_transmitter(e);
        message_transmitter.require_auth();
        extend_instance_ttl(e, DEFAULT_TTL_THRESHOLD, DEFAULT_EXTEND_AMOUNT);

        // Validate sender is a registered remote token messenger for the domain
        remote_token_messenger::require_remote_token_messenger(e, source_domain, &sender);

        // Validate finality threshold meets minimum requirement
        if finality_threshold_executed < TOKEN_MESSENGER_MIN_FINALITY_THRESHOLD {
            panic_with_error!(e, TokenMessengerMinterError::UnsupportedFinalityThreshold);
        }

        // Handle the message
        handle_receive_message_impl(e, source_domain, &message_body)
    }
}

#[contractimpl]
impl Pausable for TokenMessengerMinterV2Contract {
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
impl Ownable for TokenMessengerMinterV2Contract {
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
impl Rescuable for TokenMessengerMinterV2Contract {
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
impl TokenController for TokenMessengerMinterV2Contract {
    fn get_token_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, token_controller::TOKEN_CONTROLLER)
    }
    fn set_token_controller(e: &Env, new_token_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            token_controller::TOKEN_CONTROLLER,
            &new_token_controller,
            token_controller::emit_set_token_controller,
        );
    }
    fn link_token_pair(
        e: &Env,
        local_token: Address,
        remote_domain: u32,
        remote_token: BytesN<32>,
    ) {
        token_controller::link_token_pair(e, &local_token, remote_domain, &remote_token);
    }
    fn unlink_token_pair(
        e: &Env,
        local_token: Address,
        remote_domain: u32,
        remote_token: BytesN<32>,
    ) {
        token_controller::unlink_token_pair(e, &local_token, remote_domain, &remote_token);
    }
    fn set_max_burn_amount_per_message(
        e: &Env,
        local_token: Address,
        burn_limit_per_message: i128,
    ) {
        token_controller::set_max_burn_amount_per_message(e, &local_token, burn_limit_per_message);
    }
    fn get_local_token(e: &Env, remote_domain: u32, remote_token: BytesN<32>) -> Option<Address> {
        token_controller::get_local_token(e, remote_domain, &remote_token)
    }
    fn get_token_decimal_config(e: &Env, local_token: Address) -> Option<TokenDecimalConfig> {
        token_controller::get_token_decimal_config(e, &local_token)
    }
    fn set_token_decimal_config(
        e: &Env,
        local_token: Address,
        local_decimals: u32,
        canonical_decimals: u32,
    ) {
        token_controller::set_token_decimal_config(
            e,
            &local_token,
            local_decimals,
            canonical_decimals,
        );
    }
    fn get_max_burn_amount_per_message(e: &Env, local_token: Address) -> Option<i128> {
        token_controller::get_max_burn_amount_per_message(e, &local_token)
    }
    fn get_swap_minter_config(e: &Env, local_token: Address) -> Option<SwapMinterConfig> {
        token_controller::get_swap_minter_config(e, &local_token)
    }
    fn set_swap_minter_config(
        e: &Env,
        local_token: Address,
        swap_minter: Address,
        allow_asset: Address,
    ) {
        token_controller::set_swap_minter_config(e, &local_token, &swap_minter, &allow_asset);
    }
    fn remove_swap_minter_config(e: &Env, local_token: Address) {
        token_controller::remove_swap_minter_config(e, &local_token);
    }
}

#[contractimpl]
impl Manageable for TokenMessengerMinterV2Contract {
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

#[contractimpl]
impl MinFeeController for TokenMessengerMinterV2Contract {
    fn get_min_fee_controller(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, min_fee_controller::MIN_FEE_CONTROLLER)
    }
    fn set_min_fee_controller(e: &Env, new_min_fee_controller: Address) {
        simple_role::set_role_and_emit(
            e,
            min_fee_controller::MIN_FEE_CONTROLLER,
            &new_min_fee_controller,
            min_fee_controller::emit_min_fee_controller_set,
        );
    }
    fn set_min_fee(e: &Env, burn_token: Address, min_fee: i128) {
        min_fee_controller::set_min_fee(e, &burn_token, min_fee);
    }
    fn get_min_fee(e: &Env, burn_token: Address) -> i128 {
        min_fee_controller::get_min_fee(e, &burn_token)
    }
    fn get_min_fee_amount(e: &Env, burn_token: Address, amount: i128) -> i128 {
        // Normalize the amount to canonical decimals before computing the fee,
        // matching the calculation performed during deposit_for_burn.
        let normalized = crate::deposit::to_canonical_amount_normalized(e, &burn_token, amount);
        let canonical_fee =
            min_fee_controller::get_min_fee_amount(e, &burn_token, normalized.canonical_amount);
        to_local_amount(canonical_fee, normalized.decimal_pair).unwrap_or_else(|_| {
            panic_with_error!(e, TokenMessengerMinterError::DecimalConversionFailed)
        })
    }
}

#[contractimpl]
impl RemoteTokenMessenger for TokenMessengerMinterV2Contract {
    fn add_remote_token_messenger(e: &Env, domain: u32, token_messenger: BytesN<32>) {
        remote_token_messenger::add_remote_token_messenger(e, domain, &token_messenger);
    }
    fn remove_remote_token_messenger(e: &Env, domain: u32) {
        remote_token_messenger::remove_remote_token_messenger(e, domain);
    }
    fn get_remote_token_messenger(e: &Env, domain: u32) -> Option<BytesN<32>> {
        remote_token_messenger::get_remote_token_messenger(e, domain)
    }
}

#[contractimpl]
impl FeeRecipient for TokenMessengerMinterV2Contract {
    fn get_fee_recipient(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, fee_recipient::FEE_RECIPIENT)
    }
    fn set_fee_recipient(e: &Env, new_fee_recipient: Address) {
        simple_role::set_role_and_emit(
            e,
            fee_recipient::FEE_RECIPIENT,
            &new_fee_recipient,
            fee_recipient::emit_fee_recipient_set,
        );
    }
}

#[contractimpl]
impl Denylistable for TokenMessengerMinterV2Contract {
    fn get_denylister(e: &Env) -> Option<Address> {
        simple_role::try_get_role(e, denylistable::DENYLISTER)
    }
    fn update_denylister(e: &Env, denylister: Address) {
        simple_role::set_role_and_emit_with_previous(
            e,
            denylistable::DENYLISTER,
            &denylister,
            denylistable::emit_denylister_changed,
        );
    }
    fn denylist(e: &Env, account: Address) {
        denylistable::denylist(e, &account);
    }
    fn un_denylist(e: &Env, account: Address) {
        denylistable::un_denylist(e, &account);
    }
    fn is_denylisted(e: &Env, account: Address) -> bool {
        denylistable::is_denylisted(e, &account)
    }
}

impl UpgradeableInternal for TokenMessengerMinterV2Contract {
    fn _require_auth(e: &Env, _operator: &Address) {
        manageable::enforce_admin_auth(e);
    }
}
