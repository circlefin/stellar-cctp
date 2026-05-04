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
use simple_role_macros::enforce_role_auth;
use soroban_sdk::{contracttype, crypto::Hash, panic_with_error, Bytes, BytesN, Env, Vec};
use stellar_utils::is_zero_bytes;

use super::ATTESTER_MANAGER;
use crate::{
    emit_attester_disabled, emit_attester_enabled, emit_signature_threshold_updated,
    AttestationError, SIGNATURE_LENGTH,
};

#[contracttype]
pub enum AttestableStorageKey {
    SignatureThreshold,
    EnabledAttesters,
}

/// Sets the signature threshold
///
/// # Arguments
/// * `e` - The contract environment
/// * `threshold` - The new signature threshold
///
/// # Errors
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester
///   manager fails.
/// * [`AttestationError::InvalidSignatureThreshold`] - If the threshold is zero.
/// * [`AttestationError::SignatureThresholdTooHigh`] - If the threshold exceeds enabled attesters.
/// * [`AttestationError::SignatureThresholdAlreadySet`] - If the threshold is already set to this value.
#[enforce_role_auth(ATTESTER_MANAGER)]
pub fn set_signature_threshold(e: &Env, threshold: u32) {
    set_signature_threshold_unchecked(e, threshold);
}

/// Sets the signature threshold without authorization checks.
///
/// # Arguments
/// * `e` - The contract environment.
/// * `threshold` - The new signature threshold.
///
/// # Errors
/// * [`AttestationError::InvalidSignatureThreshold`] - If the threshold is zero.
/// * [`AttestationError::SignatureThresholdTooHigh`] - If the threshold exceeds enabled attesters.
/// * [`AttestationError::SignatureThresholdAlreadySet`] - If the threshold is already set to this value.
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call this function only in the constructor!
pub fn set_signature_threshold_unchecked(e: &Env, threshold: u32) {
    if threshold == 0 {
        panic_with_error!(e, AttestationError::InvalidSignatureThreshold);
    }
    if threshold > get_num_enabled_attesters(e) {
        panic_with_error!(e, AttestationError::SignatureThresholdTooHigh);
    }
    let signature_threshold = get_signature_threshold(e);
    if signature_threshold == Some(threshold) {
        panic_with_error!(e, AttestationError::SignatureThresholdAlreadySet);
    }
    e.storage()
        .instance()
        .set(&AttestableStorageKey::SignatureThreshold, &threshold);

    emit_signature_threshold_updated(e, signature_threshold.unwrap_or(0), threshold);
}

/// Retrieves the signature threshold from storage
///
/// # Arguments
/// * `e` - The contract environment
///
/// # Returns
/// The signature threshold
pub fn get_signature_threshold(e: &Env) -> Option<u32> {
    e.storage()
        .instance()
        .get::<_, u32>(&AttestableStorageKey::SignatureThreshold)
}

/// Enables an attester
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester_address` - The address of the attester to enable
///
/// # Errors
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester
///   manager fails.
/// * [`AttestationError::InvalidAttesterPublicKey`] - If the attester address is all zeros.
/// * [`AttestationError::AttesterAlreadyEnabled`] - If the attester is already enabled.
#[enforce_role_auth(ATTESTER_MANAGER)]
pub fn enable_attester(e: &Env, attester_address: &BytesN<20>) {
    enable_attester_unchecked(e, attester_address);
}

/// Enables an attester without checking authorization
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester_address` - The address of the attester to enable
///
/// # Notes
///
/// * IMPORTANT: This function lacks authorization checks. It is expected to call this function only in the constructor!
pub fn enable_attester_unchecked(e: &Env, attester_address: &BytesN<20>) {
    if is_zero_bytes(attester_address) {
        panic_with_error!(e, AttestationError::InvalidAttesterAddress);
    }
    if is_enabled_attester(e, attester_address) {
        panic_with_error!(e, AttestationError::AttesterAlreadyEnabled);
    }

    let mut attesters = get_enabled_attesters(e);
    attesters.push_back(attester_address.clone());
    e.storage()
        .instance()
        .set(&AttestableStorageKey::EnabledAttesters, &attesters);

    emit_attester_enabled(e, attester_address);
}

/// Disables an attester
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester_address` - The address of the attester to disable
///
/// # Errors
/// * `HostError: Error(Auth, InvalidAction)` – Authorization from the attester
///   manager fails.
/// * [`AttestationError::AttesterAlreadyDisabled`] - If the attester is already disabled.
/// * [`AttestationError::TooFewEnabledAttesters`] - If disabling would leave too few attesters.
#[enforce_role_auth(ATTESTER_MANAGER)]
pub fn disable_attester(e: &Env, attester_address: &BytesN<20>) {
    let num_enabled = get_num_enabled_attesters(e);
    if num_enabled <= 1 {
        panic_with_error!(e, AttestationError::TooFewEnabledAttesters);
    }

    // Disallow disabling an attester if it will go below the signature threshold
    let signature_threshold = get_signature_threshold(e)
        .unwrap_or_else(|| panic_with_error!(e, AttestationError::SignatureThresholdNotSet));
    if num_enabled <= signature_threshold {
        panic_with_error!(e, AttestationError::TooFewEnabledAttesters);
    }

    let mut attesters = get_enabled_attesters(e);
    let index = attesters
        .first_index_of(attester_address)
        .unwrap_or_else(|| panic_with_error!(e, AttestationError::AttesterAlreadyDisabled));

    attesters.remove(index);

    e.storage()
        .instance()
        .set(&AttestableStorageKey::EnabledAttesters, &attesters);

    emit_attester_disabled(e, attester_address);
}

/// Checks if an address corresponds to an enabled attester
///
/// # Arguments
/// * `e` - The contract environment
/// * `attester_address` - The address of the attester to check
///
/// # Returns
/// `true` if the address is an enabled attester, `false` otherwise
pub fn is_enabled_attester(e: &Env, attester_address: &BytesN<20>) -> bool {
    get_enabled_attesters(e).contains(attester_address)
}

/// Retrieves the number of enabled attesters
///
/// # Arguments
/// * `e` - The contract environment
///
/// # Returns
/// The number of enabled attesters
pub fn get_num_enabled_attesters(e: &Env) -> u32 {
    get_enabled_attesters(e).len()
}

/// Retrieves an enabled attester by index
///
/// # Arguments
/// * `e` - The contract environment
/// * `index` - The index of the attester to retrieve
///
/// # Returns
/// The public key of the enabled attester at the given index
pub fn get_enabled_attester(e: &Env, index: u32) -> BytesN<20> {
    let attesters = get_enabled_attesters(e);
    attesters
        .get(index)
        .unwrap_or_else(|| panic_with_error!(e, AttestationError::AttesterIndexOutOfBounds))
}

/// Retrieves the list of enabled attesters
fn get_enabled_attesters(e: &Env) -> Vec<BytesN<20>> {
    e.storage()
        .instance()
        .get::<_, Vec<BytesN<20>>>(&AttestableStorageKey::EnabledAttesters)
        .unwrap_or_else(|| Vec::new(e))
}

/// Verifies attestation signatures for a given message.
///
/// This function verifies that a message has been signed by the required threshold of attesters.
/// The attestation consists of concatenated 65-byte secp256k1 signatures.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment
/// * `message` - The message that was signed
/// * `attestation` - Concatenated signatures (65 bytes each)
///
/// # Errors
/// * [`AttestationError::InvalidAttestationLength`] - If the attestation length is invalid.
/// * [`AttestationError::InvalidSignatureOrder`] - If the signatures are not in strictly increasing order or a duplicate signature was found.
/// * [`AttestationError::SignerNotAttester`] - If the recovered signer is not an enabled attester.
pub fn verify_attestation_signatures(e: &Env, message: &Bytes, attestation: &Bytes) {
    let signature_threshold = get_signature_threshold(e)
        .unwrap_or_else(|| panic_with_error!(e, AttestationError::SignatureThresholdNotSet));

    let expected_length = SIGNATURE_LENGTH * signature_threshold;
    if attestation.len() != expected_length {
        panic_with_error!(e, AttestationError::InvalidAttestationLength);
    }

    let mut latest_attester_address: BytesN<20> = BytesN::from_array(e, &[0u8; 20]);

    let digest = e.crypto().keccak256(message);

    for i in 0..signature_threshold {
        let offset = i * SIGNATURE_LENGTH;

        let signature = attestation.slice(offset..(offset + SIGNATURE_LENGTH));

        let recovered_public_key = recover_secp256k1_public_key(e, &digest, &signature);
        let recovered_attester = eth_address_from_public_key(e, &recovered_public_key);

        // Verify signatures are in strictly increasing order of eth_address and prevent duplicates.
        if recovered_attester.to_array() <= latest_attester_address.to_array() {
            panic_with_error!(e, AttestationError::InvalidSignatureOrder);
        }
        if !is_enabled_attester(e, &recovered_attester) {
            panic_with_error!(e, AttestationError::SignerNotAttester);
        }

        latest_attester_address = recovered_attester;
    }
}

/// Recovers a secp256k1 public key from a message digest and signature.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment
/// * `digest` - The Keccak256 hash of the message
/// * `signature` - The 65-byte signature (r: 32 bytes, s: 32 bytes, v: 1 byte)
///
/// # Returns
///
/// The recovered 64-byte uncompressed public key (without the 0x04 prefix)
///
/// # Errors
///
/// Panics with [`AttestationError::InvalidRecoveryId`] if the recovery ID is not 0 or 1 (or 27/28).
/// Panics with `AttestationError::SignatureRecoveryFailed` if recovery fails.
pub fn recover_secp256k1_public_key(e: &Env, digest: &Hash<32>, signature: &Bytes) -> BytesN<64> {
    let r: BytesN<32> = signature
        .slice(0..32)
        .try_into()
        // Unreachable: slice() throws host IndexBounds before try_into() can fail
        .unwrap_or_else(|_| panic_with_error!(e, AttestationError::SignatureRecoveryFailed));
    let s: BytesN<32> = signature
        .slice(32..64)
        .try_into()
        // Unreachable: slice() throws host IndexBounds before try_into() can fail
        .unwrap_or_else(|_| panic_with_error!(e, AttestationError::SignatureRecoveryFailed));

    // In Ethereum signatures, v is typically 27 or 28
    // For secp256k1 recovery, we need recovery_id which is 0 or 1
    let v = match signature.get(64) {
        Some(v) => v as u32,
        None => panic_with_error!(e, AttestationError::SignatureRecoveryFailed),
    };
    let recovery_id = if v >= 27 { v - 27 } else { v };

    if recovery_id > 1 {
        panic_with_error!(e, AttestationError::InvalidRecoveryId);
    }

    match try_recover_secp256k1(e, digest, &r, &s, recovery_id) {
        Some(public_key) => public_key,
        None => panic_with_error!(e, AttestationError::SignatureRecoveryFailed),
    }
}

/// Attempts to recover a secp256k1 public key.
///
/// This is a wrapper around the Soroban SDK's secp256k1 recovery functionality.
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment
/// * `digest` - The message hash
/// * `r` - The r component of the signature
/// * `s` - The s component of the signature
/// * `recovery_id` - The recovery ID (0 or 1)
///
/// # Returns
///
/// `Some(public_key)` if recovery succeeds, `None` otherwise.
fn try_recover_secp256k1(
    e: &Env,
    digest: &Hash<32>,
    r: &BytesN<32>,
    s: &BytesN<32>,
    recovery_id: u32,
) -> Option<BytesN<64>> {
    let mut sig_bytes = Bytes::new(e);
    sig_bytes.append(&r.to_bytes());
    sig_bytes.append(&s.to_bytes());

    let signature: BytesN<64> = match sig_bytes.try_into() {
        Ok(sig) => sig,
        // Note: Unreachable because r (32 bytes) + s (32 bytes) = 64 bytes exactly.
        Err(_) => return None,
    };

    let recovered_65 = e
        .crypto()
        // Soroban SDK handles rejecting malleable signatures https://github.com/stellar/rs-soroban-env/blob/688bc34e6cd15c71742139e625268c7f30f55a92/soroban-env-host/src/crypto/mod.rs#L175
        .secp256k1_recover(digest, &signature, recovery_id);

    let key_array: [u8; 64] = recovered_65.to_array()[1..65].try_into().ok()?;
    Some(BytesN::from_array(e, &key_array))
}

/// Converts a secp256k1 public key to an Ethereum address
///
/// # Arguments
///
/// * `e` - Access to the Soroban environment
/// * `public_key` - The public key to convert
///
/// # Returns
///
/// The Ethereum address
fn eth_address_from_public_key(e: &Env, public_key: &BytesN<64>) -> BytesN<20> {
    let hash = e.crypto().keccak256(&public_key.to_bytes());
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash.to_array()[12..32]);
    BytesN::from_array(e, &address)
}
