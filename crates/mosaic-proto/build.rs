//! Generates the Rust types for `proto/mosaic/v1`.
//!
//! The schemas are compiled by `protox`, a protobuf compiler written in Rust,
//! so building the crate needs no `protoc` installation.

use std::env;
use std::error::Error;
use std::path::PathBuf;

const PROTO_FILES: &[&str] = &[
    "mosaic/v1/types.proto",
    "mosaic/v1/checkpoint.proto",
    "mosaic/v1/envelope.proto",
];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_root = manifest_dir.join("../../proto");

    let descriptors = protox::compile(PROTO_FILES, [&proto_root])?;
    prost_build::Config::new().compile_fds(descriptors)?;

    println!("cargo:rerun-if-changed={}", proto_root.display());
    Ok(())
}
