//! Protobuf message types shared across MOSAIC components.
//!
//! The types are generated at build time from the schemas in
//! `proto/mosaic/v1`. Every message is a plain data type; the crate contains no
//! behaviour beyond what `prost` derives.
//!
//! Protobuf encoding is not canonical: one message has many valid encodings, and
//! re-encoding a decoded message may produce different bytes. Code that hashes or
//! signs a message must therefore hash the bytes it stored or received, never a
//! re-encoded copy.

/// Version 1 of the MOSAIC protobuf package, `mosaic.v1`.
pub mod v1 {
    // The generated code carries no documentation for most items.
    #![allow(missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/mosaic.v1.rs"));
}
