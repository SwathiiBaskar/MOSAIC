# Implementation state

This document describes the software as it exists in this repository. It lists
what is implemented, how it is built and checked, and its known limitations.

MOSAIC is a distributed operating fabric in which autonomous agents are
first-class processes. The repository currently contains the Rust workspace
foundation and the shared identifier types.

## Toolchain and workspace

| Item | Value |
| --- | --- |
| Rust toolchain | 1.99.1 (`rust-toolchain.toml`, with `rustfmt` and `clippy`) |
| Edition | 2024 |
| Minimum supported Rust version | 1.99, equal to the pinned toolchain; not tested against an older compiler |
| Workspace resolver | 3 |
| Workspace members | `crates/mosaic-core` |
| Line endings | LF, enforced by `.gitattributes` |

Workspace lints (inherited by every crate through `[lints] workspace = true`):

- `unsafe_code = "forbid"`; see [`docs/unsafe-policy.md`](../docs/unsafe-policy.md).
- `missing_docs` and `rust_2018_idioms` warn.
- Clippy: the default group plus `dbg_macro`, `todo`, `unimplemented`,
  `unwrap_used` and `expect_used` warn. Checks run with warnings denied.

## Crates

| Crate | Responsibility |
| --- | --- |
| `mosaic-core` | Foundation types shared by MOSAIC components |

### `mosaic-core::ids`

Typed, time-ordered identifiers: `Id<K>` over UUID version 7, with a sealed
kind marker `K`. Aliases and textual prefixes:

| Alias | Prefix | Names |
| --- | --- | --- |
| `Aid` | `ap_` | agent process |
| `CapabilityId` | `cap_` | capability |
| `CapsuleId` | `cps_` | capsule |
| `NodeId` | `nod_` | node |
| `CheckpointId` | `ckp_` | checkpoint |
| `FabricId` | `fab_` | fabric |
| `EventId` | `evt_` | journal event |
| `PageId` | `pg_` | context page |

Operations: `from_parts` (pure construction from a millisecond timestamp and
random bytes), `try_generate` (system clock and operating-system entropy),
`from_bytes`, `as_bytes`, `unix_ms`, `Display`, `FromStr`, and the standard
comparison and hashing traits. Errors are `IdError` and `GenerateError`.

Format and parsing rules: `<prefix>_<canonical lowercase hyphenated UUID>`,
parsed strictly. The rationale and trade-offs are in
[`ADR-18`](../docs/adr/ADR-18-typed-uuidv7-identifiers.md).

## Dependencies

Runtime: `getrandom` 0.4.3. Development: `proptest` 1.12.0. The complete
resolved set, with licences and review results, is in
[`docs/deps.md`](../docs/deps.md).

## Configuration

None. The crates read no environment variables and no files.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

Test inventory for `mosaic-core`: 11 deterministic tests and 4 property-based
tests in `tests/ids.rs`, and one compile-fail documentation test showing that
identifier kinds cannot be interchanged.

Results of the checks above with Rust 1.99.1 on Linux x86_64: formatting,
Clippy, tests and documentation build pass. The RustSec advisory scan of
`Cargo.lock` (`cargo-audit` 0.22.1, advisory database revision `2295466c`)
reports no advisories.

## Known limitations

- Identifiers order by creation time at millisecond granularity. Order within a
  millisecond, and across nodes with different clocks, is not guaranteed.
- Identifiers have no serialisation other than their text form and their 16
  raw bytes.
- The code has been built and tested on Linux x86_64 only.
- No project licence has been selected.
