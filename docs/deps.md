# Dependency register

Every third-party crate in the resolved dependency graph is recorded here with
its licence, release date and role. The lockfile (`Cargo.lock`) pins the exact
versions listed.

## Acceptance criteria

A crate is admitted when all of the following hold:

- Its declared licence offers a permissive option compatible with distributing
  the project under Apache-2.0 or MIT terms.
- It has an identifiable upstream repository and release history. Crates with
  no release in the last two years are listed explicitly below.
- It has no unresolved entry in the RustSec advisory database.
- It supports Linux on x86_64 and aarch64.
- Its transitive dependency tree is proportionate to what the crate provides.

## Review record

| Item | Result |
| --- | --- |
| Review date | 2026-11-05 |
| Packages in `Cargo.lock` | 82 (2 first-party, 80 from crates.io) |
| Advisory scan | `cargo-audit` 0.22.1 with `--deny warnings` against the RustSec advisory database at revision `26fd2d43` (2026-11-05): no advisories for the locked set |
| Licence, source and ban policy | `deny.toml`, enforced by `cargo-deny` 0.20.2: licences limited to MIT and Apache-2.0 with the two exceptions below, crates.io registry only, wildcard version requirements and yanked releases denied; passes for the supported targets |
| Yanked releases | None |
| Releases younger than 14 days | None. `log` is held at 0.4.34 because 0.4.35 was published on 2026-10-30, and `rustix` is held at 1.1.5 because 1.1.6 was published on 2026-10-27 |
| Project licence | No licence has been selected for the project; all workspace crates are marked `publish = false` |

Licence strings are the SPDX expressions declared in the crates.io metadata of
each locked version. Where an expression offers a choice, the permissive
alternative applies: `r-efi` also offers LGPL-2.1-or-later, which is not used.

**Licence exceptions.** Two crates carry a permissive licence that is neither
MIT nor Apache-2.0. `deny.toml` allows each licence for that crate only.

| Crate | Licence | Why it is in the graph |
| --- | --- | --- |
| `unicode-ident` | `(MIT OR Apache-2.0) AND Unicode-3.0` | Reached only through the `prost-derive` procedural macro (`syn`, `quote`, `proc-macro2`). The Unicode-3.0 licence covers the Unicode data tables in the crate. |
| `foldhash` | Zlib | Reached only through the `prost-build` build dependency (`petgraph`, `hashbrown`). |

Both crates run on the build machine only. Neither is linked into a MOSAIC
binary: every dependency path to `unicode-ident` passes through a procedural
macro crate, and the only path to `foldhash` passes through a build dependency.

`cargo-deny` evaluates the dependency graph of the supported targets. In the
version in use that graph does not include crates reached only through
development dependencies, so `cargo-audit` is the check that covers every
package in `Cargo.lock`, and the licences of the development-only crates are
recorded in the tables below.

## Direct dependencies

| Crate | Used by | Section | Purpose |
| --- | --- | --- | --- |
| getrandom | `mosaic-core` | `[dependencies]` | Operating-system entropy for `Id::try_generate` |
| prost | `mosaic-proto` | `[dependencies]` | Protobuf encoding, decoding and message derives |
| prost-build | `mosaic-proto` | `[build-dependencies]` | Generates Rust types from the schemas |
| protox | `mosaic-proto` | `[build-dependencies]` | Compiles the schemas without a `protoc` installation |
| proptest | `mosaic-core`, `mosaic-proto` | `[dev-dependencies]` | Property-based tests |

## Runtime dependency closure

Crates linked into MOSAIC code. Some are selected only on particular platforms:
`r-efi` is compiled for UEFI targets only and not on Linux.

| Crate | Version | Licence | Released |
| --- | --- | --- | --- |
| bytes | 1.12.1 | MIT | 2026-07-08 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | 2026-09-16 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | 2026-06-17 |
| libc | 0.2.191 | MIT OR Apache-2.0 | 2026-10-16 |
| prost | 0.14.4 | Apache-2.0 | 2026-06-07 |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2026-02-03 |

## Compile-time dependencies

Procedural macros and the build-script tree of `mosaic-proto`. They run on the
build machine and are not part of any shipped artefact.

| Crate | Version | Licence | Released |
| --- | --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense OR MIT | 2026-08-03 |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | 2026-07-18 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | 2026-09-10 |
| either | 1.19.0 | MIT OR Apache-2.0 | 2026-10-06 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | 2026-10-13 |
| errno | 0.3.14 | MIT OR Apache-2.0 | 2025-09-09 |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | 2026-07-19 |
| fixedbitset | 0.5.7 | MIT OR Apache-2.0 | 2024-04-14 |
| fnv | 1.0.7 | Apache-2.0 / MIT | 2020-05-14 |
| foldhash | 0.1.5 | Zlib | 2025-03-15 |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | 2025-08-07 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | 2026-05-09 |
| heck | 0.5.0 | MIT OR Apache-2.0 | 2024-03-12 |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | 2026-09-05 |
| itertools | 0.14.0 | MIT OR Apache-2.0 | 2024-12-31 |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 2025-12-23 |
| log | 0.4.34 | MIT OR Apache-2.0 | 2026-08-22 |
| logos | 0.16.1 | MIT OR Apache-2.0 | 2026-01-30 |
| logos-codegen | 0.16.1 | MIT OR Apache-2.0 | 2026-01-30 |
| logos-derive | 0.16.1 | MIT OR Apache-2.0 | 2026-01-30 |
| memchr | 2.8.3 | Unlicense OR MIT | 2026-07-08 |
| miette | 7.6.0 | Apache-2.0 | 2025-04-27 |
| miette-derive | 7.6.0 | Apache-2.0 | 2025-04-27 |
| multimap | 0.10.1 | MIT OR Apache-2.0 | 2025-05-11 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | 2026-03-12 |
| petgraph | 0.8.3 | MIT OR Apache-2.0 | 2025-09-30 |
| prettyplease | 0.2.37 | MIT OR Apache-2.0 | 2025-08-19 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | 2026-07-19 |
| prost-build | 0.14.4 | Apache-2.0 | 2026-06-07 |
| prost-derive | 0.14.4 | Apache-2.0 | 2026-06-07 |
| prost-reflect | 0.16.5 | MIT OR Apache-2.0 | 2026-07-09 |
| prost-types | 0.14.4 | Apache-2.0 | 2026-06-07 |
| protox | 0.10.0 | MIT OR Apache-2.0 | 2026-10-07 |
| protox-parse | 0.10.0 | MIT OR Apache-2.0 | 2026-10-07 |
| quote | 1.0.47 | MIT OR Apache-2.0 | 2026-07-19 |
| regex | 1.13.1 | MIT OR Apache-2.0 | 2026-07-15 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | 2026-08-04 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | 2026-06-09 |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 2026-09-16 |
| syn | 2.0.119 | MIT OR Apache-2.0 | 2026-07-15 |
| syn | 3.0.9 | MIT OR Apache-2.0 | 2026-10-12 |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | 2026-03-11 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | 2026-09-23 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | 2026-09-23 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | 2026-10-12 |
| unicode-width | 0.1.14 | MIT OR Apache-2.0 | 2024-09-19 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | 2025-10-06 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | 2025-10-06 |

## Development-only dependencies

Reached only through `proptest`; not part of any shipped artefact.

| Crate | Version | Licence | Released |
| --- | --- | --- | --- |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | 2026-05-22 |
| bit-set | 0.11.1 | Apache-2.0 OR MIT | 2026-07-10 |
| bit-vec | 0.10.1 | Apache-2.0 OR MIT | 2026-07-10 |
| borsh | 1.8.1 | MIT OR Apache-2.0 | 2026-08-26 |
| borsh-derive | 1.8.1 | Apache-2.0 | 2026-08-26 |
| cfg_aliases | 0.2.2 | MIT | 2026-07-16 |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | 2026-08-27 |
| core_detect | 1.0.0 | MIT/Apache-2.0 | 2021-05-09 |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | 2026-08-26 |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | 2024-05-03 |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | 2026-03-04 |
| proptest | 1.12.0 | MIT OR Apache-2.0 | 2026-10-10 |
| quick-error | 1.2.3 | MIT/Apache-2.0 | 2020-01-02 |
| rand | 0.10.3 | MIT OR Apache-2.0 | 2026-09-20 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | 2026-04-13 |
| rand_xorshift | 0.5.0 | MIT OR Apache-2.0 | 2026-02-02 |
| rusty-fork | 0.3.1 | MIT/Apache-2.0 | 2025-10-04 |
| serde | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-08 |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-09 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-09 |
| unarray | 0.1.4 | MIT OR Apache-2.0 | 2022-11-30 |
| wait-timeout | 0.2.1 | MIT/Apache-2.0 | 2025-02-03 |
| winnow | 1.0.4 | MIT | 2026-07-13 |

Crates in the locked set with no release in the last two years: `core_detect` (2021), `fixedbitset` (2024), `fnv` (2020), `heck` (2024), `num-traits` (2024), `quick-error` (2020), `unarray` (2022), `unicode-width` (2024).

## Build and continuous-integration tooling

These tools run in `.github/workflows/ci.yml`. Versions are pinned there; the
third-party actions are pinned to full revision hashes.

| Tool | Version | Licence | Released | Role |
| --- | --- | --- | --- | --- |
| cargo-deny | 0.20.2 | MIT OR Apache-2.0 | 2026-07-09 | Licence, source and ban policy (`deny.toml`) |
| cargo-nextest | 0.9.148 | Apache-2.0 OR MIT | 2026-10-08 | Test runner |
| cargo-audit | 0.22.1 | Apache-2.0 OR MIT | 2026-02-04 | Advisory scan of `Cargo.lock` |
| gitleaks | 8.30.1 | MIT | 2026-03-21 | Secret scan of repository history |
| buf | 1.74.0 | Apache-2.0 | 2026-10-14 | Lint and breaking-change check of `proto/` |

The gitleaks and buf release binaries are checked against their published
SHA-256 digests before use: gitleaks
`551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb`, buf
`92326d957d613712669fb55f9e4340946a8142a92120e09008459118778273d2`.

| Action | Release | Licence | Released | Revision |
| --- | --- | --- | --- | --- |
| actions/checkout | v7.0.1 | MIT | 2026-07-17 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| taiki-e/install-action | v2.87.29 | Apache-2.0 OR MIT | 2026-10-16 | `8cca0918f8471c33515202b73f56ab27f2bed283` |

`taiki-e/install-action` installs `cargo-nextest`, `cargo-deny` and
`cargo-audit` at the versions above. The aarch64 jobs install
`gcc-aarch64-linux-gnu` and `qemu-user` from the runner's Ubuntu 24.04 archive.
