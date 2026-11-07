//! Wire-format tests for the `mosaic.v1` types.
//!
//! The expected byte strings below are written out by hand from the protobuf
//! encoding rules (tag = field number << 3 | wire type), not produced by the
//! library under test. They pin every field number, so renumbering a field in a
//! schema fails these tests as well as the `buf breaking` check.

use std::error::Error;

use mosaic_proto::v1::{Budget, Checkpoint, Envelope, Hash, Hello, Label, SideEffects, envelope};
use proptest::prelude::*;
use prost::Message;

type TestResult = Result<(), Box<dyn Error>>;

fn concat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// Bytes of `Hash { blake3: [fill; 32] }`.
fn hash_bytes(fill: u8) -> Vec<u8> {
    let mut out = vec![0x0a, 0x20];
    out.extend_from_slice(&[fill; 32]);
    out
}

fn hash(fill: u8) -> Hash {
    Hash {
        blake3: vec![fill; 32],
    }
}

fn sample_hello() -> Hello {
    Hello {
        proto_min: 1,
        proto_max: 2,
        features: vec!["cuda".to_owned()],
        node_id: "nod_x".to_owned(),
    }
}

const HELLO_BYTES: &[u8] = &[
    0x08, 0x01, // proto_min = 1
    0x10, 0x02, // proto_max = 2
    0x1a, 0x04, b'c', b'u', b'd', b'a', // features[0] = "cuda"
    0x22, 0x05, b'n', b'o', b'd', b'_', b'x', // node_id = "nod_x"
];

#[test]
fn hello_encodes_with_the_specified_field_numbers() {
    assert_eq!(sample_hello().encode_to_vec(), HELLO_BYTES);
}

#[test]
fn budget_encodes_every_field_with_its_specified_number_and_type() {
    let budget = Budget {
        tokens: 1,
        usd: 2.0,
        gpu_seconds: 3,
        wall_seconds: 4,
        mem_bytes: 5,
        cpu_millis: 6,
    };
    let expected = concat(&[
        &[0x08, 0x01],                      // tokens
        &[0x11, 0, 0, 0, 0, 0, 0, 0, 0x40], // usd = 2.0 (double)
        &[0x18, 0x03],                      // gpu_seconds
        &[0x20, 0x04],                      // wall_seconds
        &[0x28, 0x05],                      // mem_bytes
        &[0x30, 0x06],                      // cpu_millis
    ]);
    assert_eq!(budget.encode_to_vec(), expected);
}

#[test]
fn checkpoint_encodes_every_field_with_its_specified_number() {
    let checkpoint = Checkpoint {
        ckp_id: "c".to_owned(),
        aid: "a".to_owned(),
        journal_seq: 300,
        journal_hash: Some(hash(1)),
        workspace_root: Some(hash(2)),
        page_table_root: Some(hash(3)),
        caps_chain: vec![1, 2, 3],
        ledger: Some(Budget {
            tokens: 5,
            usd: 0.5,
            ..Budget::default()
        }),
        epoch: 9,
    };
    let ledger = concat(&[&[0x08, 0x05], &[0x11, 0, 0, 0, 0, 0, 0, 0xe0, 0x3f]]);
    let expected = concat(&[
        &[0x0a, 0x01, b'c'], // ckp_id
        &[0x12, 0x01, b'a'], // aid
        &[0x18, 0xac, 0x02], // journal_seq = 300
        &[0x22, 0x22],       // journal_hash
        &hash_bytes(1),
        &[0x2a, 0x22], // workspace_root
        &hash_bytes(2),
        &[0x32, 0x22], // page_table_root
        &hash_bytes(3),
        &[0x3a, 0x03, 1, 2, 3], // caps_chain
        &[0x42, 0x0b],          // ledger
        &ledger,
        &[0x48, 0x09], // epoch
    ]);
    assert_eq!(checkpoint.encode_to_vec(), expected);
}

#[test]
fn envelope_carries_hello_in_the_payload_oneof() -> TestResult {
    let envelope = Envelope {
        deadline_unix_ms: 1000,
        epoch: 7,
        payload: Some(envelope::Payload::Hello(sample_hello())),
        ..Envelope::default()
    };
    let expected = concat(&[
        &[0x10, 0xe8, 0x07], // deadline_unix_ms = 1000
        &[0x20, 0x07],       // epoch = 7
        &[0x52, 0x11],       // hello = field 10, 17 bytes
        HELLO_BYTES,
    ]);
    let bytes = envelope.encode_to_vec();
    assert_eq!(bytes, expected);

    match Envelope::decode(bytes.as_slice())?.payload {
        Some(envelope::Payload::Hello(hello)) => assert_eq!(hello, sample_hello()),
        other => panic!("expected a Hello payload, got {other:?}"),
    }
    Ok(())
}

#[test]
fn envelope_without_a_payload_decodes_with_the_oneof_unset() -> TestResult {
    let decoded = Envelope::decode(&[0x20, 0x07][..])?;
    assert_eq!(decoded.epoch, 7);
    assert!(decoded.payload.is_none());
    Ok(())
}

#[test]
fn enum_values_match_the_specification() {
    assert_eq!(Label::Unspecified as i32, 0);
    assert_eq!(Label::Public as i32, 1);
    assert_eq!(Label::Internal as i32, 2);
    assert_eq!(Label::Confidential as i32, 3);
    assert_eq!(Label::LocalOnly as i32, 4);

    assert_eq!(SideEffects::SeUnspecified as i32, 0);
    assert_eq!(SideEffects::None as i32, 1);
    assert_eq!(SideEffects::Workspace as i32, 2);
    assert_eq!(SideEffects::External as i32, 3);
}

#[test]
fn enum_names_are_the_specified_vocabulary() {
    assert_eq!(Label::Public.as_str_name(), "PUBLIC");
    assert_eq!(Label::LocalOnly.as_str_name(), "LOCAL_ONLY");
    assert_eq!(
        Label::from_str_name("CONFIDENTIAL"),
        Some(Label::Confidential)
    );
    assert_eq!(Label::from_str_name("public"), None);
    assert_eq!(SideEffects::External.as_str_name(), "EXTERNAL");
    assert_eq!(
        SideEffects::from_str_name("WORKSPACE"),
        Some(SideEffects::Workspace)
    );
}

#[test]
fn unknown_enum_numbers_are_not_silently_mapped() {
    assert!(Label::try_from(5).is_err());
    assert!(Label::try_from(-1).is_err());
    assert!(SideEffects::try_from(4).is_err());
    assert_eq!(Label::try_from(4), Ok(Label::LocalOnly));
}

#[test]
fn unknown_fields_are_ignored_when_decoding() -> TestResult {
    // A field number the schema does not define, appended to a valid message.
    let mut bytes = HELLO_BYTES.to_vec();
    bytes.extend_from_slice(&[0x98, 0x06, 0x05]); // field 99, varint 5
    assert_eq!(Hello::decode(bytes.as_slice())?, sample_hello());
    Ok(())
}

#[test]
fn encoding_is_not_canonical_so_stored_bytes_must_be_hashed() -> TestResult {
    // The same message with its fields in a different, equally valid order.
    let reordered = concat(&[
        &[0x22, 0x05, b'n', b'o', b'd', b'_', b'x'],
        &[0x1a, 0x04, b'c', b'u', b'd', b'a'],
        &[0x10, 0x02],
        &[0x08, 0x01],
    ]);
    let decoded = Hello::decode(reordered.as_slice())?;
    assert_eq!(decoded, sample_hello());
    assert_ne!(decoded.encode_to_vec(), reordered);

    // A repeated scalar field: the last occurrence wins.
    let duplicated = concat(&[&[0x08, 0x01], &[0x08, 0x09]]);
    assert_eq!(Hello::decode(duplicated.as_slice())?.proto_min, 9);
    Ok(())
}

#[test]
fn truncated_and_malformed_input_is_rejected() {
    let full = sample_hello().encode_to_vec();
    for cut in 1..full.len() {
        let prefix = &full[..cut];
        // A prefix either ends on a field boundary (and is a valid message) or
        // is rejected; it must never panic.
        let _ = Hello::decode(prefix);
    }
    // A length-delimited field that claims more bytes than remain.
    assert!(Hello::decode(&[0x22, 0x05, b'n'][..]).is_err());
    // A tag with field number zero.
    assert!(Hello::decode(&[0x00, 0x01][..]).is_err());
}

fn arb_hash() -> impl Strategy<Value = Option<Hash>> {
    prop::option::of(prop::collection::vec(any::<u8>(), 32).prop_map(|blake3| Hash { blake3 }))
}

fn arb_budget() -> impl Strategy<Value = Option<Budget>> {
    prop::option::of(
        (
            any::<u64>(),
            -1.0e9f64..1.0e9,
            any::<u64>(),
            any::<u64>(),
            any::<u64>(),
            any::<u32>(),
        )
            .prop_map(
                |(tokens, usd, gpu_seconds, wall_seconds, mem_bytes, cpu_millis)| Budget {
                    tokens,
                    usd,
                    gpu_seconds,
                    wall_seconds,
                    mem_bytes,
                    cpu_millis,
                },
            ),
    )
}

fn arb_checkpoint() -> impl Strategy<Value = Checkpoint> {
    (
        ".{0,24}",
        ".{0,24}",
        any::<u64>(),
        arb_hash(),
        arb_hash(),
        arb_hash(),
        prop::collection::vec(any::<u8>(), 0..64),
        arb_budget(),
        any::<u64>(),
    )
        .prop_map(
            |(
                ckp_id,
                aid,
                journal_seq,
                journal_hash,
                workspace_root,
                page_table_root,
                caps_chain,
                ledger,
                epoch,
            )| {
                Checkpoint {
                    ckp_id,
                    aid,
                    journal_seq,
                    journal_hash,
                    workspace_root,
                    page_table_root,
                    caps_chain,
                    ledger,
                    epoch,
                }
            },
        )
}

fn arb_envelope() -> impl Strategy<Value = Envelope> {
    (
        ".{0,24}",
        any::<i64>(),
        ".{0,24}",
        any::<u64>(),
        prop::option::of((
            any::<u32>(),
            any::<u32>(),
            prop::collection::vec(".{0,12}", 0..4),
            ".{0,24}",
        )),
    )
        .prop_map(
            |(trace_parent, deadline_unix_ms, idem_key, epoch, hello)| Envelope {
                trace_parent,
                deadline_unix_ms,
                idem_key,
                epoch,
                payload: hello.map(|(proto_min, proto_max, features, node_id)| {
                    envelope::Payload::Hello(Hello {
                        proto_min,
                        proto_max,
                        features,
                        node_id,
                    })
                }),
            },
        )
}

proptest! {
    #[test]
    fn checkpoint_survives_an_encode_decode_round_trip(checkpoint in arb_checkpoint()) {
        let bytes = checkpoint.encode_to_vec();
        prop_assert_eq!(bytes.len(), checkpoint.encoded_len());
        prop_assert_eq!(Checkpoint::decode(bytes.as_slice())?, checkpoint);
    }

    #[test]
    fn envelope_survives_an_encode_decode_round_trip(envelope in arb_envelope()) {
        let bytes = envelope.encode_to_vec();
        prop_assert_eq!(bytes.len(), envelope.encoded_len());
        prop_assert_eq!(Envelope::decode(bytes.as_slice())?, envelope);
    }

    #[test]
    fn decoding_arbitrary_bytes_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = Envelope::decode(bytes.as_slice());
        let _ = Checkpoint::decode(bytes.as_slice());
        let _ = Hello::decode(bytes.as_slice());
    }
}
