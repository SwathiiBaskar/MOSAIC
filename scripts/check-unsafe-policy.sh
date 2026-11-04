#!/bin/sh
# Checks the repository against docs/unsafe-policy.md.
#
#   check-unsafe-policy.sh [ROOT]      check the workspace rooted at ROOT (default: .)
#   check-unsafe-policy.sh --self-test run the checker against built-in fixtures
#
# Exit status: 0 policy holds, 1 policy violated, 2 usage or environment error.
#
# The compiler is the primary enforcement (workspace lint `unsafe_code =
# "forbid"`). This script verifies that every crate is actually subject to that
# lint, that only approved `sys-*` crates opt out, and that the opt-out is
# narrow. It reads manifests and sources with POSIX tools only.

set -eu

FAILED=0

violation() {
    printf 'policy violation: %s\n' "$*" >&2
    FAILED=1
}

# crate_name MANIFEST: prints the [package] name.
crate_name() {
    awk -F'"' '
        /^\[package\]/ { in_pkg = 1; next }
        /^\[/          { in_pkg = 0 }
        in_pkg && /^[[:space:]]*name[[:space:]]*=/ { print $2; exit }
    ' "$1"
}

# section_has MANIFEST SECTION REGEX: succeeds if a line inside [SECTION] matches.
section_has() {
    awk -v want="[$2]" -v re="$3" '
        /^\[/ { sec = $0; sub(/[[:space:]]*#.*$/, "", sec) }
        sec == want && $0 ~ re { found = 1 }
        END { exit found ? 0 : 1 }
    ' "$1"
}

# rust_sources DIR: prints the Rust source files of a crate.
rust_sources() {
    find "$1/src" "$1/tests" "$1/benches" "$1/examples" "$1/build.rs" \
        -name '*.rs' -type f 2>/dev/null | sort
}

# approved_sys_crates ROOT: prints crate names listed under "## Approved crates".
approved_sys_crates() {
    policy="$1/docs/unsafe-policy.md"
    [ -f "$policy" ] || return 0
    awk '
        /^## / { in_sec = ($0 == "## Approved crates"); next }
        in_sec && /^- `sys-[a-z0-9-]+`/ {
            name = $2; gsub(/`|:/, "", name); print name
        }
    ' "$policy"
}

check_standard_crate() {
    manifest="$1"; dir="$2"; name="$3"
    if ! section_has "$manifest" lints '^[[:space:]]*workspace[[:space:]]*=[[:space:]]*true'; then
        violation "$name: Cargo.toml must contain [lints] workspace = true"
    fi
    for file in $(rust_sources "$dir"); do
        # Lines that are comments do not count; any other use of the word does.
        hits=$(grep -nw 'unsafe' "$file" | grep -vE '^[0-9]+:[[:space:]]*//' || true)
        if [ -n "$hits" ]; then
            violation "$name: unsafe code outside an approved sys-* crate in $file"
            printf '%s\n' "$hits" >&2
        fi
    done
}

check_sys_crate() {
    manifest="$1"; dir="$2"; name="$3"; approved="$4"
    if ! printf '%s\n' "$approved" | grep -qx "$name"; then
        violation "$name: not listed under \"Approved crates\" in docs/unsafe-policy.md"
    fi
    if section_has "$manifest" lints '^[[:space:]]*workspace[[:space:]]*=[[:space:]]*true'; then
        violation "$name: a sys-* crate must declare its own lint table, not inherit the workspace table"
    fi
    for lint in unsafe_code unsafe_op_in_unsafe_fn; do
        section_has "$manifest" lints.rust "^[[:space:]]*${lint}[[:space:]]*=[[:space:]]*\"deny\"" ||
            violation "$name: [lints.rust] must set $lint = \"deny\""
    done
    for lint in undocumented_unsafe_blocks multiple_unsafe_ops_per_block missing_safety_doc; do
        section_has "$manifest" lints.clippy "^[[:space:]]*${lint}[[:space:]]*=[[:space:]]*\"deny\"" ||
            violation "$name: [lints.clippy] must set $lint = \"deny\""
    done
    for file in $(rust_sources "$dir"); do
        if grep -nE '^[[:space:]]*#!\[(allow|warn|expect)\([^]]*unsafe' "$file" >/dev/null; then
            violation "$name: crate-level or module-level inner allow of unsafe lints in $file"
        fi
        # An item-level allow must not sit directly on a module.
        if awk '
            /^[[:space:]]*#\[(allow|expect)\([^]]*unsafe_code/ { armed = 1; next }
            armed && /^[[:space:]]*#\[/                         { next }
            armed && /^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]/ { bad = 1 }
            { armed = 0 }
            END { exit bad ? 0 : 1 }
        ' "$file"; then
            violation "$name: #[allow(unsafe_code)] applied to a module in $file; apply it to the narrowest item"
        fi
    done
}

check_workspace() {
    root="$1"
    [ -f "$root/Cargo.toml" ] || { printf 'no Cargo.toml in %s\n' "$root" >&2; return 2; }

    if ! section_has "$root/Cargo.toml" workspace.lints.rust '^[[:space:]]*unsafe_code[[:space:]]*=[[:space:]]*"forbid"'; then
        violation "workspace Cargo.toml must set unsafe_code = \"forbid\" in [workspace.lints.rust]"
    fi

    approved=$(approved_sys_crates "$root")
    standard=0; sys=0
    for manifest in "$root"/crates/*/Cargo.toml; do
        [ -f "$manifest" ] || continue
        dir=$(dirname "$manifest")
        name=$(crate_name "$manifest")
        [ -n "$name" ] || { violation "$manifest: no [package] name"; continue; }
        case "$name" in
            sys-*) sys=$((sys + 1)); check_sys_crate "$manifest" "$dir" "$name" "$approved" ;;
            *)     standard=$((standard + 1)); check_standard_crate "$manifest" "$dir" "$name" ;;
        esac
    done

    if [ "$FAILED" -eq 0 ]; then
        printf 'unsafe-code policy: OK (%d standard crate(s), %d sys-* crate(s))\n' "$standard" "$sys"
        return 0
    fi
    return 1
}

# ---------------------------------------------------------------- self-test

SELF_PASSED=0
SELF_FAILED=0

expect() {
    # expect pass|fail DESCRIPTION ROOT
    want="$1"; desc="$2"; root="$3"
    if sh "$0" "$root" >/dev/null 2>&1; then got=pass; else got=fail; fi
    if [ "$got" = "$want" ]; then
        SELF_PASSED=$((SELF_PASSED + 1))
        printf '  ok    %-4s %s\n' "$want" "$desc"
    else
        SELF_FAILED=$((SELF_FAILED + 1))
        printf '  FAIL  expected %s, got %s: %s\n' "$want" "$got" "$desc"
    fi
}

write_root() {
    # write_root DIR [forbid|none]
    mkdir -p "$1/crates" "$1/docs"
    if [ "${2:-forbid}" = forbid ]; then lint='unsafe_code = "forbid"'; else lint='missing_docs = "warn"'; fi
    printf '[workspace]\nmembers = ["crates/*"]\n\n[workspace.lints.rust]\n%s\n' "$lint" > "$1/Cargo.toml"
    printf '# Unsafe code policy\n\n## Approved crates\n\nNone.\n\n## Current state\n\nNone.\n' > "$1/docs/unsafe-policy.md"
}

write_standard() {
    # write_standard ROOT NAME [inherit|plain] [source-line]
    d="$1/crates/$2"; mkdir -p "$d/src"
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\n' "$2" > "$d/Cargo.toml"
    if [ "${3:-inherit}" = inherit ]; then printf '\n[lints]\nworkspace = true\n' >> "$d/Cargo.toml"; fi
    printf '%s\n' "${4:-pub fn f() {}}" > "$d/src/lib.rs"
}

write_sys() {
    # write_sys ROOT NAME LINTS_FILE_VARIANT SOURCE
    d="$1/crates/$2"; mkdir -p "$d/src"
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\n' "$2" > "$d/Cargo.toml"
    case "$3" in
        inherit) printf '\n[lints]\nworkspace = true\n' >> "$d/Cargo.toml" ;;
        full|missing)
            printf '\n[lints.rust]\nunsafe_code = "deny"\nunsafe_op_in_unsafe_fn = "deny"\n\n[lints.clippy]\nundocumented_unsafe_blocks = "deny"\nmissing_safety_doc = "deny"\n' >> "$d/Cargo.toml"
            if [ "$3" = full ]; then printf 'multiple_unsafe_ops_per_block = "deny"\n' >> "$d/Cargo.toml"; fi ;;
    esac
    printf '%s\n' "$4" > "$d/src/lib.rs"
}

# The backticks in the printf format are literal Markdown, not command substitution.
# shellcheck disable=SC2016
approve() { printf '# Unsafe code policy\n\n## Approved crates\n\n- `%s`: wraps a kernel interface\n\n## Current state\n\nNone.\n' "$2" > "$1/docs/unsafe-policy.md"; }

self_test() {
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT INT TERM
    echo "checker fixtures:"

    r="$tmp/good-standard"; write_root "$r"; write_standard "$r" mosaic-core inherit
    expect pass "standard crate inheriting the forbid lint" "$r"

    r="$tmp/comment-only"; write_root "$r"
    write_standard "$r" mosaic-core inherit '// the word unsafe in a comment is fine'
    expect pass "the word unsafe in a comment" "$r"

    r="$tmp/no-forbid"; write_root "$r" none; write_standard "$r" mosaic-core inherit
    expect fail "workspace does not forbid unsafe_code" "$r"

    r="$tmp/no-inherit"; write_root "$r"; write_standard "$r" mosaic-core plain
    expect fail "crate does not inherit workspace lints" "$r"

    r="$tmp/unsafe-in-standard"; write_root "$r"
    write_standard "$r" mosaic-core inherit 'pub fn f() { unsafe { core::hint::unreachable_unchecked() } }'
    expect fail "unsafe block in a standard crate" "$r"

    r="$tmp/good-sys"; write_root "$r"; approve "$r" sys-sandbox
    write_sys "$r" sys-sandbox full '// SAFETY: documented at each block'
    expect pass "approved sys crate with the full lint table" "$r"

    r="$tmp/sys-unlisted"; write_root "$r"; write_sys "$r" sys-sandbox full 'pub fn f() {}'
    expect fail "sys crate missing from the approved list" "$r"

    r="$tmp/sys-inherits"; write_root "$r"; approve "$r" sys-sandbox; write_sys "$r" sys-sandbox inherit 'pub fn f() {}'
    expect fail "sys crate inheriting the workspace lint table" "$r"

    r="$tmp/sys-missing-lint"; write_root "$r"; approve "$r" sys-sandbox; write_sys "$r" sys-sandbox missing 'pub fn f() {}'
    expect fail "sys crate without multiple_unsafe_ops_per_block" "$r"

    r="$tmp/sys-crate-allow"; write_root "$r"; approve "$r" sys-sandbox
    write_sys "$r" sys-sandbox full '#![allow(unsafe_code)]'
    expect fail "crate-level allow(unsafe_code)" "$r"

    r="$tmp/sys-module-allow"; write_root "$r"; approve "$r" sys-sandbox
    write_sys "$r" sys-sandbox full '#[allow(unsafe_code)]
mod raw;'
    expect fail "allow(unsafe_code) on a module" "$r"

    r="$tmp/sys-item-allow"; write_root "$r"; approve "$r" sys-sandbox
    write_sys "$r" sys-sandbox full '#[allow(unsafe_code)]
pub fn raw() {}'
    expect pass "allow(unsafe_code) on a single function" "$r"

    if command -v cargo >/dev/null 2>&1; then
        echo "compiler enforcement:"
        c="$tmp/compiler"; mkdir -p "$c/crates/probe/src"
        printf '[workspace]\nresolver = "3"\nmembers = ["crates/probe"]\n\n[workspace.lints.rust]\nunsafe_code = "forbid"\n' > "$c/Cargo.toml"
        printf '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "2024"\n\n[lints]\nworkspace = true\n' > "$c/crates/probe/Cargo.toml"
        printf 'pub fn f() {}\n' > "$c/crates/probe/src/lib.rs"
        if (cd "$c" && cargo check --offline --quiet >/dev/null 2>&1); then
            SELF_PASSED=$((SELF_PASSED + 1)); echo "  ok    pass compiles without unsafe"
        else
            SELF_FAILED=$((SELF_FAILED + 1)); echo "  FAIL  probe crate without unsafe did not compile"
        fi
        printf '#[allow(unsafe_code)]\npub fn f() { unsafe { core::hint::unreachable_unchecked() } }\n' > "$c/crates/probe/src/lib.rs"
        if out=$(cd "$c" && cargo check --offline 2>&1); then
            SELF_FAILED=$((SELF_FAILED + 1)); echo "  FAIL  unsafe with a local allow compiled; forbid was overridden"
        elif printf '%s' "$out" | grep -q 'forbid'; then
            SELF_PASSED=$((SELF_PASSED + 1)); echo "  ok    fail  local allow cannot override the workspace forbid (compiler error)"
        else
            SELF_FAILED=$((SELF_FAILED + 1)); echo "  FAIL  compile failed, but not because of the forbid lint"
        fi
    else
        echo "compiler enforcement: skipped (cargo not found)"
    fi

    printf '%d case(s) passed, %d failed\n' "$SELF_PASSED" "$SELF_FAILED"
    [ "$SELF_FAILED" -eq 0 ]
}

# --------------------------------------------------------------------- main

case "${1:-}" in
    --self-test) self_test ;;
    -h|--help)   sed -n '2,8p' "$0" ;;
    *)           check_workspace "${1:-.}" ;;
esac
