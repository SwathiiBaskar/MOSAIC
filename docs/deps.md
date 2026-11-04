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
| Review date | 2026-11-04 |
| Packages in `Cargo.lock` | 51 (1 first-party, 50 from crates.io) |
| Advisory scan | `cargo-audit` 0.22.1 with `--deny warnings` against the RustSec advisory database at revision `965bf9ad` (2026-11-03): no advisories for the locked set |
| Licence, source and ban policy | `deny.toml`, enforced by `cargo-deny` 0.20.2: licences limited to MIT and Apache-2.0, crates.io registry only, wildcard version requirements and yanked releases denied; passes for the supported targets |
| Yanked releases | None |
| Releases younger than 14 days | None. `rustix` is held at 1.1.5 because 1.1.6 was published on 2026-10-27 |
| Project licence | No licence has been selected for the project; all workspace crates are marked `publish = false` |

Licence strings are the SPDX expressions declared in the crates.io metadata of
each locked version. Where an expression offers a choice, the permissive
alternative applies: `r-efi` also offers LGPL-2.1-or-later, which is not used.
`unicode-ident` additionally carries the permissive Unicode-3.0 licence for its
data tables (an `AND` condition, not a choice).

Crates in the locked set with no release in the last two years: `core_detect`
(2021), `fnv` (2020), `quick-error` (2020), `unarray` (2022) and `num-traits`
(2024). All five are development-only.

`cargo-deny` evaluates the dependency graph of the supported targets. In the
version in use that graph does not include crates reached only through
development dependencies, so `cargo-audit` is the check that covers every
package in `Cargo.lock`, and the licences of the development-only crates are
recorded in the tables below.

## Direct dependencies

| Crate | Section | Purpose |
| --- | --- | --- |
| getrandom | `[dependencies]` of `mosaic-core` | Operating-system entropy for `Id::try_generate` |
| proptest | `[dev-dependencies]` of `mosaic-core` | Property-based tests |

## Runtime dependency closure of `mosaic-core`

| Crate | Version | Licence | Released |
| --- | --- | --- | --- |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | 2026-09-16 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | 2026-06-17 |
| libc | 0.2.191 | MIT OR Apache-2.0 | 2026-10-16 |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | 2026-02-03 |

`r-efi` is selected only for UEFI targets and is not compiled on Linux.

## Development-only dependencies

These crates are reached only through `proptest` and are not part of any
shipped artefact.

| Crate | Version | Licence | Released |
| --- | --- | --- | --- |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | 2026-05-22 |
| bit-set | 0.11.1 | Apache-2.0 OR MIT | 2026-07-10 |
| bit-vec | 0.10.1 | Apache-2.0 OR MIT | 2026-07-10 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | 2026-09-10 |
| borsh | 1.8.1 | MIT OR Apache-2.0 | 2026-08-26 |
| borsh-derive | 1.8.1 | Apache-2.0 | 2026-08-26 |
| bytes | 1.12.1 | MIT | 2026-07-08 |
| cfg_aliases | 0.2.2 | MIT | 2026-07-16 |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | 2026-08-27 |
| core_detect | 1.0.0 | MIT/Apache-2.0 | 2021-05-09 |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | 2026-08-26 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | 2026-10-13 |
| errno | 0.3.14 | MIT OR Apache-2.0 | 2025-09-09 |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | 2026-07-19 |
| fnv | 1.0.7 | Apache-2.0 / MIT | 2020-05-14 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | 2026-05-09 |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | 2026-09-05 |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 2025-12-23 |
| memchr | 2.8.3 | Unlicense OR MIT | 2026-07-08 |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | 2024-05-03 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | 2026-03-12 |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | 2026-03-04 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | 2026-07-19 |
| proptest | 1.12.0 | MIT OR Apache-2.0 | 2026-10-10 |
| quick-error | 1.2.3 | MIT/Apache-2.0 | 2020-01-02 |
| quote | 1.0.47 | MIT OR Apache-2.0 | 2026-07-19 |
| rand | 0.10.3 | MIT OR Apache-2.0 | 2026-09-20 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | 2026-04-13 |
| rand_xorshift | 0.5.0 | MIT OR Apache-2.0 | 2026-02-02 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | 2026-06-09 |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 2026-09-16 |
| rusty-fork | 0.3.1 | MIT/Apache-2.0 | 2025-10-04 |
| serde | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | 2026-07-18 |
| syn | 3.0.9 | MIT OR Apache-2.0 | 2026-10-12 |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | 2026-03-11 |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-08 |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-09 |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | 2026-10-09 |
| unarray | 0.1.4 | MIT OR Apache-2.0 | 2022-11-30 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | 2026-10-12 |
| wait-timeout | 0.2.1 | MIT/Apache-2.0 | 2025-02-03 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | 2025-10-06 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | 2025-10-06 |
| winnow | 1.0.4 | MIT | 2026-07-13 |

## Build and continuous-integration tooling

These tools run in `.github/workflows/ci.yml`. Versions are pinned there; the
third-party actions are pinned to full revision hashes.

| Tool | Version | Licence | Released | Role |
| --- | --- | --- | --- | --- |
| cargo-deny | 0.20.2 | MIT OR Apache-2.0 | 2026-07-09 | Licence, source and ban policy (`deny.toml`) |
| cargo-nextest | 0.9.148 | Apache-2.0 OR MIT | 2026-10-08 | Test runner |
| cargo-audit | 0.22.1 | Apache-2.0 OR MIT | 2026-02-04 | Advisory scan of `Cargo.lock` |
| gitleaks | 8.30.1 | MIT | 2026-03-21 | Secret scan of repository history |

The gitleaks release archive is checked against the SHA-256 digest
`551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb` before it is
unpacked.

| Action | Release | Licence | Released | Revision |
| --- | --- | --- | --- | --- |
| actions/checkout | v7.0.1 | MIT | 2026-07-17 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| taiki-e/install-action | v2.87.29 | Apache-2.0 OR MIT | 2026-10-16 | `8cca0918f8471c33515202b73f56ab27f2bed283` |

`taiki-e/install-action` installs `cargo-nextest`, `cargo-deny` and
`cargo-audit` at the versions above. The aarch64 jobs install
`gcc-aarch64-linux-gnu` and `qemu-user` from the runner's Ubuntu 24.04 archive.
