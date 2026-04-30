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

/**
 * Parses a comma-separated env var value into an array of trimmed strings.
 * Returns an empty array if the value is empty or undefined.
 */
export const parseCommaSeparated = (value: string | undefined): string[] => {
  if (!value || value.trim() === "") {
    return [];
  }
  return value.split(",").map((s) => s.trim());
};

/**
 * Parses a hex string (with optional 0x prefix) into a Buffer.
 */
export const hexToBuffer = (hex: string): Buffer => {
  const cleaned = hex.startsWith("0x") ? hex.slice(2) : hex;
  return Buffer.from(cleaned, "hex");
};
