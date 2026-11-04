# Implementation state

This document describes the software as it exists in this repository. It lists
what is implemented, how it is built and checked, and its known limitations.

MOSAIC is a distributed operating fabric in which autonomous agents are
first-class processes. The repository currently contains the Rust workspace
foundation, the shared identifier types, and the automated quality gates.

## Toolchain and workspace

| Item | Value |
| --- | --- |
| Rust toolchain | 1.99.1 (`rust-toolchain.toml`, with `rustfmt` and `clippy`) |
| Edition | 2024 |
| Minimum supported Rust version | 1.99, equal to the pinned toolchain; not tested against an older compiler |
| Workspace resolver | 3 |
| Workspace members | `crates/mosaic-core` |
| Supported targets | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` |
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
resolved set, the build tooling, and the review results are in
[`docs/deps.md`](../docs/deps.md). Licences, sources and bans are enforced by
`deny.toml`.

## Configuration

None. The crates read no environment variables and no files.

## Quality gates

The workflow `.github/workflows/ci.yml` runs on every push to `main` and on
every pull request. Each job installs the toolchain pinned in
`rust-toolchain.toml`.

| Job | What it runs |
| --- | --- |
| `lint` | `cargo fmt --check`; Clippy with warnings denied; `cargo doc` with warnings denied |
| `test` | `cargo nextest run` and the documentation tests, on x86_64 |
| `test-aarch64` | The unit and integration tests cross-compiled for aarch64 and run under `qemu-user` |
| `build` | `cargo build` for both targets in the dev and release profiles |
| `supply-chain` | `cargo deny check` (licences, sources, bans); `cargo audit --deny warnings` over the whole lockfile |
| `secrets` | `gitleaks` over the full repository history |
| `policy` | `scripts/check-unsafe-policy.sh` and its `--self-test` |

Local equivalents of the individual gates:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo deny --locked check
cargo audit --deny warnings
sh scripts/check-unsafe-policy.sh
sh scripts/check-unsafe-policy.sh --self-test
```

Test inventory: for `mosaic-core`, 11 deterministic tests and 4 property-based
tests in `tests/ids.rs`, and one compile-fail documentation test showing that
identifier kinds cannot be interchanged. For the policy checker, 14 cases in
its self-test: twelve fixture workspaces that must pass or fail as labelled,
and two compilations of a probe crate that confirm the workspace `forbid`
cannot be overridden locally.

## Verification

Results with Rust 1.99.1 on Linux x86_64: formatting, Clippy, tests (including
documentation tests), the documentation build, debug and release builds, the
`cargo deny` policy, and the `cargo audit` scan (RustSec database revision
`965bf9ad`, no advisories) pass. The same tests pass for
`aarch64-unknown-linux-gnu` under `qemu-user`, and both targets build in the dev
and release profiles. The policy checker passes on the repository and passes its
self-test. `gitleaks` reports no findings in the repository history, and
reports a finding when a synthetic token is committed to a scratch repository.

The workflow file passes `actionlint`. Every command it runs has been executed
outside the workflow; the runner-specific steps (installing the toolchain with
`rustup` and the third-party actions) have not.

## Known limitations

- Identifiers order by creation time at millisecond granularity. Order within a
  millisecond, and across nodes with different clocks, is not guaranteed.
- Identifiers have no serialisation other than their text form and their 16
  raw bytes.
- aarch64 has been tested under emulation only, not on native hardware.
- `cargo deny` evaluates the dependency graph of the supported targets and does
  not include crates reached only through development dependencies; `cargo
  audit` covers every package in `Cargo.lock`.
- No project licence has been selected.
