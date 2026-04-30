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

//! Test utilities for verifying Attestable contract implementations

use soroban_sdk::{
    bytes,
    testutils::{MockAuth, MockAuthInvoke},
    Address, Bytes, BytesN, Env, IntoVal,
};

/// A simple container for a message and its attestation bytes.
#[derive(Debug)]
pub struct AttestationFixture {
    pub message: Bytes,
    pub attestation: Bytes,
}

// ================================
// Byte manipulation helpers
// ================================

/// Returns a copy of `b` with one extra byte appended.
pub fn bytes_with_trailing_byte(env: &Env, b: &Bytes, byte: u8) -> Bytes {
    let mut out = Bytes::new(env);
    out.append(b);
    out.push_back(byte);
    out
}

/// Returns a copy of `b` with the byte at `idx` set to `new_byte`.
pub fn bytes_with_byte_set(env: &Env, b: &Bytes, idx: u32, new_byte: u8) -> Bytes {
    let mut out = Bytes::new(env);
    for i in 0..b.len() {
        let byte = if i == idx {
            new_byte
        } else {
            b.get(i)
                .unwrap_or_else(|| panic!("Bytes.get({}) unexpectedly returned None", i))
        };
        out.push_back(byte);
    }
    out
}

/// Returns a copy of `b` where bytes in `[start, end)` are filled with `value`.
pub fn bytes_with_range_filled(env: &Env, b: &Bytes, start: u32, end: u32, value: u8) -> Bytes {
    let mut out = Bytes::new(env);
    for i in 0..b.len() {
        let byte = if i >= start && i < end {
            value
        } else {
            b.get(i)
                .unwrap_or_else(|| panic!("Bytes.get({}) unexpectedly returned None", i))
        };
        out.push_back(byte);
    }
    out
}

// ================================
// Test fixtures
// ================================

/// Fixture where the attestation signatures are in the wrong order
pub fn fixture_invalid_order(env: &Env) -> AttestationFixture {
    // load destination message and attestation data from hash:
    // 0x5fb6eac86e4ea9cadf6f37a1469cf32d7b4b95666addcc507993a4ba8594e41b
    let message_u8: &[u8] = &[
        0, 0, 0, 1, 0, 0, 0, 13, 0, 0, 0, 1, 124, 164, 236, 114, 162, 50, 74, 41, 249, 153, 158,
        155, 158, 51, 145, 229, 102, 48, 9, 13, 25, 61, 111, 181, 216, 135, 198, 81, 68, 196, 239,
        204, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181, 160, 233, 198, 33, 165, 186, 218, 165,
        54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181,
        160, 233, 198, 33, 165, 186, 218, 165, 54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 3, 232, 0, 0, 7, 208, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 33, 157,
        212, 0, 242, 191, 96, 229, 162, 61, 19, 190, 114, 180, 134, 212, 3, 136, 148, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 38, 46, 16, 114, 121, 87, 22, 93, 186, 150, 183, 232, 130, 229, 78,
        16, 164, 63, 228, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 34, 39, 5, 176, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 35, 249, 67, 24, 20,
        8, 234, 196, 36, 17, 106, 247, 183, 121, 12, 148, 203, 151, 165, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let message = Bytes::from_slice(env, message_u8);

    let attestation = bytes!(
        env,
        0x71a2cb9a1a2ebb617146d2dc5b68f288c825ed2401a47c883a4537ac295184aa507000fd11054198e39d7df39280d365c2e0966d86a4f63280bf8d21b748cc111c2617e687d62726f9c994785be959b52fa01167ca6f8cb118972629847c27bb0a32726cf6df976ccb4f39546fa9379964aadab465a5c2be0c359ed940b80733761b,
    );

    AttestationFixture {
        message,
        attestation,
    }
}

/// Fixture with duplicated signature bytes
pub fn fixture_dupe_signatures(env: &Env) -> AttestationFixture {
    // load destination message and attestation data from hash:
    // 0x5fb6eac86e4ea9cadf6f37a1469cf32d7b4b95666addcc507993a4ba8594e41b
    let message_u8: &[u8] = &[
        0, 0, 0, 1, 0, 0, 0, 13, 0, 0, 0, 1, 124, 164, 236, 114, 162, 50, 74, 41, 249, 153, 158,
        155, 158, 51, 145, 229, 102, 48, 9, 13, 25, 61, 111, 181, 216, 135, 198, 81, 68, 196, 239,
        204, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181, 160, 233, 198, 33, 165, 186, 218, 165,
        54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181,
        160, 233, 198, 33, 165, 186, 218, 165, 54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 3, 232, 0, 0, 7, 208, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 33, 157,
        212, 0, 242, 191, 96, 229, 162, 61, 19, 190, 114, 180, 134, 212, 3, 136, 148, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 38, 46, 16, 114, 121, 87, 22, 93, 186, 150, 183, 232, 130, 229, 78,
        16, 164, 63, 228, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 34, 39, 5, 176, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 35, 249, 67, 24, 20,
        8, 234, 196, 36, 17, 106, 247, 183, 121, 12, 148, 203, 151, 165, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let message = Bytes::from_slice(env, message_u8);

    let attestation = bytes!(
        env,
        0x71a2cb9a1a2ebb617146d2dc5b68f288c825ed2401a47c883a4537ac295184aa507000fd11054198e39d7df39280d365c2e0966d86a4f63280bf8d21b748cc111c71a2cb9a1a2ebb617146d2dc5b68f288c825ed2401a47c883a4537ac295184aa507000fd11054198e39d7df39280d365c2e0966d86a4f63280bf8d21b748cc111c,
    );

    AttestationFixture {
        message,
        attestation,
    }
}

/// Valid fixture with multiple signatures.
pub fn fixture_valid(env: &Env) -> AttestationFixture {
    // load destination message and attestation data from hash:
    // 0xc05fbd5c4a86902c4ee8d28a09a745d8e57361df7271de2d88157e205ceb04e0
    let message_u8: &[u8] = &[
        0, 0, 0, 1, 0, 0, 0, 13, 0, 0, 0, 2, 187, 97, 196, 135, 192, 3, 72, 149, 190, 246, 120, 69,
        230, 242, 100, 247, 27, 44, 54, 208, 145, 63, 108, 165, 188, 122, 204, 31, 84, 157, 233,
        150, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181, 160, 233, 198, 33, 165, 186, 218, 165,
        54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 181,
        160, 233, 198, 33, 165, 186, 218, 165, 54, 33, 155, 58, 34, 140, 129, 104, 207, 93, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 176, 67, 132, 12, 210, 243, 38, 135, 236, 107, 99, 251, 4,
        18, 88, 93, 227, 152, 34, 0, 0, 3, 232, 0, 0, 7, 208, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 41, 33, 157, 212, 0, 242, 191, 96, 229, 162, 61, 19, 190, 114, 180, 134, 212,
        3, 136, 148, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 176, 67, 132, 12, 210, 243, 38, 135,
        236, 107, 99, 251, 4, 18, 88, 93, 227, 152, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 165, 55, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 9, 176, 67, 132, 12, 210, 243, 38, 135, 236, 107, 99, 251, 4, 18, 88, 93, 227, 152, 34,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 53, 165, 55, 32, 33, 117, 169, 251, 110, 36, 228, 30, 255, 204, 230, 30, 146,
        142, 92, 159, 229, 179, 219, 75, 3, 104, 167, 119, 150, 221, 89, 247, 22, 17, 203, 3, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 174, 104, 183, 17, 123, 224, 2, 108, 189, 67, 102, 48, 63,
        116, 238, 203, 177, 158, 64, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 164, 245, 87, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 112, 69, 109,
    ];

    let message = Bytes::from_slice(env, message_u8);

    let attestation = bytes!(
        env,
        0x111a2b4ce084ccc83c8ef18eabf50fc332f603c7cd2ef7e177ea45a1c332f1d97c0fc30429b2c6ce57f7cb6bbcbdabff5d94b487075dff3b09182c25d6e9d98a1cc901a511af9b85b6db2b6660d3aeb029129a26944b69a74dd7c26a64e154726f23bd811da1a90c6ce36a9170785966de2a9280c39042fedb95bbc29bbf301c0d1c,
    );

    AttestationFixture {
        message,
        attestation,
    }
}

// ================================
// Helpers for constructing malleated secp256k1 signatures in tests.
//
// In ECDSA, a valid signature `(r, s)` has an equally-valid twin `(r, n - s)` (where `n` is the
// secp256k1 curve order). Some systems reject the "high-s" form (`s > n/2`) to avoid signature
// malleability and these helpers let tests exercise that behavior.
// ================================

/// The secp256k1 curve order (n). https://en.bitcoin.it/wiki/Secp256k1
const SECP256K1_N: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
];

// Big-endian subtraction: out = a - b, assuming a >= b.
fn sub_be_32(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut borrow: i16 = 0;

    for i in (0..32).rev() {
        let ai = a[i] as i16;
        let bi = b[i] as i16;
        let mut v = ai - bi - borrow;
        if v < 0 {
            v += 256;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[i] = v as u8;
    }

    out
}

/// Takes a 65-byte (r[32]||s[32]||v[1]) signature and returns the malleated version:
/// same r, s' = n - s, and v flipped (27<->28).
pub fn malleate_secp256k1_sig(env: &Env, sig65: &Bytes) -> Bytes {
    assert_eq!(sig65.len(), 65);

    let r = sig65.slice(0..32);
    let s_bytes = sig65.slice(32..64);
    let v = sig65.get(64).unwrap();

    let mut s_arr = [0u8; 32];
    for (i, b) in s_arr.iter_mut().enumerate() {
        *b = s_bytes.get(i as u32).unwrap();
    }

    // s' = n - s
    let s_prime = sub_be_32(&SECP256K1_N, &s_arr);

    // Flip v
    let v_prime = match v {
        27 => 28,
        28 => 27,
        0 => 1,
        1 => 0,
        _ => v,
    };

    // Rebuild signature
    let mut out = Bytes::new(env);
    out.append(&r);

    let mut s_prime_bytes = Bytes::new(env);
    for b in s_prime {
        s_prime_bytes.push_back(b);
    }
    out.append(&s_prime_bytes);

    out.push_back(v_prime);
    out
}

// ================================
// Mock auth helpers for attestable functions
// ================================

/// Sets mock auth so that the owner can call `update_attester_manager` on `contract_id`.
pub fn mock_update_attester_manager_auth(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    new_attester_manager: &Address,
) {
    env.mock_auths(&[MockAuth {
        address: owner,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "update_attester_manager",
            args: (new_attester_manager,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the attester_manager can call `enable_attester` on `contract_id`.
pub fn mock_enable_attester_auth(
    env: &Env,
    contract_id: &Address,
    attester_manager: &Address,
    attester: &BytesN<20>,
) {
    env.mock_auths(&[MockAuth {
        address: attester_manager,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "enable_attester",
            args: (attester,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the attester_manager can call `disable_attester` on `contract_id`.
pub fn mock_disable_attester_auth(
    env: &Env,
    contract_id: &Address,
    attester_manager: &Address,
    attester: &BytesN<20>,
) {
    env.mock_auths(&[MockAuth {
        address: attester_manager,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "disable_attester",
            args: (attester,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Sets mock auth so that the attester_manager can call `set_signature_threshold` on `contract_id`.
pub fn mock_set_signature_threshold_auth(
    env: &Env,
    contract_id: &Address,
    attester_manager: &Address,
    threshold: u32,
) {
    env.mock_auths(&[MockAuth {
        address: attester_manager,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "set_signature_threshold",
            args: (threshold,).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
