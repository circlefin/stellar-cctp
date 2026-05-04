# Copyright 2026 Circle Internet Group, Inc. All rights reserved.
#
# SPDX-License-Identifier: Apache-2.0
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#!/usr/bin/env bash

set -euo pipefail

RUSTC_VERSION="1.93.0"

if ! command -v rustup &> /dev/null; then
  echo "rustup is not installed, installing with toolchain $RUSTC_VERSION"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain="$RUSTC_VERSION" -y
  . "$HOME/.cargo/env"
  if [ -n "${GITHUB_PATH:-}" ] && [ -d "$HOME/.cargo/bin" ]; then
    echo "$HOME/.cargo/bin" >> "$GITHUB_PATH"
  fi
fi

rustup toolchain install "$RUSTC_VERSION" --component rustfmt --component clippy
rustup override set "$RUSTC_VERSION"
rustup target add wasm32v1-none

rustup --version
rustup show
