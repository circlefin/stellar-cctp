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

import {
  Address,
  Contract,
  type Keypair,
  type rpc,
  scValToNative,
  TransactionBuilder,
  xdr,
} from "@stellar/stellar-sdk";
import { ResultAsync } from "neverthrow";

// ==================== Types ====================

export interface ContractCallResult<T> {
  value: T;
  txHash: string;
  ledger: number;
}

// ==================== Contract Invocation ====================

/**
 * Invokes a contract method that modifies state (requires signing).
 */
export const invokeContract = <T>(
  server: rpc.Server,
  contract: Contract,
  caller: Keypair,
  networkPassphrase: string,
  method: string,
  args: xdr.ScVal[],
): ResultAsync<ContractCallResult<T>, Error> => {
  return ResultAsync.fromThrowable(
    async () => {
      const account = await server.getAccount(caller.publicKey());

      const tx = new TransactionBuilder(account, {
        fee: "1000000",
        networkPassphrase,
      })
        .addOperation(contract.call(method, ...args))
        .setTimeout(30)
        .build();

      tx.sign(caller);

      const prepared = await server.prepareTransaction(tx);
      prepared.sign(caller);
      const sendResult = await server.sendTransaction(prepared);

      let status = await server.getTransaction(sendResult.hash);
      while (status.status === "NOT_FOUND") {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        status = await server.getTransaction(sendResult.hash);
      }

      if (status.status !== "SUCCESS") {
        throw new Error(`Transaction failed: ${sendResult.hash}`);
      }

      const successStatus = status as rpc.Api.GetSuccessfulTransactionResponse;
      const returnValue = successStatus.returnValue ? scValToNative(successStatus.returnValue) : null;

      return {
        value: returnValue as T,
        txHash: sendResult.hash,
        ledger: successStatus.ledger,
      };
    },
    (e) => e as Error,
  )();
};

/**
 * Queries a contract method (read-only, uses simulation).
 */
export const queryContract = <T>(
  server: rpc.Server,
  contract: Contract,
  caller: Keypair,
  networkPassphrase: string,
  method: string,
  args: xdr.ScVal[] = [],
): ResultAsync<T, Error> => {
  return ResultAsync.fromThrowable(
    async () => {
      const account = await server.getAccount(caller.publicKey());

      const tx = new TransactionBuilder(account, {
        fee: "100",
        networkPassphrase,
      })
        .addOperation(contract.call(method, ...args))
        .setTimeout(30)
        .build();

      const simulated = await server.simulateTransaction(tx);

      if ("result" in simulated && simulated.result) {
        return scValToNative(simulated.result.retval) as T;
      }

      if ("error" in simulated) {
        throw new Error(`Simulation failed: ${simulated.error}`);
      }

      return null as T;
    },
    (e) => e as Error,
  )();
};

// ==================== SAC Client ====================

/**
 * Client for interacting with a Stellar Asset Contract (SAC).
 *
 * @param server - Soroban RPC server instance.
 * @param contractId - The SAC contract ID (C… address).
 * @param eoa - Keypair that signs and submits transactions. This may be the SAC admin
 *   (for admin operations like `set_authorized`, `mint`, `clawback`) or an ordinary
 *   account (for user operations like `transfer`, `approve`, `balance`).
 * @param networkPassphrase - Stellar network passphrase for transaction signing.
 */
export class SacClient {
  private contract: Contract;

  constructor(
    private server: rpc.Server,
    contractId: string,
    private eoa: Keypair,
    private networkPassphrase: string,
  ) {
    this.contract = new Contract(contractId);
  }

  setAuthorized(address: string, authorized: boolean): ResultAsync<ContractCallResult<void>, Error> {
    return invokeContract<void>(this.server, this.contract, this.eoa, this.networkPassphrase, "set_authorized", [
      new Address(address).toScVal(),
      xdr.ScVal.scvBool(authorized),
    ]);
  }

  transfer(to: string, amount: bigint): ResultAsync<ContractCallResult<void>, Error> {
    return invokeContract<void>(this.server, this.contract, this.eoa, this.networkPassphrase, "transfer", [
      new Address(this.eoa.publicKey()).toScVal(),
      new Address(to).toScVal(),
      xdr.ScVal.scvI128(
        new xdr.Int128Parts({ lo: xdr.Uint64.fromString(amount.toString()), hi: xdr.Int64.fromString("0") }),
      ),
    ]);
  }

  balance(address: string): ResultAsync<bigint, Error> {
    return queryContract<bigint>(this.server, this.contract, this.eoa, this.networkPassphrase, "balance", [
      new Address(address).toScVal(),
    ]);
  }

  approve(spender: string, amount: bigint, expirationLedger: number): ResultAsync<ContractCallResult<void>, Error> {
    return invokeContract<void>(this.server, this.contract, this.eoa, this.networkPassphrase, "approve", [
      new Address(this.eoa.publicKey()).toScVal(),
      new Address(spender).toScVal(),
      xdr.ScVal.scvI128(
        new xdr.Int128Parts({ lo: xdr.Uint64.fromString(amount.toString()), hi: xdr.Int64.fromString("0") }),
      ),
      xdr.ScVal.scvU32(expirationLedger),
    ]);
  }

  allowance(from: string, spender: string): ResultAsync<bigint, Error> {
    return queryContract<bigint>(this.server, this.contract, this.eoa, this.networkPassphrase, "allowance", [
      new Address(from).toScVal(),
      new Address(spender).toScVal(),
    ]);
  }

  mint(to: string, amount: bigint): ResultAsync<ContractCallResult<void>, Error> {
    return invokeContract<void>(this.server, this.contract, this.eoa, this.networkPassphrase, "mint", [
      new Address(to).toScVal(),
      xdr.ScVal.scvI128(
        new xdr.Int128Parts({ lo: xdr.Uint64.fromString(amount.toString()), hi: xdr.Int64.fromString("0") }),
      ),
    ]);
  }

  clawback(from: string, amount: bigint): ResultAsync<ContractCallResult<void>, Error> {
    return invokeContract<void>(this.server, this.contract, this.eoa, this.networkPassphrase, "clawback", [
      new Address(from).toScVal(),
      xdr.ScVal.scvI128(
        new xdr.Int128Parts({ lo: xdr.Uint64.fromString(amount.toString()), hi: xdr.Int64.fromString("0") }),
      ),
    ]);
  }
}
