# ADR-18: Typed UUIDv7 identifiers

Status: Accepted (2026-11-02)

## Context

MOSAIC names several kinds of object: agent processes, capabilities, capsules,
nodes, checkpoints, fabrics, journal events and context pages. Identifiers for
these objects must satisfy the following.

- Any node can mint one without coordinating with another node.
- Identifiers sort approximately by creation time, which keeps database indexes
  append-friendly and makes logs and journals readable in order.
- A value of one kind cannot be mistaken for another kind, either in code or in
  a log line, a CLI argument or a persisted record.
- Each identifier has exactly one textual spelling, because identifiers are
  stored and compared as text (database `TEXT` keys, journal fields, command
  lines) as well as handled in binary form.
- Identifiers are labels, not secrets: nothing grants authority because a caller
  knows an identifier.

## Decision

Identifiers are UUID version 7 values as defined by RFC 9562, section 5.7:
a 48-bit Unix timestamp in milliseconds, the version and variant bits, and 74
random bits.

**Kind is part of the type.** `Id<K>` carries a zero-sized kind marker `K`. The
marker trait is sealed, so the set of kinds and their prefixes is closed and
cannot collide. Each kind has an alias: `Aid` (`ap`), `CapabilityId` (`cap`),
`CapsuleId` (`cps`), `NodeId` (`nod`), `CheckpointId` (`ckp`), `FabricId`
(`fab`), `EventId` (`evt`) and `PageId` (`pg`). Passing one kind where another
is expected is a compile error.

**Textual form** is `<prefix>_<uuid>`, with the UUID in canonical lowercase
hyphenated form (36 characters), for example
`ap_017f22e2-79b0-7cc3-98c4-dc0c0c07398f`.

**Parsing is strict.** Only the form produced by `Display` is accepted.
Uppercase digits, missing hyphens, other UUID encodings (braced, URN), surrounding
whitespace, a wrong prefix, and UUIDs that are not version 7 with the RFC variant
are rejected with a typed error. RFC 9562 asks readers to accept uppercase
input; this decision is deliberately stricter so that one identifier cannot have
two spellings as a text key.

**Construction is explicit.** `Id::from_parts(unix_ms, random)` is a pure
function. `Id::try_generate()` reads the system clock and operating-system
entropy and returns an error instead of panicking when either is unavailable.
Keeping the pure constructor public lets callers that need reproducible
identifiers (tests, replayable runs) supply their own clock and entropy.

## Alternatives considered

- **ULID.** Time-ordered and compact, but not a UUID: it cannot be stored in
  UUID-typed columns or passed through UUID tooling, and it is not an IETF
  standard.
- **Snowflake-style 64-bit identifiers.** Small and ordered, but they need
  coordinated worker identifiers and a tightly controlled clock, which conflicts
  with minting identifiers on any node without coordination.
- **Random UUIDv4.** Standard and simple, but unordered, so index locality is
  poor and logs do not sort by time.
- **Unprefixed UUIDs.** Rejected because the kind is invisible in logs and at
  every text boundary, so mix-ups surface late and are hard to diagnose.
- **Counter-based monotonic UUIDv7 (RFC 9562, section 6.2).** Would give strict
  ordering within a process, at the cost of shared mutable state in the
  generator. Not adopted: where strict order matters, records carry an explicit
  sequence number.
- **Shorter text encodings (base32, base58).** Shorter, but they would discard
  the canonical UUID text form that existing tools understand.

## Consequences

- Identifiers order by creation time at millisecond granularity only.
  Identifiers minted within one millisecond are ordered by their random bits.
  Timestamps come from each node's local clock, so ordering across nodes is
  approximate.
- An identifier reveals its creation time to anyone who sees it.
- With 74 random bits per millisecond, the probability that any two of 1,000
  identifiers minted in the same millisecond collide is below 10^-16. Uniqueness
  is probabilistic and is not enforced by a registry.
- For two identifiers of the same kind, binary order, `Ord` order and textual
  order agree.
- Producers outside this codebase must emit lowercase canonical text, or their
  identifiers will not parse.
- The 48-bit timestamp field is exhausted in the year 10889; `from_parts`
  rejects larger values.

## Evidence

The `mosaic-core` test suite covers the following.

- The byte and text layout against a vector built from the RFC 9562 field
  definitions.
- Round-trip of every field, for every kind, over the full timestamp range.
- Rejection of wrong prefixes, non-canonical text, other UUID versions and
  other variants.
- Agreement between binary order and textual order.
- Property tests that arbitrary and near-canonical text is either rejected or
  parses back to the identical string.
- A compile-fail test showing that kinds cannot be interchanged.

## Revisit conditions

- A consumer needs strict ordering of identifiers within a process.
- The textual length becomes a constraint in a storage or interface format.
- RFC 9562 is revised or superseded in a way that affects version 7 layout.
