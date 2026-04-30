# Git Hooks

This directory contains Git hooks for the Stellar CCTP.

## Setup

To enable these hooks, run the following command from the repository root:

```bash
git config core.hooksPath .githooks
```

Or run this one-liner to set it up:

```bash
chmod +x .githooks/pre-commit && git config core.hooksPath .githooks
```

## Available Hooks

### pre-commit

Runs before each commit to ensure:
- Code is properly formatted (`cargo fmt`)
- No linting issues exist (`cargo clippy`)

The hook automatically runs checks on the Soroban contracts in the `soroban/` directory.

If any check fails, the commit will be blocked until issues are fixed.

## Bypassing Hooks

If you need to bypass hooks in an emergency (not recommended):

```bash
git commit --no-verify
```

## Requirements

These hooks require:
- `rustfmt` - Install with: `rustup component add rustfmt`
- `clippy` - Install with: `rustup component add clippy`

Both are automatically installed if you're using the project's `rust-toolchain.toml` file.

