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

//! CCTP-specific roles for Stellar smart contracts.

#![no_std]

pub mod attestable;
pub mod denylistable;
pub mod fee_recipient;
pub mod min_fee_controller;
pub mod remote_token_messenger;
pub mod token_controller;

#[cfg(any(test, feature = "testutils"))]
pub mod test_utils;

pub use attestable::*;
pub use denylistable::*;
pub use fee_recipient::*;
pub use min_fee_controller::*;
pub use remote_token_messenger::*;
pub use token_controller::*;
