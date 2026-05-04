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
//! # Attestable Contract Module
//!
//! This module provides functionality for managing attesters and verifying attestation signatures.
//!
//! ## Usage
//!
//! Contracts implementing the `Attestable` trait use `simple_role` directly:
//!
//! ```ignore
//! use cctp_roles::{attestable, simple_role};
//!
//! impl Attestable for MyContract {
//!     fn get_attester_manager(e: &Env) -> Option<Address> {
//!         simple_role::try_get_role(e, attestable::ATTESTER_MANAGER)
//!     }
//!
//!     fn update_attester_manager(e: &Env, new_attester_manager: Address) {
//!         simple_role::set_role_and_emit_with_previous(
//!             e,
//!             attestable::ATTESTER_MANAGER,
//!             &new_attester_manager,
//!             attestable::emit_attester_manager_updated,
//!         );
//!     }
//!
//!     // ... other trait methods
//! }
//! ```

mod storage;
#[cfg(test)]
mod test;

pub use storage::{
    disable_attester, enable_attester, enable_attester_unchecked, get_enabled_attester,
    get_num_enabled_attesters, get_signature_threshold, is_enabled_attester,
    recover_secp256k1_public_key, set_signature_threshold, set_signature_threshold_unchecked,
    verify_attestation_signatures,
};

use soroban_sdk::{contracterror, contractevent, Address, BytesN, Env};

/// Role identifier for the attester manager.
pub const ATTESTER_MANAGER: &str = "attester_manager";

/// Length of a secp256k1 signature in bytes (r: 32 bytes, s: 32 bytes, v: 1 byte)
pub const SIGNATURE_LENGTH: u32 = 65;

/// A trait for managing attestation in a contract.
///
/// Provides functions to manage attesters, signature thresholds, and the attester manager.
pub trait Attestable {
    /// Returns the current attester manager address, if set.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_attester_manager(e: &Env) -> Option<Address>;

    /// Updates the attester manager address.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_attester_manager` - The new attester manager address.
    ///
    /// # Errors
    ///
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the contract owner fails.
    fn update_attester_manager(e: &Env, new_attester_manager: Address);

    /// Enables an attester.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `attester` - The Ethereum address of the attester to enable.
    ///
    /// # Errors
    ///
    /// * [`RoleError::RoleNotSet`] - If the attester manager is not set.
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester manager fails.
    /// * [`AttestationError::InvalidAttesterPublicKey`] - If the attester address is all zeros.
    /// * [`AttestationError::AttesterAlreadyEnabled`] - If the attester is already enabled.
    fn enable_attester(e: &Env, attester: BytesN<20>);

    /// Disables an attester.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `attester` - The Ethereum address of the attester to disable.
    ///
    /// # Errors
    ///
    /// * [`RoleError::RoleNotSet`] - If the attester manager is not set.
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester manager fails.
    /// * [`AttestationError::AttesterAlreadyDisabled`] - If the attester is already disabled.
    /// * [`AttestationError::TooFewEnabledAttesters`] - If disabling would leave too few attesters.
    fn disable_attester(e: &Env, attester: BytesN<20>);

    /// Returns an enabled attester at the given index.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `index` - The index of the attester to retrieve.
    ///
    /// # Errors
    ///
    /// * [`AttestationError::AttesterIndexOutOfBounds`] - If the index is out of bounds.
    fn get_enabled_attester(e: &Env, index: u32) -> BytesN<20>;

    /// Returns the number of enabled attesters.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_num_enabled_attesters(e: &Env) -> u32;

    /// Returns whether an attester is enabled.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `attester` - The Ethereum address of the attester to check.
    fn is_enabled_attester(e: &Env, attester: BytesN<20>) -> bool;

    /// Returns the current signature threshold.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    fn get_signature_threshold(e: &Env) -> Option<u32>;

    /// Sets the signature threshold.
    ///
    /// # Arguments
    ///
    /// * `e` - Access to the Soroban environment.
    /// * `new_signature_threshold` - The new signature threshold.
    ///
    /// # Errors
    ///
    /// * [`RoleError::RoleNotSet`] - If the attester manager is not set.
    /// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester manager fails.
    /// * [`AttestationError::InvalidSignatureThreshold`] - If the threshold is zero.
    /// * [`AttestationError::SignatureThresholdTooHigh`] - If the threshold exceeds enabled attesters.
    /// * [`AttestationError::SignatureThresholdAlreadySet`] - If the threshold is already set to this value.
    fn set_signature_threshold(e: &Env, new_signature_threshold: u32);
}

/// Error codes for attestation verification
#[contracterror]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationError {
    /// The attestation length is invalid (must be SIGNATURE_LENGTH * threshold)
    InvalidAttestationLength = 6000,
    /// Signatures are not in increasing order or a duplicate signature was found
    InvalidSignatureOrder = 6001,
    /// The recovered signer is not an enabled attester
    SignerNotAttester = 6002,
    /// Failed to recover public key from signature
    SignatureRecoveryFailed = 6003,
    /// The signature threshold is invalid
    InvalidSignatureThreshold = 6004,
    /// Attempted to enable an attester that is already enabled
    AttesterAlreadyEnabled = 6005,
    /// Attempted to disable an attester that is already disabled
    AttesterAlreadyDisabled = 6006,
    /// Attempted to get an enabled attester at an index that is out of bounds
    AttesterIndexOutOfBounds = 6007,
    /// Public key is invalid (all zeros)
    InvalidAttesterAddress = 6008,
    /// Disabling would leave too few enabled attesters
    TooFewEnabledAttesters = 6009,
    /// The signature threshold exceeds the number of enabled attesters
    SignatureThresholdTooHigh = 6010,
    /// The signature threshold is already set
    SignatureThresholdAlreadySet = 6011,
    /// The signature threshold is not set
    SignatureThresholdNotSet = 6012,
    /// The signature recovery ID is invalid (must be 0 or 1, or 27/28 in Ethereum encoding)
    InvalidRecoveryId = 6013,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureThresholdUpdated {
    pub old_signature_threshold: u32,
    pub new_signature_threshold: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterEnabled {
    #[topic]
    pub attester: BytesN<20>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterDisabled {
    #[topic]
    pub attester: BytesN<20>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttesterManagerUpdated {
    #[topic]
    pub previous_attester_manager: Option<Address>,
    #[topic]
    pub new_attester_manager: Address,
}

/// Emits a SignatureThresholdUpdated event
///
/// # Arguments
/// * `e` - The contract environment
/// * `old_signature_threshold` - The old signature threshold
/// * `new_signature_threshold` - The new signature threshold
pub fn emit_signature_threshold_updated(
    e: &Env,
    old_signature_threshold: u32,
    new_signature_threshold: u32,
) {
    SignatureThresholdUpdated {
        old_signature_threshold,
        new_signature_threshold,
    }
    .publish(e);
}

/// Emits an AttesterEnabled event
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester` - The address of the attester that was enabled
pub fn emit_attester_enabled(e: &Env, attester: &BytesN<20>) {
    AttesterEnabled {
        attester: attester.clone(),
    }
    .publish(e);
}

/// Emits an AttesterDisabled event
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester` - The address of the attester that was disabled
pub fn emit_attester_disabled(e: &Env, attester: &BytesN<20>) {
    AttesterDisabled {
        attester: attester.clone(),
    }
    .publish(e);
}

/// Emits an AttesterManagerUpdated event.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment.
/// * `previous_attester_manager` - The previous attester manager address.
/// * `new_attester_manager` - The new attester manager address.
pub fn emit_attester_manager_updated(
    e: &Env,
    previous_attester_manager: Option<Address>,
    new_attester_manager: &Address,
) {
    AttesterManagerUpdated {
        previous_attester_manager,
        new_attester_manager: new_attester_manager.clone(),
    }
    .publish(e);
}
