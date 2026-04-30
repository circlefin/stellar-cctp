/**
 * Copyright 2026 Circle Internet Group, Inc. All rights reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { StrKey } from "@stellar/stellar-sdk";

/**
 * Validates that the input is a Stellar contract address (C…) and decodes it
 * to a 0x-prefixed bytes32 hex string suitable for EVM contract calls.
 *
 * @param strkey - Stellar contract address (C…)
 * @returns 0x-prefixed 64-character hex string
 * @throws If the input is not a valid contract address
 */
export function contractStrkeyToBytes32(strkey: string): `0x${string}` {
  if (!StrKey.isValidContract(strkey)) {
    throw new Error(`Invalid contract strkey: ${strkey}`);
  }
  return `0x${Buffer.from(StrKey.decodeContract(strkey)).toString("hex")}`;
}

/**
 * Builds the hookData buffer for a CCTP Forwarder burn message.
 *
 * Hook data layout:
 *   bytes  0–23: reserved (zeroed)
 *   bytes 24–27: hook data version (u32 BE, currently 0)
 *   bytes 28–31: forward_recipient byte length (u32 BE)
 *   bytes 32+  : forward_recipient (UTF-8 encoded Stellar strkey)
 *
 * @param forwardRecipientStrkey - Stellar strkey of the final token recipient (C…, G…, or M…)
 * @returns Hook data as a 0x-prefixed hex string
 */
export function buildCctpForwarderHookData(forwardRecipientStrkey: string): `0x${string}` {
  const isValid =
    StrKey.isValidEd25519PublicKey(forwardRecipientStrkey) ||
    StrKey.isValidContract(forwardRecipientStrkey) ||
    StrKey.isValidMed25519PublicKey(forwardRecipientStrkey);
  if (!isValid) {
    throw new Error(
      `Invalid forward recipient: ${forwardRecipientStrkey} (expected G..., C..., or M... address)`,
    );
  }

  const recipientBytes = Buffer.from(forwardRecipientStrkey, "utf8");
  const hookData = Buffer.alloc(32 + recipientBytes.length);
  hookData.writeUInt32BE(0, 24); // hook version = 0
  hookData.writeUInt32BE(recipientBytes.length, 28); // recipient byte length
  recipientBytes.copy(hookData, 32); // recipient strkey as UTF-8
  return `0x${hookData.toString("hex")}`;
}
