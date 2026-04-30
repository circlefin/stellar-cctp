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

import type { Client as TokenMessengerMinterV2Client } from "../clients/token-messenger-minter-v2/src/index.js";

export interface LinkRemoteResourcesConfig {
  localToken: string;
  remoteDomains: number[];
  remoteTokenMessengers: Buffer[];
  remoteTokens: Buffer[];
  ownerClient: TokenMessengerMinterV2Client;
  tokenControllerClient: TokenMessengerMinterV2Client;
}

/**
 * Registers remote token messengers and links token pairs for each remote domain.
 * Idempotent: skips resources that are already configured.
 */
export const linkRemoteResources = async (config: LinkRemoteResourcesConfig): Promise<void> => {
  const { localToken, remoteDomains, remoteTokenMessengers, remoteTokens, ownerClient, tokenControllerClient } = config;

  if (remoteTokenMessengers.length > 0 && remoteTokenMessengers.length !== remoteDomains.length) {
    throw new Error(
      `Token messengers length mismatch: domains=${remoteDomains.length}, ` +
        `messengers=${remoteTokenMessengers.length}`,
    );
  }

  if (remoteTokens.length > 0 && remoteTokens.length !== remoteDomains.length) {
    throw new Error(`Token pairs length mismatch: domains=${remoteDomains.length}, ` + `tokens=${remoteTokens.length}`);
  }

  // Add remote token messengers (idempotent)
  for (let i = 0; i < remoteTokenMessengers.length; i++) {
    const domain = remoteDomains[i];
    const tokenMessenger = remoteTokenMessengers[i];

    const existingMessenger = (await ownerClient.get_remote_token_messenger({ domain })).result;

    if (existingMessenger) {
      console.log(`  Remote token messenger already set for domain=${domain}, skipping.`);
    } else {
      await (
        await ownerClient.add_remote_token_messenger({
          domain,
          token_messenger: tokenMessenger,
        })
      ).signAndSend();
      console.log(`  Remote token messenger added (domain=${domain})`);
    }
  }

  // Link token pairs (idempotent)
  for (let i = 0; i < remoteTokens.length; i++) {
    const domain = remoteDomains[i];
    const remoteToken = remoteTokens[i];

    const existingLocalToken = (
      await tokenControllerClient.get_local_token({
        remote_domain: domain,
        remote_token: remoteToken,
      })
    ).result;

    if (existingLocalToken) {
      console.log(`  Token pair already linked for domain=${domain}, skipping.`);
    } else {
      await (
        await tokenControllerClient.link_token_pair({
          local_token: localToken,
          remote_domain: domain,
          remote_token: remoteToken,
        })
      ).signAndSend();
      console.log(`  Token pair linked (domain=${domain}).`);
    }
  }
};
