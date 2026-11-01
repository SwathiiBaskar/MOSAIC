//! Typed, time-ordered identifiers.
//!
//! Every MOSAIC object that needs a stable name carries an [`Id`]: a UUID
//! version 7 (RFC 9562, section 5.7) tagged at the type level with the kind of
//! object it names. The textual form is `<prefix>_<uuid>` where the UUID is in
//! canonical lowercase hyphenated form, for example
//! `ap_017f22e2-79b0-7cc3-98c4-dc0c0c07398f`.
//!
//! The kind is part of the type, so passing a node identifier where an agent
//! process identifier is expected does not compile:
//!
//! ```compile_fail
//! use mosaic_core::ids::{Aid, NodeId};
//!
//! fn takes_aid(_: Aid) {}
//!
//! fn mismatched(node: NodeId) {
//!     takes_aid(node);
//! }
//! ```
//!
//! # Ordering
//!
//! The first 48 bits of the UUID are a Unix timestamp in milliseconds, so
//! identifiers order by creation time at millisecond granularity. Identifiers
//! created within the same millisecond are ordered by their random bits, not by
//! creation order. Components that need a strict order of events must carry an
//! explicit sequence number.
//!
//! # Strictness of parsing
//!
//! Identifiers are compared and stored as text in several places, so one value
//! must have exactly one spelling. [`FromStr`] accepts only the canonical form
//! produced by [`fmt::Display`]; uppercase hexadecimal, missing hyphens,
//! surrounding whitespace and other UUID encodings are rejected. This is
//! deliberately stricter than the case-insensitive input rule of RFC 9562.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::str::FromStr;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Largest representable timestamp: the 48-bit millisecond field is full.
pub const MAX_UNIX_MS: u64 = (1 << 48) - 1;

mod sealed {
    pub trait Sealed {}
}

/// Marker trait implemented by the closed set of identifier kinds.
///
/// The trait is sealed: the set of kinds, and therefore the set of textual
/// prefixes, is fixed by this crate so two kinds can never share a prefix.
pub trait IdKind: sealed::Sealed + 'static {
    /// Textual prefix without the trailing underscore.
    const PREFIX: &'static str;
    /// Name of the corresponding alias, used in `Debug` output.
    const NAME: &'static str;
}

macro_rules! id_kinds {
    ($($(#[$doc:meta])* $kind:ident / $alias:ident = $prefix:literal;)+) => {
        $(
            $(#[$doc])*
            #[derive(Debug)]
            pub enum $kind {}

            impl sealed::Sealed for $kind {}

            impl IdKind for $kind {
                const PREFIX: &'static str = $prefix;
                const NAME: &'static str = stringify!($alias);
            }

            #[doc = concat!("Identifier of kind [`", stringify!($kind), "`] (prefix `", $prefix, "_`).")]
            pub type $alias = Id<$kind>;
        )+
    };
}

id_kinds! {
    /// Kind marker for agent process identifiers.
    ApKind / Aid = "ap";
    /// Kind marker for capability identifiers.
    CapabilityKind / CapabilityId = "cap";
    /// Kind marker for capsule identifiers.
    CapsuleKind / CapsuleId = "cps";
    /// Kind marker for node identifiers.
    NodeKind / NodeId = "nod";
    /// Kind marker for checkpoint identifiers.
    CheckpointKind / CheckpointId = "ckp";
    /// Kind marker for fabric identifiers.
    FabricKind / FabricId = "fab";
    /// Kind marker for journal event identifiers.
    EventKind / EventId = "evt";
    /// Kind marker for context page identifiers.
    PageKind / PageId = "pg";
}

/// A UUIDv7 tagged with the kind of object it identifies.
pub struct Id<K: IdKind> {
    bytes: [u8; 16],
    kind: PhantomData<fn() -> K>,
}

impl<K: IdKind> Id<K> {
    /// Builds an identifier from a millisecond Unix timestamp and random bytes.
    ///
    /// Of the ten random bytes, 74 bits are used (12 for `rand_a`, 62 for
    /// `rand_b`); the bits that overlap the version and variant fields are
    /// discarded. The function is pure, which makes identifier generation
    /// reproducible in tests that supply their own clock and entropy.
    ///
    /// # Errors
    ///
    /// Returns [`IdError::TimestampOutOfRange`] if `unix_ms` does not fit in
    /// 48 bits.
    pub fn from_parts(unix_ms: u64, random: [u8; 10]) -> Result<Self, IdError> {
        if unix_ms > MAX_UNIX_MS {
            return Err(IdError::TimestampOutOfRange);
        }
        let mut bytes = [0u8; 16];
        let ts = unix_ms.to_be_bytes();
        bytes[..6].copy_from_slice(&ts[2..]);
        bytes[6] = 0x70 | (random[0] & 0x0f);
        bytes[7] = random[1];
        bytes[8] = 0x80 | (random[2] & 0x3f);
        bytes[9..].copy_from_slice(&random[3..]);
        Ok(Self::from_valid_bytes(bytes))
    }

    /// Creates an identifier from the system clock and operating-system
    /// entropy.
    ///
    /// # Errors
    ///
    /// Fails if the clock reads earlier than the Unix epoch or later than the
    /// 48-bit timestamp range, or if the operating system cannot supply random
    /// bytes.
    pub fn try_generate() -> Result<Self, GenerateError> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| GenerateError::ClockBeforeEpoch)?
            .as_millis();
        let unix_ms = u64::try_from(millis).map_err(|_| GenerateError::TimestampOutOfRange)?;
        let mut random = [0u8; 10];
        getrandom::fill(&mut random).map_err(|e| GenerateError::Entropy(Box::new(e)))?;
        Self::from_parts(unix_ms, random).map_err(|_| GenerateError::TimestampOutOfRange)
    }

    /// Wraps the 16 raw bytes of a UUID, checking version and variant.
    ///
    /// # Errors
    ///
    /// Returns [`IdError::NotUuidV7`] unless the version nibble is 7 and the
    /// variant bits are `10`.
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, IdError> {
        if bytes[6] >> 4 != 7 || bytes[8] >> 6 != 0b10 {
            return Err(IdError::NotUuidV7);
        }
        Ok(Self::from_valid_bytes(bytes))
    }

    /// Returns the 16 raw bytes of the UUID in big-endian field order.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Returns the creation time embedded in the identifier, in milliseconds
    /// since the Unix epoch.
    #[must_use]
    pub fn unix_ms(&self) -> u64 {
        let mut be = [0u8; 8];
        be[2..].copy_from_slice(&self.bytes[..6]);
        u64::from_be_bytes(be)
    }

    fn from_valid_bytes(bytes: [u8; 16]) -> Self {
        Self {
            bytes,
            kind: PhantomData,
        }
    }
}

impl<K: IdKind> Clone for Id<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: IdKind> Copy for Id<K> {}

impl<K: IdKind> PartialEq for Id<K> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<K: IdKind> Eq for Id<K> {}

impl<K: IdKind> Hash for Id<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<K: IdKind> PartialOrd for Id<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: IdKind> Ord for Id<K> {
    /// Orders by the UUID bytes, which places the timestamp first. For two
    /// identifiers of the same kind this agrees with the order of their
    /// textual forms.
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<K: IdKind> fmt::Display for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut buf = [0u8; 36];
        let mut at = 0;
        for (i, byte) in self.bytes.iter().enumerate() {
            if matches!(i, 4 | 6 | 8 | 10) {
                buf[at] = b'-';
                at += 1;
            }
            buf[at] = HEX[usize::from(byte >> 4)];
            buf[at + 1] = HEX[usize::from(byte & 0x0f)];
            at += 2;
        }
        let text = core::str::from_utf8(&buf).map_err(|_| fmt::Error)?;
        write!(f, "{}_{}", K::PREFIX, text)
    }
}

impl<K: IdKind> fmt::Debug for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(K::NAME)
            .field(&format_args!("{self}"))
            .finish()
    }
}

impl<K: IdKind> FromStr for Id<K> {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, IdError> {
        let uuid = s
            .strip_prefix(K::PREFIX)
            .and_then(|rest| rest.strip_prefix('_'))
            .ok_or(IdError::WrongPrefix {
                expected: K::PREFIX,
            })?;
        Self::from_bytes(parse_canonical_uuid(uuid)?)
    }
}

/// Decodes the 36-character lowercase hyphenated UUID form.
fn parse_canonical_uuid(text: &str) -> Result<[u8; 16], IdError> {
    let chars = text.as_bytes();
    if chars.len() != 36 {
        return Err(IdError::Malformed);
    }
    let mut out = [0u8; 16];
    let mut nibbles = 0usize;
    for (i, &c) in chars.iter().enumerate() {
        if matches!(i, 8 | 13 | 18 | 23) {
            if c != b'-' {
                return Err(IdError::Malformed);
            }
            continue;
        }
        let value = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => return Err(IdError::Malformed),
        };
        let slot = &mut out[nibbles / 2];
        *slot = if nibbles.is_multiple_of(2) {
            value << 4
        } else {
            *slot | value
        };
        nibbles += 1;
    }
    Ok(out)
}

/// Reasons an identifier cannot be built or parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdError {
    /// The text does not start with the prefix of the requested kind.
    WrongPrefix {
        /// Prefix the requested kind uses, without the underscore.
        expected: &'static str,
    },
    /// The UUID part is not 36 characters of lowercase hexadecimal digits with
    /// hyphens in the canonical positions.
    Malformed,
    /// The UUID is well formed but is not version 7 with the RFC variant.
    NotUuidV7,
    /// The timestamp does not fit in 48 bits.
    TimestampOutOfRange,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPrefix { expected } => {
                write!(f, "identifier does not start with `{expected}_`")
            }
            Self::Malformed => f.write_str("identifier is not in canonical lowercase UUID form"),
            Self::NotUuidV7 => f.write_str("identifier is not a UUID version 7"),
            Self::TimestampOutOfRange => f.write_str("timestamp does not fit in 48 bits"),
        }
    }
}

impl Error for IdError {}

/// Reasons [`Id::try_generate`] can fail.
#[derive(Debug)]
#[non_exhaustive]
pub enum GenerateError {
    /// The system clock reads earlier than the Unix epoch.
    ClockBeforeEpoch,
    /// The system clock reads later than the 48-bit timestamp range.
    TimestampOutOfRange,
    /// The operating system could not supply random bytes.
    Entropy(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClockBeforeEpoch => f.write_str("system clock is before the Unix epoch"),
            Self::TimestampOutOfRange => {
                f.write_str("system clock is outside the 48-bit timestamp range")
            }
            Self::Entropy(_) => f.write_str("operating system entropy source failed"),
        }
    }
}

impl Error for GenerateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Entropy(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}
