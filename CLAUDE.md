# Seismic Alloy-Core

Fork of [alloy-rs/core](https://github.com/alloy-rs/core) that adds **shielded type** support for Seismic's confidential EVM. Upstream is tracked through the `main` branch; all Seismic work lives on `seismic`.

## What This Does

Standard EVM storage is publicly readable. Seismic extends the Ethereum type system with **shielded types** — `suint`, `sint`, `saddress`, `sbool` — that represent values hidden from the state tree. This crate provides Rust-side type definitions, ABI encoding/decoding, Solidity parsing, and macro support for these types. It also introduces `FlaggedStorage` — a storage value wrapper that tracks whether data is private or public.

## Branches

- `seismic` — main development branch (PR target)
- `main` — upstream tracking only (do not PR here)

## Build & Test

Rust workspace, edition 2024, MSRV 1.85.

```bash
# Build (default features include seismic)
cargo build --workspace

# Build with CI feature set
cargo build --features arbitrary,eip712,seismic

# Run all tests — all should pass with 0 failures
cargo test --workspace

# Formatting (requires nightly)
cargo +nightly fmt --all --check

# Clippy — warnings are expected (workspace uses warn, not deny)
# CI uses RUSTFLAGS="-D warnings" cargo check (not clippy)
cargo clippy --workspace

# Update trybuild snapshots (requires nightly)
./scripts/bless_tests.sh

# Feature powerset validation (requires cargo-hack)
./scripts/check_features.sh
```

## Seismic vs Upstream — What Changed

All Seismic code is gated behind `#[cfg(feature = "seismic")]`. Default features include `seismic` in `alloy-sol-types` and `alloy-dyn-abi`, so it's on by default for normal builds.

### Feature flag propagation

- `alloy-primitives` — defines the `seismic` feature (gate only, no deps)
- `alloy-sol-types` — default includes `seismic`, forwards to `alloy-primitives/seismic` + `alloy-sol-macro/seismic`
- `alloy-dyn-abi` — default includes `seismic`, forwards to `alloy-primitives`, `alloy-json-abi`, `alloy-sol-types`, `alloy-sol-type-parser`
- `alloy-core` — optional propagation to all sub-crates

### Modifications by layer

The changes follow the data path through the stack: primitives → parser → macro expansion → compile-time ABI → runtime ABI → JSON-ABI.

**1. Primitives** (`crates/primitives/`)
- `src/aliases.rs` — `SUint`, `SInt`, `SAddress` newtype wrappers (with size aliases SU8–SU256, SI8–SI256)
- `src/storage/flagged_storage.rs` — `FlaggedStorage { value: U256, is_private: bool }` with `public()`/`private()` constructors, RLP encoding, serde support
- `src/storage/storage_slot.rs` — `StorageSlot` and `PrivateSlot` traits

**2. Solidity type parser** (`crates/sol-type-parser/`)
- `src/root.rs` — recognizes `"saddress"`, `"sbool"`, `"sint"`, `"suint"`, `"sbytes"` type strings

**3. Solidity syntax tree** (`crates/syn-solidity/`)
- `src/type/mod.rs` — `Sint`, `Suint`, `Saddress`, `Sbool` variants in the `Type` enum

**4. Macro expansion** (`crates/sol-macro-expander/`)
- `src/expand/ty.rs` — expands shielded types to their `sol_data` counterparts

**5. Compile-time ABI** (`crates/sol-types/`)
- `src/types/data_type.rs` — `SolType` impls for `Sbool`, `Saddress`, `Sint<BITS>`, `Suint<BITS>` (32-byte ABI encoding, sealed `SupportedSint` trait)
- `src/types/value.rs` — value encoding for shielded types
- `src/types/event/topic.rs` — event topic handling

**6. Runtime ABI** (`crates/dyn-abi/`)
- `src/dynamic/ty.rs` — `DynSolType` enum variants: `Saddress`, `Sint(usize)`, `Suint(usize)`, `Sbool`, `Sbytes(usize)`
- `src/dynamic/value.rs` — encode/decode for shielded `DynSolValue` variants
- `src/specifier.rs` — runtime resolution of shielded type strings to `DynSolType`
- `src/eip712/coerce.rs` — EIP-712 structured data support
- `src/coerce.rs` — type coercion for shielded types
- `src/arbitrary.rs` — property-based testing support

**7. JSON-ABI** (`crates/json-abi/`)
- Feature-flagged shielded type support in ABI JSON serialization

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

## Code Style

- Max line width: **100** characters (see `rustfmt.toml`)
- Formatting requires **nightly**: `cargo +nightly fmt`
- Lints are configured at workspace level in `Cargo.toml` (all `warn`, not `deny`)

## CI (seismic.yml)

| Job        | What it does                                                              |
| ---------- | ------------------------------------------------------------------------- |
| `rustfmt`  | `cargo +nightly fmt --all --check`                                        |
| `build`    | `cargo build` + `cargo build --features arbitrary,eip712,seismic`         |
| `warnings` | `RUSTFLAGS="-D warnings" cargo check` (with and without seismic features) |
| `test`     | `cargo test`                                                              |

Upstream `ci.yml` runs on `main` only (full matrix: stable/nightly/MSRV, feature powerset, miri, WASM, no_std, clippy, docs, deny).

## Troubleshooting

| Problem | Fix |
| --- | --- |
| `--all-features` build fails (`#![feature]` on stable) | The `nightly` feature requires nightly Rust. Use `cargo build --workspace` or `--features arbitrary,eip712,seismic` |
| `--no-default-features` build fails (`unresolved import Sbool`) | Known issue — `alloy-dyn-abi` imports `Sbool` unconditionally but it requires `seismic` feature. Default features include `seismic`, so normal builds work fine |
| Clippy warnings | Expected — workspace uses `warn` not `deny`. CI uses `RUSTFLAGS="-D warnings" cargo check` (not clippy) |
| `rustfmt` fails on stable | Formatting config requires nightly: `cargo +nightly fmt` |
| `bless_tests.sh` fails on stable | Trybuild snapshots require nightly: script runs `cargo +nightly test` |
