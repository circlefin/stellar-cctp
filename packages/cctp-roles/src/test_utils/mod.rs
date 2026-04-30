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

//! Test utilities for CCTP roles.

pub mod attestable;
pub mod denylistable;
pub mod fee_recipient;
pub mod min_fee_controller;
pub mod remote_token_messenger;
pub mod token_controller;

#[cfg(any(test, feature = "testutils"))]
mod event_assertions;

#[cfg(any(test, feature = "testutils"))]
pub use event_assertions::CctpEventAssertions;
