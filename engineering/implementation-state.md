# Implementation state

This document describes the software as it exists in this repository. It lists
what is implemented, how it is built and checked, and its known limitations.

MOSAIC is a distributed operating fabric in which autonomous agents are
first-class processes. The repository currently contains the Rust workspace
foundation, the shared identifier types, the protobuf contracts for the first
shared messages, and the automated quality gates.

## Toolchain and workspace

| Item | Value |
| --- | --- |
| Rust toolchain | 1.99.1 (`rust-toolchain.toml`, with `rustfmt` and `clippy`) |
| Edition | 2024 |
| Minimum supported Rust version | 1.99, equal to the pinned toolchain; not tested against an older compiler |
| Workspace resolver | 3 |
| Workspace members | `crates/mosaic-core`, `crates/mosaic-proto` |
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
| `mosaic-proto` | Protobuf message types generated from `proto/mosaic/v1` |

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

### `mosaic-proto` and `proto/mosaic/v1`

The schemas in `proto/mosaic/v1` define package `mosaic.v1`; the crate generates
Rust types from them in its build script (`protox` compiles the schemas and
`prost-build` generates the code, so no `protoc` is needed). Every message is a
plain data type. Contract rules and their rationale are in
[`ADR-17`](../docs/adr/ADR-17-protobuf-wire-contracts.md).

| Message or enum | Contents |
| --- | --- |
| `Label` | `LABEL_UNSPECIFIED` 0, `PUBLIC` 1, `INTERNAL` 2, `CONFIDENTIAL` 3, `LOCAL_ONLY` 4 |
| `SideEffects` | `SE_UNSPECIFIED` 0, `NONE` 1, `WORKSPACE` 2, `EXTERNAL` 3 |
| `Hash` | `blake3` (field 1): a 32-byte BLAKE3-256 digest; the length is not checked by the type |
| `Budget` | `tokens` 1, `usd` 2, `gpu_seconds` 3, `wall_seconds` 4, `mem_bytes` 5, `cpu_millis` 6 |
| `Checkpoint` | `ckp_id` 1, `aid` 2, `journal_seq` 3, `journal_hash` 4, `workspace_root` 5, `page_table_root` 6, `caps_chain` 7, `ledger` 8, `epoch` 9 |
| `Envelope` | `trace_parent` 1, `deadline_unix_ms` 2, `idem_key` 3, `epoch` 4, and the `payload` oneof with `hello` 10 |
| `Hello` | `proto_min` 1, `proto_max` 2, `features` 3, `node_id` 4 |

Identifiers appear in messages as strings in the format described above; no
conversion between `mosaic-core` identifiers and message fields exists.

`cargo run -p mosaic-proto --example wire_tour` prints the wire encoding of a
`Hello`, an `Envelope` and a `Checkpoint`, and shows non-canonical encodings,
unknown-field handling and enum names.

## Dependencies

Runtime: `getrandom` 0.4.3 and `prost` 0.14.4. Build: `prost-build` 0.14.4 and
`protox` 0.10.0. Development: `proptest` 1.12.0. The complete resolved set, the
build tooling, and the review results are in
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
| `proto` | `buf lint` and `buf breaking` (WIRE_JSON) on `proto/` against the previous revision |

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
buf lint proto
buf breaking proto --against '.git#ref=HEAD,subdir=proto'
```

Test inventory: for `mosaic-core`, 11 deterministic tests and 4 property-based
tests in `tests/ids.rs`, and one compile-fail documentation test showing that
identifier kinds cannot be interchanged. For `mosaic-proto`, 11 deterministic
tests and 3 property-based tests in `tests/wire.rs`: expected encodings written
out by hand for every message, enum values and names, unknown-field and
unknown-enum handling, non-canonical encodings, malformed input, and encode and
decode round-trips. For the policy checker, 14 cases in
its self-test: twelve fixture workspaces that must pass or fail as labelled,
and two compilations of a probe crate that confirm the workspace `forbid`
cannot be overridden locally.

## Verification

Results with Rust 1.99.1 on Linux x86_64: formatting, Clippy, tests (including
documentation tests), the documentation build, debug and release builds, the
`cargo deny` policy, and the `cargo audit` scan (RustSec database revision
`26fd2d43`, no advisories) pass. `buf lint` passes on `proto/`, and `buf
breaking` rejects a renumbered field, a retyped field and a removed field
without a reservation, and accepts an added field. The same tests pass for
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
- `Envelope.payload` carries `Hello` only.
- The `Hash` type does not enforce its 32-byte length.
- aarch64 has been tested under emulation only, not on native hardware.
- `cargo deny` evaluates the dependency graph of the supported targets and does
  not include crates reached only through development dependencies; `cargo
  audit` covers every package in `Cargo.lock`.
- No project licence has been selected.
