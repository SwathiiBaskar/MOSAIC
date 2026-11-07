# ADR-17: Protobuf wire and journal contracts

Status: Accepted (2026-11-05)

Scope: the message contracts exchanged between nodes and persisted in journals
and checkpoints. REST interfaces are outside this record.

## Context

MOSAIC components exchange messages across the network and persist some of them
(journal events, checkpoints) for long periods. The contracts therefore have to
satisfy the following.

- Compatible evolution. A node must read what the previous release wrote, and a
  node must tolerate fields it does not know.
- A single source of truth for the shape of each message, from which the Rust
  types are generated rather than written by hand.
- Mechanical protection against accidental incompatibility, so a renumbered or
  retyped field is caught before it ships.
- Stable integrity checks. Hashes and signatures over messages must stay valid
  however a message is re-encoded by another implementation.

## Decision

**Protobuf (proto3) defines every wire and journal contract.** Schemas live in
`proto/mosaic/v1` and form the package `mosaic.v1`.

**Rust types are generated at build time** by the `mosaic-proto` crate. The
schemas are compiled by `protox`, a protobuf compiler written in Rust, and the
types are produced by `prost-build`, so building needs no `protoc` installation.

**Evolution rules for a package version:**

- Changes are additive: new fields and new messages only.
- A field number is never reused, renumbered or retyped. When a field is
  removed, its number is reserved.
- Readers ignore unknown fields. Enum fields are open: an unknown value is kept
  as its integer and is never mapped to a known name.
- A change that breaks these rules requires a new package version (`mosaic.v2`)
  rather than an edit to `mosaic.v1`.

**Enforcement.** `buf lint` (the STANDARD rule set) and `buf breaking` (the
WIRE_JSON category, which covers wire and JSON compatibility) run in
continuous integration. `mosaic-proto` additionally carries tests with
hand-written expected encodings that pin every field number.

**Enum value names are not prefixed with the enum name.** Label values are
`PUBLIC`, `INTERNAL`, `CONFIDENTIAL` and `LOCAL_ONLY`; side-effect classes are
`NONE`, `WORKSPACE` and `EXTERNAL`. These names are the vocabulary used in
policies, logs and documents, so the `ENUM_VALUE_PREFIX` lint rule is disabled
in `proto/buf.yaml`; every other STANDARD rule applies.

**Encoding is not canonical, so integrity covers stored bytes.** One message has
many valid encodings (field order, repeated scalar fields, packing), and
re-encoding a decoded message may produce different bytes. A hash or signature
is therefore always computed over the bytes that were stored or received, never
over a freshly encoded copy of the decoded message.

## Alternatives considered

- **JSON only.** Self-describing and easy to inspect, but it has no enforced
  schema, no compatibility tooling, ambiguous numeric types, and larger
  messages.
- **Cap'n Proto or FlatBuffers.** Both avoid a decode step, but `buf`, the tool
  that makes the evolution rules above mechanically enforceable, works on
  protobuf schemas only.
- **Committing generated code (for example with `buf generate`).** Keeps the
  build graph smaller, but needs a separate regeneration step and a check that
  the committed code matches the schemas. Build-time generation keeps one
  source of truth and a plain `cargo build`.
- **Canonical encoding rules, so that re-encoded messages hash identically.**
  Protobuf does not define one, and every implementation would have to follow a
  private convention. Hashing stored bytes needs no agreement beyond the bytes.

## Consequences

- Schema changes are cheap when additive and deliberately expensive otherwise.
- The build compiles the generator crates (`protox`, `prost-build` and their
  dependencies). Two of those crates carry permissive licences other than MIT
  and Apache-2.0; they are listed as exceptions in `deny.toml` and
  `docs/deps.md`. Neither is linked into a MOSAIC binary.
- Messages that carry digests use a `Hash` message holding 32 raw bytes. The
  generated type does not check the length.
- Because integrity covers stored bytes, a component that must hash a message
  keeps the original bytes alongside the decoded value.

## Evidence

- The `mosaic-proto` tests check the encoding of every message field by field
  against bytes derived from the protobuf encoding rules, and check that
  enum values and names match the specification.
- They show that out-of-order and duplicated fields decode to the expected
  message while re-encoding yields different bytes.
- They show that unknown fields are ignored, unknown enum numbers are not
  mapped, and truncated or malformed input is rejected without panicking.
- Property tests check that encoding and decoding round-trip for arbitrary
  checkpoints and envelopes, and that decoding arbitrary bytes never panics.
- `buf breaking` rejects a renumbered field, a retyped field and a removed
  field without a reservation, and accepts an added field.

## Revisit conditions

- A consumer needs a deterministic serialisation, for example to sign message
  contents rather than stored bytes.
- The build-time cost of the generator crates outweighs the convenience of
  generating during the build.
- Message volume or latency requirements make a decode step unacceptable.
