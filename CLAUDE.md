# Seismic Alloy-Core

Fork of [alloy-rs/core](https://github.com/alloy-rs/core) that adds **shielded type** support for Seismic's confidential EVM. Upstream is tracked through the `main` branch; all Seismic work lives on `seismic`.

## What This Does

Standard EVM storage is publicly readable. Seismic extends the Ethereum type system with **shielded types** — `suint`, `sint`, `saddress`, `sbool` — that represent values hidden from the state tree. This crate library provides Rust-side type definitions, ABI encoding/decoding, Solidity parsing, and macro support for these types. It also introduces `FlaggedStorage` — a storage value wrapper that tracks whether data is private or public.

## Build

Rust workspace (10 crates + 1 test crate). Edition 2024, MSRV 1.85.

### macOS

```bash
# Prerequisites: Rust stable >= 1.85
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build (default features include seismic)
cargo build --workspace

# Build with CI feature set
cargo build --features arbitrary,eip712,seismic
```

### Linux (Ubuntu)

```bash
# Prerequisites
sudo apt-get update && sudo apt-get install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --workspace

# Build with CI feature set
cargo build --features arbitrary,eip712,seismic
```

### Verify

```bash
cargo build --workspace 2>&1 | tail -1
# Expected: Finished `dev` profile [unoptimized + debuginfo] target(s) in ...
```

## Test

```bash
# Full test suite (593 tests, ~15s)
cargo test --workspace
```

All 593 tests should pass with 0 failures (22 ignored doc-tests are expected).

### Compile tests (trybuild, requires nightly)

```bash
# Update trybuild snapshots
./scripts/bless_tests.sh
# Or run directly:
cargo +nightly test -p alloy-sol-types --test compiletest -- --include-ignored
```

### Feature powerset validation (requires cargo-hack)

```bash
cargo install cargo-hack
./scripts/check_features.sh
```

### Formatting (requires nightly)

```bash
cargo +nightly fmt --all --check
```

### Clippy

```bash
cargo clippy --workspace
```

Warnings are expected (workspace uses `warn` level, not `deny`). CI checks for warnings via `RUSTFLAGS="-D warnings" cargo check`.

## Project Layout

```
crates/
  core/                Meta-crate re-exporting all others
  primitives/          Ethereum primitives (U256, Address, FixedBytes, FlaggedStorage)
  sol-types/           Compile-time Solidity type ABI encoding (SolType trait)
  sol-macro/           sol! procedural macro entry point
  sol-macro-expander/  Macro expansion logic
  sol-macro-input/     Macro input type parsing
  dyn-abi/             Runtime ABI encoding/decoding (DynSolType, DynSolValue)
  json-abi/            Full Ethereum JSON-ABI representation
  sol-type-parser/     Solidity type string parser (winnow-based)
  syn-solidity/        syn-powered Solidity parser (items, types, expressions)
tests/
  core-sol/            Integration tests
scripts/               Build/test automation (check_features.sh, bless_tests.sh, etc.)
```

## Key Seismic Modifications

All Seismic code is gated behind `#[cfg(feature = "seismic")]`. Default features include `seismic` in `alloy-sol-types` and `alloy-dyn-abi`.

- **Shielded primitives**: `crates/primitives/src/aliases.rs` — `SUint`, `SInt`, `SAddress` wrapper types
- **FlaggedStorage**: `crates/primitives/src/storage/flagged_storage.rs` — `U256` + `is_private` flag with `public()`/`private()` constructors
- **Solidity parsing**: `crates/syn-solidity/src/type/mod.rs` — `Sint`, `Suint`, `Saddress`, `Sbool` type variants
- **Type parser**: `crates/sol-type-parser/src/root.rs` — recognizes `"saddress"`, `"sbool"`, `"sint"`, `"suint"` strings
- **Compile-time ABI**: `crates/sol-types/src/types/data_type.rs` — `Suint`, `Sint`, `Saddress`, `Sbool` SolType impls
- **Runtime ABI**: `crates/dyn-abi/src/specifier.rs` — runtime shielded type resolution
- **JSON-ABI**: `crates/json-abi/src/` — shielded type support in ABI JSON serialization

## Code Style

Defined in `rustfmt.toml`:

- Max line width: **100** characters
- Imports: crate-level granularity, reordered
- Field init shorthand enabled
- Comments wrapped at 100 chars, formatted in doc comments
- Formatting requires **nightly** (`cargo +nightly fmt`)

Lints (workspace-level in `Cargo.toml`):

- Clippy: `use-self`, `uninlined-format-args`, `missing-const-for-fn`, `redundant-clone` (all warn)
- Rust: `missing-docs`, `missing-copy-implementations`, `rust-2018-idioms`, `unreachable-pub` (all warn)

## CI

### seismic.yml (Seismic branch)

| Job        | What it does                                                              |
| ---------- | ------------------------------------------------------------------------- |
| `rustfmt`  | `cargo +nightly fmt --all --check`                                        |
| `build`    | `cargo build` + `cargo build --features arbitrary,eip712,seismic`         |
| `warnings` | `RUSTFLAGS="-D warnings" cargo check` (with and without seismic features) |
| `test`     | `cargo test`                                                              |

### ci.yml (upstream main branch)

Full matrix: stable/nightly/MSRV, Ubuntu/Windows, feature powerset, miri, WASM, no_std, clippy, docs, deny.

## Branches

- `seismic` — main development branch (PR target)
- `main` — upstream tracking only

## Troubleshooting

| Problem                                                                                              | Fix                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--all-features` build fails: `#![feature]` may not be used on the stable release channel (foldhash) | The `nightly` feature requires nightly Rust. Use `cargo build --workspace` (default features) or specify features explicitly: `--features arbitrary,eip712,seismic` |
| `--no-default-features` build fails: `unresolved import alloy_sol_types::sol_data::Sbool`            | Known issue — `alloy-dyn-abi` imports `Sbool` unconditionally but it requires `seismic` feature. Default features include `seismic`, so normal builds work fine     |
| Clippy warnings about `use-self`, `const fn`, `From` impl                                            | Expected — workspace uses `warn` not `deny`. CI uses `RUSTFLAGS="-D warnings" cargo check` (not clippy)                                                             |
| `rustfmt` fails on stable                                                                            | Formatting config requires nightly: use `cargo +nightly fmt`                                                                                                        |
| `bless_tests.sh` fails on stable                                                                     | Trybuild snapshot updates require nightly: script runs `cargo +nightly test`                                                                                        |
