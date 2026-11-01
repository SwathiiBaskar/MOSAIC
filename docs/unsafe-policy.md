# Unsafe code policy

## Rule

`unsafe` code is forbidden everywhere in the workspace except in crates whose
names begin with `sys-`. A `sys-*` crate exists only to wrap an operating-system
or hardware interface behind a safe API; no other crate may contain `unsafe`
blocks, `unsafe` functions, `unsafe` trait implementations or `unsafe` attributes.

## Enforcement

The workspace manifest sets `unsafe_code = "forbid"` in `[workspace.lints.rust]`,
and every crate inherits it with `[lints] workspace = true`. Cargo passes the
setting to the compiler as a command-line `forbid`, which a crate cannot
relax: an `#[allow(unsafe_code)]` attribute in source is rejected with error
E0453 ("overruled by previous forbid").

## Requirements for `sys-*` crates

A `sys-*` crate does not inherit the workspace lint table. It declares its own
`[lints]` section that:

- sets `unsafe_code = "deny"` and `unsafe_op_in_unsafe_fn = "deny"`;
- denies the Clippy lints `undocumented_unsafe_blocks`,
  `multiple_unsafe_ops_per_block` and `missing_safety_doc`;
- repeats the remaining workspace lints, so its warnings are as strict as those
  of every other crate.

Within such a crate:

- Unsafe code is permitted only inside items that carry a narrowly scoped
  `#[allow(unsafe_code)]`; a crate-level or module-level allow is not used.
- Every `unsafe` block is preceded by a `// SAFETY:` comment that states the
  invariants the caller of the block upholds and why they hold.
- The public API of the crate is safe. Invariants that cannot be expressed in
  the type system are established inside the crate and covered by tests.
- Code that Miri or a sanitizer can exercise is exercised by them.

## Approved crates

No `sys-*` crate exists. Adding one requires an architectural decision record
naming the interface it wraps and why a safe third-party wrapper is not
sufficient.

## Current state

No first-party crate contains `unsafe` code. Dependencies listed in
[`deps.md`](deps.md) are third-party code and are outside this policy.
