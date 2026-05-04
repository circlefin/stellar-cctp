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

import { createHash, randomBytes } from "node:crypto";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { Client as FiatTokenAdminClient } from "@circlefin/stellar-stablecoin-scripts/clients/fiat-token-admin/src/index.js";
import type { FundingConfig } from "@circlefin/stellar-stablecoin-scripts/common/accounts.js";
import {
  loadEnvFile,
  resolveOrCreateEoa,
  upsertEnvFile,
} from "@circlefin/stellar-stablecoin-scripts/common/cli-utils.js";
import {
  createClientFactory,
  createSigner,
  createTrustline,
  fundAccount,
  fundAccounts,
  startQuickstartContainer,
} from "@circlefin/stellar-stablecoin-scripts/common/index.js";
import { deployStablecoinBase } from "@circlefin/stellar-stablecoin-scripts/deploy/stablecoin-base.js";
import { Address, Contract, Keypair, Networks, rpc, xdr } from "@stellar/stellar-sdk";
import { Wallet } from "ethers";
import { Client as TokenMessengerMinterV2Client } from "../../clients/token-messenger-minter-v2/src/index.js";
import { deployCctpForwarder } from "../../deploy/deploy-cctp-forwarder.js";
import { deployMessageTransmitter } from "../../deploy/deploy-message-transmitter-v2.js";
import { deployTokenMessengerMinter } from "../../deploy/deploy-token-messenger-minter-v2.js";
import { invokeContract, SacClient } from "../../sac/contract.js";
import { linkRemoteResources } from "../../setup/link-remote-resources.js";
import { hexToBuffer } from "../utils.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const { values } = parseArgs({
  options: {
    "mint-asset-code": { type: "string", default: "USDC" },
    "allow-asset-code": { type: "string", default: "USDCAllCCTP" },
    "from-env": { type: "string" },
    debug: { type: "boolean", default: false },
  },
  strict: true,
});

const MINT_ASSET_CODE = values["mint-asset-code"] ?? "USDC";
const ALLOW_ASSET_CODE = values["allow-asset-code"] ?? "USDCAllCCTP";
const ENV_FILE = values["from-env"] ? path.resolve(values["from-env"]) : path.resolve(__dirname, "../../.env.local");

// Default CCTP configuration for local testing
const DEFAULT_LOCAL_DOMAIN = 27; // Stellar domain
const DEFAULT_REMOTE_DOMAIN = 0;
const DEFAULT_MESSAGE_VERSION = 1;
const DEFAULT_MESSAGE_BODY_VERSION = 1;
const DEFAULT_MAX_MESSAGE_BODY_SIZE = 8192;
const DEFAULT_SIGNATURE_THRESHOLD = 2;
const DEFAULT_LOCAL_DECIMALS = 7;
const DEFAULT_CANONICAL_DECIMALS = 6;
const DEFAULT_MAX_BURN_AMOUNT = 1_000_000_000n; // 100 tokens (7 decimals)

const EOA_ALIASES = [
  // CCTP Deployer
  "cctp_deployer",
  // MessageTransmitter roles
  "message_transmitter_owner",
  "message_transmitter_pauser",
  "message_transmitter_admin",
  "message_transmitter_rescuer",
  "message_transmitter_attester_manager",
  // TokenMessengerMinter roles
  "token_messenger_minter_owner",
  "token_messenger_minter_pauser",
  "token_messenger_minter_admin",
  "token_messenger_minter_rescuer",
  "token_messenger_minter_token_controller",
  "token_messenger_minter_fee_recipient",
  "token_messenger_minter_min_fee_controller",
  "token_messenger_minter_denylister",
  // CctpForwarder roles
  "cctp_forwarder_owner",
  "cctp_forwarder_pauser",
  "cctp_forwarder_admin",
  "cctp_forwarder_rescuer",
  // Recipient (for verify scripts)
  "recipient",
  // FiatTokenAdmin roles
  "fiat_token_admin_deployer",
  "fiat_token_admin_owner",
  "fiat_token_admin_pauser",
  "fiat_token_admin_admin",
  "fiat_token_admin_minter_asset_controller",
  "fiat_token_admin_blocklister",
] as const;

async function main() {
  // 1. Start local Stellar node
  console.log("Starting local Stellar node...");
  const containerResult = await startQuickstartContainer({
    debug: values.debug ?? false,
  });
  if (containerResult.isErr()) {
    console.error("Failed to start:", containerResult.error.message);
    process.exit(1);
  }

  const { urls, stop } = containerResult.value;
  const rpcUrl = urls.rpcUrl.href;
  const friendbotUrl = urls.friendbotUrl.href;
  const networkPassphrase = Networks.STANDALONE;

  const fail = async (msg: string, err: Error): Promise<never> => {
    console.error(`${msg}:`, err.message);
    await stop();
    process.exit(1);
  };

  // Write network config to env file
  upsertEnvFile(ENV_FILE, "STELLAR_RPC_URL", rpcUrl);
  upsertEnvFile(ENV_FILE, "NETWORK_PASSPHRASE", networkPassphrase);
  upsertEnvFile(ENV_FILE, "FRIENDBOT_URL", friendbotUrl);
  upsertEnvFile(ENV_FILE, "HORIZON_URL", urls.horizonUrl.href);
  upsertEnvFile(ENV_FILE, "STELLAR_PORT", String(urls.port));
  upsertEnvFile(ENV_FILE, "ALLOW_HTTP", "true");
  upsertEnvFile(ENV_FILE, "FIAT_TOKEN_ADMIN_MINT_ASSET_CODE", MINT_ASSET_CODE);
  upsertEnvFile(ENV_FILE, "FIAT_TOKEN_ADMIN_ALLOW_ASSET_CODE", ALLOW_ASSET_CODE);

  // Load env so resolveOrCreateEoa and other helpers can read from process.env
  loadEnvFile(ENV_FILE);

  // Bootstrap funder EOA if configured
  let funding: FundingConfig = { friendbotUrl };
  const funderSecret = process.env.EOA_FUNDER_SECRET;
  if (funderSecret) {
    const funderKeypair = Keypair.fromSecret(funderSecret);
    console.log(`Bootstrapping funder EOA via friendbot: ${funderKeypair.publicKey()}`);
    await fundAccount(friendbotUrl, funderKeypair.publicKey()).match(
      () => console.log("  Funder EOA funded."),
      (err) => fail("Failed to fund funder EOA", err),
    );
    const server = new rpc.Server(rpcUrl, { allowHttp: true });
    funding = {
      friendbotUrl,
      funder: {
        keypair: funderKeypair,
        server,
        networkPassphrase,
        balanceThreshold: process.env.FUNDER_BALANCE_THRESHOLD || "1",
      },
    };
  }

  // 2. Create and fund all role accounts
  console.log("\nSetting up role accounts...");
  const eoas: Record<string, Keypair> = {};
  for (const alias of EOA_ALIASES) {
    eoas[alias] = await resolveOrCreateEoa(alias, ENV_FILE, funding);
  }

  // Fund any additional EOAs that exist in the env (from previous runs)
  const allEoaSecrets = Object.entries(process.env)
    .filter((entry): entry is [string, string] => {
      const [k, v] = entry;
      return k.startsWith("EOA_") && k.endsWith("_SECRET") && !!v;
    })
    .map(([, v]) => {
      try {
        return Keypair.fromSecret(v);
      } catch {
        return null;
      }
    })
    .filter((k): k is Keypair => k !== null);

  if (allEoaSecrets.length > 0) {
    console.log(`Funding ${allEoaSecrets.length} EOA(s)...`);
    await fundAccounts(friendbotUrl, allEoaSecrets).match(
      () => console.log("All EOAs funded."),
      (err) => fail("Failed to fund EOAs", err),
    );
  }

  // 3. Generate attester keys
  console.log("\nGenerating attester keys...");
  const attesterWallet1 = Wallet.createRandom();
  const attesterWallet2 = Wallet.createRandom();
  const attesterWallets = [attesterWallet1, attesterWallet2].sort((a, b) => {
    const pubKeyA = a.signingKey.publicKey.slice(4);
    const pubKeyB = b.signingKey.publicKey.slice(4);
    for (let i = 0; i < pubKeyA.length; i += 2) {
      const byteA = Number.parseInt(pubKeyA.slice(i, i + 2), 16);
      const byteB = Number.parseInt(pubKeyB.slice(i, i + 2), 16);
      if (byteA > byteB) return 1;
      if (byteA < byteB) return -1;
    }
    return 0;
  });
  const sortedAttesters = attesterWallets.map((wallet) => Buffer.from(wallet.address.toLowerCase().slice(2), "hex"));
  const attesterHexList = sortedAttesters.map((b) => b.toString("hex")).join(",");
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_ATTESTERS", attesterHexList);
  upsertEnvFile(
    ENV_FILE,
    "MESSAGE_TRANSMITTER_ATTESTER_PRIVATE_KEYS",
    attesterWallets.map((w) => w.privateKey).join(","),
  );
  console.log(`  Generated ${attesterWallets.length} attester(s)`);

  // 4. Deploy stablecoin base (mint asset, allow asset, FiatTokenAdmin, SAC admin)
  const mintIssuer = await resolveOrCreateEoa(`${MINT_ASSET_CODE}_ISSUER`, ENV_FILE, funding);
  const allowIssuer = await resolveOrCreateEoa(`${ALLOW_ASSET_CODE.toUpperCase()}_ISSUER`, ENV_FILE, funding);

  console.log("\nDeploying stablecoin base...");
  const ftaSalt = createHash("sha256")
    .update(eoas.fiat_token_admin_deployer.publicKey())
    .update("fiat-token-admin")
    .digest();

  const ftaWasmPath = path.resolve(__dirname, "../../../target/wasm32v1-none/release/fiat_token_admin.wasm");

  const baseResult = await deployStablecoinBase({
    server: new rpc.Server(rpcUrl, { allowHttp: true }),
    networkPassphrase,
    funding,
    mintAssetIssuer: mintIssuer,
    mintAssetCode: MINT_ASSET_CODE,
    allowAssets: [{ assetIssuer: allowIssuer, assetCode: ALLOW_ASSET_CODE }],
    fiatTokenAdmin: {
      deployer: eoas.fiat_token_admin_deployer,
      owner: eoas.fiat_token_admin_owner.publicKey(),
      pauser: eoas.fiat_token_admin_pauser.publicKey(),
      admin: eoas.fiat_token_admin_admin.publicKey(),
      minterAssetController: eoas.fiat_token_admin_minter_asset_controller.publicKey(),
      blocklister: eoas.fiat_token_admin_blocklister.publicKey(),
    },
    wasmPath: ftaWasmPath,
    salt: ftaSalt,
  }).match(
    (r) => r,
    (err) => fail("Failed to deploy stablecoin base", err),
  );

  upsertEnvFile(ENV_FILE, `ASSET_${MINT_ASSET_CODE.toUpperCase()}_CONTRACT_ID`, baseResult.mintAsset.contractId);
  upsertEnvFile(ENV_FILE, `ASSET_${MINT_ASSET_CODE.toUpperCase()}_CODE`, MINT_ASSET_CODE);
  upsertEnvFile(ENV_FILE, `ASSET_${ALLOW_ASSET_CODE.toUpperCase()}_CONTRACT_ID`, baseResult.allowAssets[0].contractId);
  upsertEnvFile(ENV_FILE, `ASSET_${ALLOW_ASSET_CODE.toUpperCase()}_CODE`, ALLOW_ASSET_CODE);
  upsertEnvFile(ENV_FILE, "FIAT_TOKEN_ADMIN_CONTRACT_ID", baseResult.fiatTokenAdmin.contractId);
  upsertEnvFile(ENV_FILE, "FIAT_TOKEN_ADMIN_WASM_HASH", baseResult.fiatTokenAdmin.wasmHash);
  console.log(`  Mint asset:       ${baseResult.mintAsset.contractId}`);
  console.log(`  Allow asset:      ${baseResult.allowAssets[0].contractId}`);
  console.log(`  FiatTokenAdmin:   ${baseResult.fiatTokenAdmin.contractId}`);

  // 5. Deploy MessageTransmitter
  console.log("\nDeploying MessageTransmitter...");
  const localDomain = Number(process.env.MESSAGE_TRANSMITTER_LOCAL_DOMAIN ?? DEFAULT_LOCAL_DOMAIN);
  const mtVersion = Number(process.env.MESSAGE_TRANSMITTER_VERSION ?? DEFAULT_MESSAGE_VERSION);
  const signatureThreshold = Number(process.env.MESSAGE_TRANSMITTER_SIGNATURE_THRESHOLD ?? DEFAULT_SIGNATURE_THRESHOLD);
  const maxMessageBodySize = Number(
    process.env.MESSAGE_TRANSMITTER_MAX_MESSAGE_BODY_SIZE ?? DEFAULT_MAX_MESSAGE_BODY_SIZE,
  );

  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_LOCAL_DOMAIN", String(localDomain));
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_VERSION", String(mtVersion));
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_SIGNATURE_THRESHOLD", String(signatureThreshold));
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_MAX_MESSAGE_BODY_SIZE", String(maxMessageBodySize));

  const mtResult = await deployMessageTransmitter({
    rpcUrl,
    networkPassphrase,
    deployer: eoas.cctp_deployer,
    owner: eoas.message_transmitter_owner.publicKey(),
    pauser: eoas.message_transmitter_pauser.publicKey(),
    rescuer: eoas.message_transmitter_rescuer.publicKey(),
    attesterManager: eoas.message_transmitter_attester_manager.publicKey(),
    admin: eoas.message_transmitter_admin.publicKey(),
    attesters: sortedAttesters,
    signatureThreshold,
    maxMessageBodySize,
    localDomain,
    version: mtVersion,
  }).match(
    (r) => r,
    (err) => fail("Failed to deploy MessageTransmitter", err),
  );
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_CONTRACT_ID", mtResult.contractId);
  upsertEnvFile(ENV_FILE, "MESSAGE_TRANSMITTER_WASM_HASH", mtResult.wasmHash);
  console.log(`  Contract ID: ${mtResult.contractId}`);

  // 6. Deploy TokenMessengerMinter
  console.log("Deploying TokenMessengerMinter...");
  const messageBodyVersion = Number(
    process.env.TOKEN_MESSENGER_MINTER_MESSAGE_BODY_VERSION ?? DEFAULT_MESSAGE_BODY_VERSION,
  );
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_MESSAGE_BODY_VERSION", String(messageBodyVersion));

  const tmmResult = await deployTokenMessengerMinter({
    rpcUrl,
    networkPassphrase,
    deployer: eoas.cctp_deployer,
    owner: eoas.token_messenger_minter_owner.publicKey(),
    pauser: eoas.token_messenger_minter_pauser.publicKey(),
    rescuer: eoas.token_messenger_minter_rescuer.publicKey(),
    tokenController: eoas.token_messenger_minter_token_controller.publicKey(),
    admin: eoas.token_messenger_minter_admin.publicKey(),
    feeRecipient: eoas.token_messenger_minter_fee_recipient.publicKey(),
    minFeeController: eoas.token_messenger_minter_min_fee_controller.publicKey(),
    denylister: eoas.token_messenger_minter_denylister.publicKey(),
    messageTransmitter: mtResult.contractId,
    messageBodyVersion,
    remoteDomains: [],
    remoteTokenMessengers: [],
  }).match(
    (r) => r,
    (err) => fail("Failed to deploy TokenMessengerMinter", err),
  );
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_CONTRACT_ID", tmmResult.contractId);
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_WASM_HASH", tmmResult.wasmHash);
  console.log(`  Contract ID: ${tmmResult.contractId}`);

  // 7. Deploy CctpForwarder
  console.log("Deploying CctpForwarder...");
  const cfExpectedMessageVersion = Number(
    process.env.CCTP_FORWARDER_EXPECTED_MESSAGE_VERSION ?? DEFAULT_MESSAGE_VERSION,
  );
  const cfExpectedBurnMessageVersion = Number(
    process.env.CCTP_FORWARDER_EXPECTED_BURN_MESSAGE_VERSION ?? DEFAULT_MESSAGE_BODY_VERSION,
  );
  upsertEnvFile(ENV_FILE, "CCTP_FORWARDER_EXPECTED_MESSAGE_VERSION", String(cfExpectedMessageVersion));
  upsertEnvFile(ENV_FILE, "CCTP_FORWARDER_EXPECTED_BURN_MESSAGE_VERSION", String(cfExpectedBurnMessageVersion));

  const cfResult = await deployCctpForwarder({
    rpcUrl,
    networkPassphrase,
    deployer: eoas.cctp_deployer,
    owner: eoas.cctp_forwarder_owner.publicKey(),
    pauser: eoas.cctp_forwarder_pauser.publicKey(),
    rescuer: eoas.cctp_forwarder_rescuer.publicKey(),
    admin: eoas.cctp_forwarder_admin.publicKey(),
    messageTransmitter: mtResult.contractId,
    tokenMessengerMinter: tmmResult.contractId,
    expectedMessageVersion: cfExpectedMessageVersion,
    expectedBurnMessageVersion: cfExpectedBurnMessageVersion,
  }).match(
    (r) => r,
    (err) => fail("Failed to deploy CctpForwarder", err),
  );
  upsertEnvFile(ENV_FILE, "CCTP_FORWARDER_CONTRACT_ID", cfResult.contractId);
  upsertEnvFile(ENV_FILE, "CCTP_FORWARDER_WASM_HASH", cfResult.wasmHash);
  console.log(`  Contract ID: ${cfResult.contractId}`);

  // 8. Add remote token messengers and link token pairs
  console.log("\nConfiguring CCTP...");

  // Read or generate remote chain config (parallel arrays)
  const remoteDomainsCsv = process.env.TOKEN_MESSENGER_MINTER_REMOTE_DOMAINS;
  const remoteTokenMessengersCsv = process.env.TOKEN_MESSENGER_MINTER_REMOTE_TOKEN_MESSENGERS;
  const remoteTokensCsv = process.env.TOKEN_MESSENGER_MINTER_REMOTE_TOKENS;

  let remoteDomains: number[];
  let remoteTokenMessengerHexes: string[];
  let remoteTokenHexes: string[];

  if (remoteDomainsCsv && remoteDomainsCsv.trim() !== "") {
    // Use provided values
    remoteDomains = remoteDomainsCsv.split(",").map((s) => Number(s.trim()));
    remoteTokenMessengerHexes = (remoteTokenMessengersCsv ?? "").split(",").map((s) => s.trim());
    remoteTokenHexes = (remoteTokensCsv ?? "").split(",").map((s) => s.trim());
  } else {
    // Default: single remote chain at domain 0 with random addresses
    remoteDomains = [DEFAULT_REMOTE_DOMAIN];
    remoteTokenMessengerHexes = [randomBytes(32).toString("hex")];
    remoteTokenHexes = [randomBytes(32).toString("hex")];
  }

  // Validate parallel arrays have the same length
  if (remoteDomains.length !== remoteTokenMessengerHexes.length || remoteDomains.length !== remoteTokenHexes.length) {
    await fail(
      "REMOTE_DOMAINS, REMOTE_TOKEN_MESSENGERS, and REMOTE_TOKENS must have the same number of entries",
      new Error(
        `lengths: domains=${remoteDomains.length}, messengers=${remoteTokenMessengerHexes.length}, tokens=${remoteTokenHexes.length}`,
      ),
    );
  }

  // Write plural vars to env
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_REMOTE_DOMAINS", remoteDomains.join(","));
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_REMOTE_TOKEN_MESSENGERS", remoteTokenMessengerHexes.join(","));
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_REMOTE_TOKENS", remoteTokenHexes.join(","));

  const tmmOwnerClient = new TokenMessengerMinterV2Client({
    contractId: tmmResult.contractId,
    rpcUrl,
    networkPassphrase,
    allowHttp: true,
    ...createSigner(eoas.token_messenger_minter_owner, networkPassphrase),
  });

  // 9. Configure TMM as minter on FiatTokenAdmin (using AllowAsset)
  console.log("  Configuring TMM as minter on FiatTokenAdmin...");
  const controllerClient = createClientFactory(
    FiatTokenAdminClient,
    baseResult.fiatTokenAdmin.contractId,
    rpcUrl,
    networkPassphrase,
  )(eoas.fiat_token_admin_minter_asset_controller);

  await (
    await controllerClient.configure_minter({
      minter: tmmResult.contractId,
      allow_asset: baseResult.allowAssets[0].contractId,
    })
  ).signAndSend();
  console.log("  TMM configured as minter.");

  // 9b. Mint AllowAsset to TMM and authorize TMM on AllowAsset
  //     The AllowAsset has AuthRequired, so TMM needs authorization before it can burn.
  //     TMM also needs a pre-funded AllowAsset balance for the swap_mint flow.
  const server = new rpc.Server(rpcUrl, { allowHttp: true });
  const allowAssetId = baseResult.allowAssets[0].contractId;
  const tmmFundingAmount = DEFAULT_MAX_BURN_AMOUNT * 1000n;

  const allowSac = new SacClient(server, allowAssetId, allowIssuer, networkPassphrase);
  await allowSac.setAuthorized(tmmResult.contractId, true).match(
    () => console.log("  TMM authorized on AllowAsset."),
    (err) => fail("Failed to authorize TMM on AllowAsset", err),
  );

  console.log("  Funding TMM with AllowAsset...");
  await invokeContract<void>(server, new Contract(allowAssetId), allowIssuer, networkPassphrase, "mint", [
    new Address(tmmResult.contractId).toScVal(),
    xdr.ScVal.scvI128(
      new xdr.Int128Parts({
        lo: xdr.Uint64.fromString(tmmFundingAmount.toString()),
        hi: xdr.Int64.fromString("0"),
      }),
    ),
  ]).match(
    () => console.log(`  TMM funded with ${tmmFundingAmount} AllowAsset.`),
    (err) => fail("Failed to mint AllowAsset to TMM", err),
  );

  // 10. Add remote token messengers and link token pairs
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_LOCAL_TOKEN_CODE", MINT_ASSET_CODE.toUpperCase());

  const tmmTokenController = new TokenMessengerMinterV2Client({
    contractId: tmmResult.contractId,
    rpcUrl,
    networkPassphrase,
    allowHttp: true,
    ...createSigner(eoas.token_messenger_minter_token_controller, networkPassphrase),
  });

  await linkRemoteResources({
    localToken: baseResult.mintAsset.contractId,
    remoteDomains,
    remoteTokenMessengers: remoteTokenMessengerHexes.map(hexToBuffer),
    remoteTokens: remoteTokenHexes.map(hexToBuffer),
    ownerClient: tmmOwnerClient,
    tokenControllerClient: tmmTokenController,
  });

  // 11. Set token decimal config
  const localDecimals = Number(process.env.TOKEN_MESSENGER_MINTER_LOCAL_DECIMALS ?? DEFAULT_LOCAL_DECIMALS);
  const canonicalDecimals = Number(process.env.TOKEN_MESSENGER_MINTER_CANONICAL_DECIMALS ?? DEFAULT_CANONICAL_DECIMALS);
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_LOCAL_DECIMALS", String(localDecimals));
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_CANONICAL_DECIMALS", String(canonicalDecimals));

  await (
    await tmmTokenController.set_token_decimal_config({
      local_token: baseResult.mintAsset.contractId,
      local_decimals: localDecimals,
      canonical_decimals: canonicalDecimals,
    })
  ).signAndSend();
  console.log(`  Token decimal config set (local=${localDecimals}, canonical=${canonicalDecimals}).`);

  // 12. Set swap minter config
  await (
    await tmmTokenController.set_swap_minter_config({
      local_token: baseResult.mintAsset.contractId,
      swap_minter: baseResult.fiatTokenAdmin.contractId,
      allow_asset: baseResult.allowAssets[0].contractId,
    })
  ).signAndSend();
  console.log("  Swap minter config set.");

  // 13. Set max burn amount
  const maxBurnAmount = BigInt(process.env.TOKEN_MESSENGER_MINTER_MAX_BURN_AMOUNT ?? DEFAULT_MAX_BURN_AMOUNT);
  upsertEnvFile(ENV_FILE, "TOKEN_MESSENGER_MINTER_MAX_BURN_AMOUNT", maxBurnAmount.toString());

  await (
    await tmmTokenController.set_max_burn_amount_per_message({
      local_token: baseResult.mintAsset.contractId,
      burn_limit_per_message: maxBurnAmount,
    })
  ).signAndSend();
  console.log(`  Max burn amount set (${maxBurnAmount}).`);

  // 14. Create USDC trustline for recipient EOA
  await createTrustline(server, eoas.recipient, MINT_ASSET_CODE, mintIssuer.publicKey(), networkPassphrase).match(
    () => console.log("  USDC trustline created for recipient."),
    () => {
      /* trustline may already exist */
    },
  );

  // 15. Summary
  console.log("\n========== Local CCTP Environment Ready ==========\n");
  console.log(`  .env.local:           ${ENV_FILE}`);
  console.log(`  RPC URL:              ${rpcUrl}`);
  console.log(`  Mint asset:           ${MINT_ASSET_CODE} (${baseResult.mintAsset.contractId})`);
  console.log(`  Allow asset:          ${ALLOW_ASSET_CODE} (${baseResult.allowAssets[0].contractId})`);
  console.log(`  FiatTokenAdmin:       ${baseResult.fiatTokenAdmin.contractId}`);
  console.log(`  MessageTransmitter:   ${mtResult.contractId}`);
  console.log(`  TokenMessengerMinter: ${tmmResult.contractId}`);
  console.log(`  CctpForwarder:        ${cfResult.contractId}`);
  console.log(`  Remote Domains:       ${remoteDomains.join(", ")}`);
  console.log(`\nPress Ctrl-C to stop the node.\n`);

  // Keep alive
  await new Promise<void>((resolve) => {
    const keepAlive = setInterval(() => {}, 1 << 30);
    let shuttingDown = false;
    const shutdown = async () => {
      if (shuttingDown) {
        resolve();
        return;
      }
      shuttingDown = true;
      clearInterval(keepAlive);
      console.log("\nShutting down...");
      await stop();
      resolve();
    };
    process.on("SIGINT", shutdown);
    process.on("SIGTERM", shutdown);
  });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
