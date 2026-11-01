//! Behavioural tests for the typed identifiers in `mosaic_core::ids`.

use std::collections::HashSet;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use mosaic_core::ids::{
    Aid, CapabilityId, CapsuleId, CheckpointId, EventId, FabricId, IdError, IdKind, MAX_UNIX_MS,
    NodeId, PageId,
};
use proptest::prelude::*;

type TestResult = Result<(), Box<dyn Error>>;

/// Field values chosen to produce a known UUID: timestamp `0x017f22e279b0`,
/// `rand_a = 0xcc3`, `rand_b = 0x18c4dc0c0c07398f`.
const VECTOR_MS: u64 = 0x017f_22e2_79b0;
const VECTOR_RANDOM: [u8; 10] = [0x0c, 0xc3, 0x18, 0xc4, 0xdc, 0x0c, 0x0c, 0x07, 0x39, 0x8f];
const VECTOR_UUID: &str = "017f22e2-79b0-7cc3-98c4-dc0c0c07398f";

#[test]
fn from_parts_lays_out_rfc_9562_version_7_fields() -> TestResult {
    let id = Aid::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(id.to_string(), format!("ap_{VECTOR_UUID}"));
    assert_eq!(id.unix_ms(), VECTOR_MS);
    assert_eq!(id.as_bytes()[6] >> 4, 7, "version nibble");
    assert_eq!(id.as_bytes()[8] >> 6, 0b10, "variant bits");
    Ok(())
}

#[test]
fn each_kind_uses_its_own_prefix() -> TestResult {
    fn text<K: IdKind>() -> Result<String, IdError> {
        Ok(mosaic_core::ids::Id::<K>::from_parts(VECTOR_MS, VECTOR_RANDOM)?.to_string())
    }
    assert_eq!(
        text::<mosaic_core::ids::ApKind>()?,
        format!("ap_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::CapabilityKind>()?,
        format!("cap_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::CapsuleKind>()?,
        format!("cps_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::NodeKind>()?,
        format!("nod_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::CheckpointKind>()?,
        format!("ckp_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::FabricKind>()?,
        format!("fab_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::EventKind>()?,
        format!("evt_{VECTOR_UUID}")
    );
    assert_eq!(
        text::<mosaic_core::ids::PageKind>()?,
        format!("pg_{VECTOR_UUID}")
    );
    Ok(())
}

#[test]
fn every_kind_round_trips_through_text() -> TestResult {
    let aid = Aid::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(aid.to_string().parse::<Aid>()?, aid);
    let cap = CapabilityId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(cap.to_string().parse::<CapabilityId>()?, cap);
    let cps = CapsuleId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(cps.to_string().parse::<CapsuleId>()?, cps);
    let nod = NodeId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(nod.to_string().parse::<NodeId>()?, nod);
    let ckp = CheckpointId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(ckp.to_string().parse::<CheckpointId>()?, ckp);
    let fab = FabricId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(fab.to_string().parse::<FabricId>()?, fab);
    let evt = EventId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(evt.to_string().parse::<EventId>()?, evt);
    let pg = PageId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(pg.to_string().parse::<PageId>()?, pg);
    Ok(())
}

#[test]
fn parsing_rejects_an_identifier_of_another_kind() -> TestResult {
    let node = NodeId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(
        node.to_string().parse::<Aid>(),
        Err(IdError::WrongPrefix { expected: "ap" })
    );
    // `cap_` ends in `ap_`, which must not be mistaken for the `ap` prefix.
    let cap = CapabilityId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(
        cap.to_string().parse::<Aid>(),
        Err(IdError::WrongPrefix { expected: "ap" })
    );
    Ok(())
}

#[test]
fn parsing_rejects_non_canonical_text() {
    let valid = format!("ap_{VECTOR_UUID}");
    assert!(valid.parse::<Aid>().is_ok());

    let wrong_prefix = [
        VECTOR_UUID.to_string(),
        format!("AP_{VECTOR_UUID}"),
        format!("ap-{VECTOR_UUID}"),
        format!("ap{VECTOR_UUID}"),
        format!(" ap_{VECTOR_UUID}"),
        String::new(),
    ];
    for text in wrong_prefix {
        assert_eq!(
            text.parse::<Aid>(),
            Err(IdError::WrongPrefix { expected: "ap" }),
            "{text:?}"
        );
    }

    let malformed = [
        "ap_".to_string(),
        format!("ap__{VECTOR_UUID}"),
        format!("ap_{}", VECTOR_UUID.to_uppercase()),
        format!("ap_{}", VECTOR_UUID.replace('-', "")),
        format!("ap_{VECTOR_UUID} "),
        format!("ap_{VECTOR_UUID}x"),
        format!("ap_{}", &VECTOR_UUID[..35]),
        format!("ap_{}g", &VECTOR_UUID[..35]),
        format!("ap_{{{VECTOR_UUID}}}"),
        format!("ap_urn:uuid:{VECTOR_UUID}"),
        "ap_017f22e2-79b0-7cc3-98c4-dc0c0c07398\u{e9}".to_string(),
        "ap_017f22e279b0-7cc3-98c4-dc0c0c07398f-".to_string(),
    ];
    for text in malformed {
        assert_eq!(text.parse::<Aid>(), Err(IdError::Malformed), "{text:?}");
    }
}

#[test]
fn parsing_rejects_other_uuid_versions_and_variants() {
    // Version 4 random UUID.
    let v4 = "ap_f47ac10b-58cc-4372-a567-0e02b2c3d479";
    assert_eq!(v4.parse::<Aid>(), Err(IdError::NotUuidV7));
    // Version 7 with the Microsoft/reserved variant bits (`110`).
    let wrong_variant = "ap_017f22e2-79b0-7cc3-d8c4-dc0c0c07398f";
    assert_eq!(wrong_variant.parse::<Aid>(), Err(IdError::NotUuidV7));
    // Nil UUID.
    let nil = "ap_00000000-0000-0000-0000-000000000000";
    assert_eq!(nil.parse::<Aid>(), Err(IdError::NotUuidV7));
}

#[test]
fn from_parts_rejects_timestamps_wider_than_48_bits() -> TestResult {
    assert!(Aid::from_parts(MAX_UNIX_MS, [0; 10]).is_ok());
    assert_eq!(
        Aid::from_parts(MAX_UNIX_MS + 1, [0; 10]),
        Err(IdError::TimestampOutOfRange)
    );
    assert_eq!(
        Aid::from_parts(u64::MAX, [0xff; 10]),
        Err(IdError::TimestampOutOfRange)
    );
    assert_eq!(
        Aid::from_parts(MAX_UNIX_MS, [0xff; 10])?.unix_ms(),
        MAX_UNIX_MS
    );
    assert_eq!(Aid::from_parts(0, [0; 10])?.unix_ms(), 0);
    Ok(())
}

#[test]
fn from_bytes_enforces_version_and_variant() -> TestResult {
    let id = Aid::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(Aid::from_bytes(*id.as_bytes())?, id);

    let mut bytes = *id.as_bytes();
    bytes[6] = 0x40 | (bytes[6] & 0x0f);
    assert_eq!(Aid::from_bytes(bytes), Err(IdError::NotUuidV7));
    Ok(())
}

#[test]
fn debug_output_names_the_kind_and_shows_the_text() -> TestResult {
    let id = NodeId::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    assert_eq!(format!("{id:?}"), format!("NodeId(nod_{VECTOR_UUID})"));
    Ok(())
}

#[test]
fn identifiers_work_as_hash_keys() -> TestResult {
    let a = Aid::from_parts(VECTOR_MS, VECTOR_RANDOM)?;
    let b = Aid::from_parts(VECTOR_MS + 1, VECTOR_RANDOM)?;
    let set: HashSet<Aid> = [a, b, a].into_iter().collect();
    assert_eq!(set.len(), 2);
    Ok(())
}

#[test]
fn generated_identifiers_are_valid_current_and_distinct() -> TestResult {
    let before = now_ms()?;
    let ids = (0..1_000)
        .map(|_| Aid::try_generate())
        .collect::<Result<Vec<_>, _>>()?;
    let after = now_ms()?;

    for id in &ids {
        assert_eq!(Aid::from_bytes(*id.as_bytes())?, *id);
        assert!(
            id.unix_ms() >= before.saturating_sub(1_000) && id.unix_ms() <= after + 1_000,
            "timestamp {} outside [{before}, {after}]",
            id.unix_ms()
        );
    }
    let distinct: HashSet<_> = ids.iter().collect();
    assert_eq!(distinct.len(), ids.len());
    Ok(())
}

fn now_ms() -> Result<u64, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(u64::try_from(millis)?)
}

proptest! {
    #[test]
    fn text_round_trip_preserves_every_field(
        ms in 0..=MAX_UNIX_MS,
        random in any::<[u8; 10]>(),
    ) {
        let id = Aid::from_parts(ms, random)?;
        let text = id.to_string();
        prop_assert_eq!(text.len(), "ap_".len() + 36);
        let parsed: Aid = text.parse()?;
        prop_assert_eq!(parsed, id);
        prop_assert_eq!(parsed.unix_ms(), ms);
        prop_assert_eq!(id.as_bytes()[6] >> 4, 7);
        prop_assert_eq!(id.as_bytes()[8] >> 6, 0b10);
    }

    #[test]
    fn order_follows_timestamp_and_agrees_with_text_order(
        ms_a in 0..=MAX_UNIX_MS,
        ms_b in 0..=MAX_UNIX_MS,
        random_a in any::<[u8; 10]>(),
        random_b in any::<[u8; 10]>(),
    ) {
        let a = EventId::from_parts(ms_a, random_a)?;
        let b = EventId::from_parts(ms_b, random_b)?;
        if ms_a < ms_b {
            prop_assert!(a < b);
        }
        prop_assert_eq!(a.cmp(&b), a.to_string().cmp(&b.to_string()));
    }

    #[test]
    fn parsing_arbitrary_text_never_panics_and_only_accepts_canonical_text(text in ".{0,64}") {
        if let Ok(id) = text.parse::<Aid>() {
            prop_assert_eq!(id.to_string(), text);
        }
    }

    #[test]
    fn parsing_near_canonical_text_only_accepts_canonical_text(
        prefix in prop::sample::select(vec!["ap_", "AP_", "ap", "cap_", "nod_", ""]),
        uuid in "[0-9a-fA-F-]{30,40}",
    ) {
        let text = format!("{prefix}{uuid}");
        if let Ok(id) = text.parse::<Aid>() {
            prop_assert_eq!(id.to_string(), text);
        }
    }
}
