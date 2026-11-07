//! Prints how a few `mosaic.v1` messages look on the wire.
//!
//! Run with `cargo run -p mosaic-proto --example wire_tour`.

use std::error::Error;

use mosaic_proto::v1::{Budget, Checkpoint, Envelope, Hash, Hello, Label, envelope};
use prost::Message;

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. A message and its bytes.
    let hello = Hello {
        proto_min: 1,
        proto_max: 2,
        features: vec!["cuda".to_owned()],
        node_id: "nod_x".to_owned(),
    };
    let hello_bytes = hello.encode_to_vec();
    println!("1. Hello encodes to {} bytes:", hello_bytes.len());
    println!("   {}", hex(&hello_bytes));
    println!("   08 01 = field 1 (proto_min) = 1");
    println!("   10 02 = field 2 (proto_max) = 2");
    println!("   1a 04 .. = field 3 (features), 4 bytes: \"cuda\"");
    println!("   22 05 .. = field 4 (node_id), 5 bytes: \"nod_x\"");

    // 2. The same message inside an Envelope.
    let envelope = Envelope {
        deadline_unix_ms: 1000,
        epoch: 7,
        payload: Some(envelope::Payload::Hello(hello.clone())),
        ..Envelope::default()
    };
    let envelope_bytes = envelope.encode_to_vec();
    println!(
        "\n2. The Envelope carrying it is {} bytes:",
        envelope_bytes.len()
    );
    println!("   {}", hex(&envelope_bytes));
    match Envelope::decode(envelope_bytes.as_slice())?.payload {
        Some(envelope::Payload::Hello(decoded)) => {
            println!("   decoded payload is a Hello from {:?}", decoded.node_id);
        }
        None => println!("   decoded envelope has no payload"),
    }

    // 3. A checkpoint survives a round trip.
    let checkpoint = Checkpoint {
        ckp_id: "ckp_example".to_owned(),
        aid: "ap_example".to_owned(),
        journal_seq: 300,
        journal_hash: Some(Hash {
            blake3: vec![1; 32],
        }),
        ledger: Some(Budget {
            tokens: 5,
            usd: 0.5,
            ..Budget::default()
        }),
        epoch: 9,
        ..Checkpoint::default()
    };
    let checkpoint_bytes = checkpoint.encode_to_vec();
    let decoded = Checkpoint::decode(checkpoint_bytes.as_slice())?;
    println!(
        "\n3. A Checkpoint is {} bytes; decoding gives back the same message: {}",
        checkpoint_bytes.len(),
        decoded == checkpoint
    );

    // 4. Encoding is not canonical.
    let reordered: Vec<u8> = [
        &[0x22, 0x05, b'n', b'o', b'd', b'_', b'x'][..],
        &[0x1a, 0x04, b'c', b'u', b'd', b'a'][..],
        &[0x10, 0x02][..],
        &[0x08, 0x01][..],
    ]
    .concat();
    let from_reordered = Hello::decode(reordered.as_slice())?;
    println!("\n4. The same Hello with its fields in another order:");
    println!("   {}", hex(&reordered));
    println!(
        "   decodes to the same message: {}",
        from_reordered == hello
    );
    println!(
        "   re-encoding gives the same bytes as received: {}",
        from_reordered.encode_to_vec() == reordered
    );
    println!("   (so a hash must cover the bytes that were stored, not a re-encoded copy)");

    // 5. Unknown fields are ignored.
    let mut with_extra = hello_bytes.clone();
    with_extra.extend_from_slice(&[0x98, 0x06, 0x05]);
    println!(
        "\n5. A Hello with an extra, unknown field 99 still decodes: {}",
        Hello::decode(with_extra.as_slice())? == hello
    );

    // 6. Enums keep their specified names and numbers.
    println!(
        "\n6. Label::LocalOnly is named {} and numbered {}",
        Label::LocalOnly.as_str_name(),
        Label::LocalOnly as i32
    );
    match Label::try_from(7) {
        Ok(label) => println!("   number 7 maps to {label:?}"),
        Err(error) => println!("   number 7 is not mapped to any label ({error})"),
    }
    Ok(())
}
