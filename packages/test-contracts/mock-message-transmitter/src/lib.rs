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

//! Mock MessageTransmitter contract for testing CCTP contracts.
//!
//! This contract provides mock implementations of both `send_message` (for testing
//! deposit_for_burn in TokenMessengerMinter) and `receive_message` (for testing
//! mint_and_forward in CctpForwarder).
//!
//! Unlike a simple mock that just returns true, this mock parses the CCTP message
//! and calls the recipient's `handle_recv_finalized_message` via `MessageHandlerClient`,
//! enabling realistic integration testing with the real TokenMessengerMinter contract.

#![no_std]

use cctp_interfaces::{MessageHandlerClient, Receiver, Relayer};
use cctp_utils::MessageV2;
use soroban_sdk::{
    address_payload::AddressPayload, contract, contractevent, contractimpl, Address, Bytes, BytesN,
    Env,
};

/// Mock local domain constant for testing
pub const MOCK_LOCAL_DOMAIN: u32 = 0;
/// Mock message version for testing
pub const MOCK_MESSAGE_VERSION: u32 = 1;
/// Finality threshold for finalized messages
pub const FINALITY_THRESHOLD_FINALIZED: u32 = 1000;

/// Event emitted when a message is sent.
/// Matches the format of the real MessageTransmitter's MessageSent event.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSent {
    pub message: Bytes,
}

/// Mock MessageTransmitter contract for testing.
///
/// This mock implements both `send_message` (for outbound CCTP messages) and
/// `receive_message` (for inbound CCTP messages), allowing it to be used for
/// testing both TokenMessengerMinter and CctpForwarder contracts.
///
/// The `receive_message` implementation parses the CCTP message and calls the
/// recipient's `handle_recv_finalized_message`, enabling realistic integration
/// testing with the real TokenMessengerMinter contract.
#[contract]
pub struct MockMessageTransmitterContract;

#[contractimpl]
impl Relayer for MockMessageTransmitterContract {
    fn send_message(
        e: &Env,
        caller: Address,
        destination_domain: u32,
        recipient: BytesN<32>,
        destination_caller: BytesN<32>,
        min_finality_threshold: u32,
        message_body: Bytes,
    ) {
        // Create a mock CCTP message wrapping the burn message body
        // This matches what the real MessageTransmitter would emit
        let message = MessageV2::format_for_relay(
            e,
            MOCK_MESSAGE_VERSION,
            MOCK_LOCAL_DOMAIN,
            destination_domain,
            stellar_utils::address_to_bytes32(&caller).unwrap_or(BytesN::from_array(e, &[0; 32])),
            recipient,
            destination_caller,
            min_finality_threshold,
            message_body,
        );

        // Emit the message_sent event matching the real MessageTransmitter format
        MessageSent {
            message: message.clone(),
        }
        .publish(e);
    }
}

#[contractimpl]
impl Receiver for MockMessageTransmitterContract {
    fn receive_message(e: &Env, _caller: Address, message: Bytes, _attestation: Bytes) -> bool {
        MessageV2::validate_format(&message).unwrap();
        // Parse the CCTP message fields
        let source_domain = MessageV2::get_source_domain(&message).unwrap();
        let sender = MessageV2::get_sender(&message).unwrap();
        let recipient = MessageV2::get_recipient(&message).unwrap();
        let message_body = MessageV2::get_message_body(&message);

        // Get the recipient address from the message
        let recipient_payload = AddressPayload::ContractIdHash(recipient);
        let recipient_address = Address::from_payload(e, recipient_payload);

        // Call the recipient's handle_recv_finalized_message
        let handler_client = MessageHandlerClient::new(e, &recipient_address);
        handler_client.handle_recv_finalized_message(
            &source_domain,
            &sender,
            &FINALITY_THRESHOLD_FINALIZED,
            &message_body,
        )
    }
}
