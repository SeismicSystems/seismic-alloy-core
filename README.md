# Seismic Alloy Core

This repository contains Seismic's fork of alloy-core

The upstream repository lives [here](https://github.com/alloy-rs/alloy-core). This fork is up-to-date with it through commit `c268e8d`. You can see this by viewing the [main](https://github.com/SeismicSystems/seismic-alloy-core/tree/main) branch on this repository

You can view all of our changes vs. upstream on this [pull request](https://github.com/SeismicSystems/seismic-alloy-core/pull/30). The sole purpose of this PR is display our diff; it will never be merged in to the main branch of this repo

## Main Changes

The repository was forked to support Seismic's [modifications](https://github.com/SeismicSystems/seismic-solidity) to [Solidity](https://github.com/ethereum/solidity). Seismic introduces new types that represent shielded state in smart contracts.

### New Shielded Types

These shielded types include:

- `saddress`
- `suint` and the `suint{n}` family
- `sint` and the `sint{n}` family
- `sbool`

Each of these types behaves similarly to its unshielded counterpart, with one key exception: the values are hidden from the state tree.

## Structure

Seismic's forks of the [reth](https://github.com/paradigmxyz/reth) stack all have the same branch structure:

- `main` or `master`: this branch only consists of commits from the upstream repository. However it will rarely be up-to-date with upstream. The latest commit from this branch reflects how recently Seismic has merged in upstream commits to the seismic branch
- `seismic`: the default and production branch for these repositories. This includes all Seismic-specific code essential to make our network run

## Overview

This repository contains the following crates:

- [`alloy-core`]: Meta-crate for the entire project
- [`alloy-primitives`] - Primitive integer and byte types
- [`alloy-sol-types`] - Compile-time [ABI] and [EIP-712] implementations
- [`alloy-sol-macro`] - The [`sol!`] procedural macro
- [`alloy-dyn-abi`] - Run-time [ABI] and [EIP-712] implementations
- [`alloy-json-abi`] - Full Ethereum [JSON-ABI] implementation
- [`alloy-sol-type-parser`] - A simple parser for Solidity type strings
- [`syn-solidity`] - [`syn`]-powered Solidity parser

[`alloy-core`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/core
[`alloy-primitives`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/primitives
[`alloy-sol-types`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/sol-types
[`alloy-sol-macro`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/sol-macro
[`alloy-dyn-abi`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/dyn-abi
[`alloy-json-abi`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/json-abi
[`alloy-sol-type-parser`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/sol-type-parser
[`syn-solidity`]: https://github.com/SeismicSystems/alloy-core/tree/seismic/crates/syn-solidity
[JSON-ABI]: https://docs.soliditylang.org/en/latest/abi-spec.html#json
[ABI]: https://docs.soliditylang.org/en/latest/abi-spec.html
[EIP-712]: https://eips.ethereum.org/EIPS/eip-712
[`sol!`]: https://docs.rs/alloy-sol-macro/latest/alloy_sol_macro/macro.sol.html
[`syn`]: https://github.com/dtolnay/syn

## Credits

None of these crates would have been possible without the great work done in:

- [`ethers.js`](https://github.com/ethers-io/ethers.js/)
- [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
- [`ruint`](https://github.com/recmo/uint)
- [`ethabi`](https://github.com/rust-ethereum/ethabi)
- [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)
- [`guac_rs`](https://github.com/althea-net/guac_rs/)
- and of course: [`alloy-core`](https://github.com/alloy-rs/alloy-core/)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in these crates by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>
