/*
 * Copyright 2025 Circle Internet Group, Inc. All rights reserved.
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

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

/// Storage keys for MockSwapMinterContract.
#[derive(Clone)]
#[contracttype]
enum DataKey {
    MintAsset,
    AllowAsset,
}

/// MockSwapMinterContract implements the SwapMinter interface for testing.
/// This simulates the behavior of FiatTokenAdminContract's swap_mint functionality.
///
/// The contract burns allow_asset from the minter and mints mint_asset to the recipient.
#[contract]
pub struct MockSwapMinterContract;

#[contractimpl]
impl MockSwapMinterContract {
    /// Initialize the mock with the mint and allow asset addresses.
    pub fn __constructor(env: Env, mint_asset: Address, allow_asset: Address) {
        env.storage()
            .instance()
            .set(&DataKey::MintAsset, &mint_asset);
        env.storage()
            .instance()
            .set(&DataKey::AllowAsset, &allow_asset);
    }

    /// Mock swap_mint implementation that burns allow_asset and mints mint_asset.
    ///
    /// This simulates the real FiatTokenAdmin behavior:
    /// 1. Burn the allow_asset from the minter (requires prior approval and balance)
    /// 2. Mint the mint_asset to the recipient
    pub fn swap_mint(env: &Env, to: Address, amount: i128, minter: Address) {
        let mint_asset: Address = env
            .storage()
            .instance()
            .get(&DataKey::MintAsset)
            .expect("mint_asset not set");
        let allow_asset: Address = env
            .storage()
            .instance()
            .get(&DataKey::AllowAsset)
            .expect("allow_asset not set");

        // Burn allow_asset from the minter (requires prior approval and balance)
        TokenClient::new(env, &allow_asset).burn_from(
            &env.current_contract_address(),
            &minter,
            &amount,
        );

        // Mint tokens to the recipient
        StellarAssetClient::new(env, &mint_asset).mint(&to, &amount);
    }

    /// Returns the configured mint asset.
    pub fn get_mint_asset(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::MintAsset)
            .expect("mint_asset not set")
    }

    /// Returns the configured allow asset.
    pub fn get_allow_asset(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::AllowAsset)
            .expect("allow_asset not set")
    }
}
