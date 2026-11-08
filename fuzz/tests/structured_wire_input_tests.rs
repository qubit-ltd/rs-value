// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Checks deterministic structured fuzz inputs, resource bounds, and Wire V1
//! round trips.

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use num_bigint::BigInt;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueRef;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireV1;
use qubit_value_fuzz::structured_wire_input::check_non_finite;
use qubit_value_fuzz::structured_wire_input::container_from_bytes;
use serde_json::Value as JsonValue;
use url::Url;

/// Asserts the generated value fits the wire size bound and survives both
/// decoding steps.
fn round_trip(data: &[u8]) -> ValueContainer {
    let value = container_from_bytes(data);
    let wire = ValueWireV1::try_from(value.clone()).expect("generated value is V1 encodable");
    let encoded = wire.to_json_vec().expect("generated value encodes");
    assert!(encoded.len() <= 4096, "{} bytes", encoded.len());
    let decoded = ValueWireV1::decode_json_slice(&encoded).expect("generated wire decodes");
    assert_eq!(decoded, wire);
    assert_eq!(decoded.into_container(), value);
    value
}

#[test]
fn test_all_52_tags_are_deterministic_encodable_and_round_trip() {
    for tag in 0..52_u8 {
        let input = [tag, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        assert_eq!(container_from_bytes(&input), container_from_bytes(&input), "tag {tag}");
        let _ = round_trip(&input);
    }
}

#[test]
fn test_input_controls_collection_length_and_object_width() {
    for tag in [26_u8, 42, 49, 50] {
        let mut lengths = Vec::new();
        for selector in 0..9_u8 {
            let input = [tag, 0, 0, 0, 0, selector, selector, 0, 1, 2, 3, 4, 5, 6];
            let value = round_trip(&input);
            let len = match value {
                ValueContainer::Collection(values) => values.len(),
                _ => panic!("tag {tag} should be collection"),
            };
            lengths.push(len);
        }
        lengths.sort_unstable();
        lengths.dedup();
        assert!(lengths.len() > 1, "tag {tag} collection length did not vary");
    }
    let widths: Vec<_> = (0..5_u8)
        .map(|width| {
            let mut input = vec![23, 0, 0, 0, 0, 0, width];
            input.extend([1, 2, 3, 4, 5, 6]);
            match round_trip(&input) {
                ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::StringMap(_)) => value
                    .get_ref::<HashMap<String, String>>()
                    .expect("string map payload")
                    .len(),
                value => panic!("unexpected value: {value:?}"),
            }
        })
        .collect();
    assert_eq!(widths, [0, 1, 2, 3, 4]);
}

/// Counts nested JSON containers, treating scalar leaves as depth zero.
#[must_use]
fn json_depth(value: &JsonValue) -> usize {
    match value {
        JsonValue::Array(items) => 1 + items.iter().map(json_depth).max().unwrap_or(0),
        JsonValue::Object(items) => 1 + items.values().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Counts every JSON container and scalar toward the shared generation budget.
#[must_use]
fn json_node_count(value: &JsonValue) -> usize {
    match value {
        JsonValue::Array(items) => 1 + items.iter().map(json_node_count).sum::<usize>(),
        JsonValue::Object(items) => 1 + items.values().map(json_node_count).sum::<usize>(),
        _ => 1,
    }
}

#[test]
fn test_input_controls_json_nesting_depth_within_budget() {
    let shallow = vec![24, 0, 0, 0, 0, 0, 0, 2, 0, 0];
    let mut deep = vec![24, 0, 0, 0, 0, 0, 0, 2, 5];
    for _ in 0..8 {
        deep.extend([8, 5]);
    }
    let depths: Vec<_> = [shallow.as_slice(), deep.as_slice()]
        .into_iter()
        .map(|input| match round_trip(input) {
            ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::Json(_)) => {
                json_depth(value.get_ref::<JsonValue>().expect("JSON payload"))
            }
            value => panic!("unexpected value: {value:?}"),
        })
        .collect();
    assert!(depths[1] > depths[0]);
    assert!(depths[1] <= 8);
}

#[test]
fn test_json_generation_obeys_the_shared_node_budget() {
    let mut input = vec![5; 4096];
    input[0] = 24;
    let value = round_trip(&input);
    let ValueContainer::Scalar(value) = value else {
        panic!("JSON tag must produce a scalar");
    };
    let json = value.get_ref::<JsonValue>().expect("JSON payload");
    assert_eq!(json_node_count(json), 96);

    let mut collection_input = vec![5; 4096];
    collection_input[0] = 50;
    let collection = round_trip(&collection_input);
    let ValueContainer::Collection(values) = collection else {
        panic!("JSON collection tag must produce a collection");
    };
    let json_values = values.get::<JsonValue>().expect("JSON values");
    assert!(json_values.iter().map(json_node_count).sum::<usize>() <= 96);
}

#[test]
fn test_url_component_selectors_produce_valid_distinct_urls() {
    let urls: Vec<_> = (0..8_u8)
        .map(|selector| {
            let input = vec![22, 0, 0, 0, 0, 0, 0, 0, 0, selector, selector, selector, selector];
            let value = round_trip(&input);
            match value {
                ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::Url(_)) => {
                    value.get_ref::<Url>().expect("URL payload").clone()
                }
                value => panic!("unexpected value: {value:?}"),
            }
        })
        .collect();
    assert!(urls.windows(2).any(|pair| pair[0] != pair[1]));
    assert!(urls.iter().all(|url| matches!(url.scheme(), "http" | "https")));
}

#[test]
fn test_empty_truncated_and_oversized_inputs_are_bounded() {
    for input in [&[][..], &[0][..], &[24, 255][..]] {
        let _ = round_trip(input);
    }
    let maximum = vec![0x5a; 4096];
    let oversized = vec![0x5a; 4097];
    assert_eq!(container_from_bytes(&maximum), container_from_bytes(&oversized));
    for tag in 0..52_u8 {
        let mut input = vec![tag; 4096];
        input[0] = tag;
        let _ = round_trip(&input);
    }
}

#[test]
fn test_named_corpus_seeds_reach_expected_shapes_and_round_trip() {
    let seeds: &[(&[u8], fn(&ValueContainer) -> bool)] = &[
        (
            include_bytes!("../corpus/structured_value_wire_v1/empty-collection"),
            |v| matches!(v, ValueContainer::Collection(x) if x.is_empty()),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/multi-item-collection"),
            |v| matches!(v, ValueContainer::Collection(x) if x.len() > 1),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/nested-json"),
            |v| matches!(v, ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::Json(_))),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/negative-scale"),
            |v| matches!(v, ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::BigDecimal(x) if x.as_bigint_and_scale().1 < 0)),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/scale-lower-bound"),
            |v| matches!(v, ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::BigDecimal(x) if x.as_bigint_and_scale().1 == -150_000)),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/scale-upper-bound"),
            |v| matches!(v, ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::BigDecimal(x) if x.as_bigint_and_scale().1 == 150_000)),
        ),
        (
            include_bytes!("../corpus/structured_value_wire_v1/date-boundary"),
            |v| matches!(v, ValueContainer::Scalar(value) if matches!(value.view(), ValueRef::Date(d) if d.year() == 0)),
        ),
    ];
    for (seed, expected) in seeds {
        let value = round_trip(seed);
        assert!(expected(&value), "seed {seed:?} produced {value:?}");
    }
}

#[test]
fn test_scale_selector_covers_legal_boundaries_and_interior_values() {
    let observed: Vec<_> = (0..5_u8)
        .map(|selector| {
            let input = [15, 0, 0, 0, 0, 0, 0, selector, 0];
            match round_trip(&input) {
                ValueContainer::Scalar(value) => match value.view() {
                    ValueRef::BigDecimal(x) => x.as_bigint_and_scale().1,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        })
        .collect();
    assert_eq!(observed, [-150_000, -1, 0, 1, 150_000]);
}

/// Float fixtures use byte 1 as the mode selector. In bit-pattern mode bytes
/// 2..6 (f32) or 2..10 (f64) are little-endian IEEE bits; boundary mode uses
/// byte 2 to select a fixed table. Short payloads are zero-padded.
#[test]
fn test_float_arithmetic_bit_pattern_and_boundary_paths() {
    let arithmetic32 = [12, 0, 0, 0, 0];
    let arithmetic64 = [13, 0, 0, 0, 0];
    assert!(
        matches!(round_trip(&arithmetic32), ValueContainer::Scalar(v) if matches!(v.view(), ValueRef::Float32(x) if x.to_bits() == 0))
    );
    assert!(
        matches!(round_trip(&arithmetic64), ValueContainer::Scalar(v) if matches!(v.view(), ValueRef::Float64(x) if x.to_bits() == (13.0_f64 / 3.0).to_bits()))
    );

    for (tag, bits, expected) in [
        (12_u8, 0x8000_0000_u64, 0x8000_0000_u64),
        (13_u8, 0x8000_0000_0000_0000_u64, 0x8000_0000_0000_0000_u64),
    ] {
        let mut input = vec![tag, 1];
        input.extend_from_slice(&bits.to_le_bytes()[..if tag == 12 { 4 } else { 8 }]);
        let value = round_trip(&input);
        let observed = match value {
            ValueContainer::Scalar(v) => match v.view() {
                ValueRef::Float32(x) => u64::from(x.to_bits()),
                ValueRef::Float64(x) => x.to_bits(),
                _ => panic!("expected float"),
            },
            _ => panic!("expected scalar"),
        };
        assert_eq!(observed, expected);
    }

    for (tag, bits, expected) in [
        (12_u8, 0x7f80_0000_u64, 0x7f7f_ffff_u64),
        (12_u8, 0xff80_0000_u64, 0xff7f_ffff_u64),
        (13_u8, 0x7ff0_0000_0000_0000_u64, 0x7fef_ffff_ffff_ffff_u64),
        (13_u8, 0xfff0_0000_0000_0000_u64, 0xffef_ffff_ffff_ffff_u64),
    ] {
        let mut input = vec![tag, 1];
        input.extend_from_slice(&bits.to_le_bytes()[..if tag == 12 { 4 } else { 8 }]);
        let value = round_trip(&input);
        let observed = match value {
            ValueContainer::Scalar(v) => match v.view() {
                ValueRef::Float32(x) => u64::from(x.to_bits()),
                ValueRef::Float64(x) => x.to_bits(),
                _ => panic!("expected float"),
            },
            _ => panic!("expected scalar"),
        };
        assert_eq!(observed, expected);
    }

    for (tag, width, patterns) in [
        (
            12_u8,
            4_usize,
            vec![
                0_u64,
                0x8000_0000,
                1,
                0x8000_0001,
                0x007f_ffff,
                0x0080_0000,
                0x7f7f_ffff,
                0xff7f_ffff,
                0x3f80_0001,
                0x3f7f_ffff,
            ],
        ),
        (
            13_u8,
            8_usize,
            vec![
                0_u64,
                0x8000_0000_0000_0000,
                1,
                0x8000_0000_0000_0001,
                0x000f_ffff_ffff_ffff,
                0x0010_0000_0000_0000,
                0x7fef_ffff_ffff_ffff,
                0xffef_ffff_ffff_ffff,
                0x3ff0_0000_0000_0001,
                0x3fefffffffffffff,
            ],
        ),
    ] {
        for (selector, expected) in patterns.into_iter().enumerate() {
            let mut input = vec![tag, 2, selector as u8];
            let bytes = expected.to_le_bytes();
            input.extend_from_slice(&bytes[..width]);
            let value = round_trip(&input);
            let observed = match value {
                ValueContainer::Scalar(v) => match v.view() {
                    ValueRef::Float32(x) => u64::from(x.to_bits()),
                    ValueRef::Float64(x) => x.to_bits(),
                    _ => panic!("expected float"),
                },
                _ => panic!("expected scalar"),
            };
            assert_eq!(observed, expected);
        }
    }

    for (tag, expected32, expected64) in [
        (38_u8, 0x8000_0000_u32, 0x8000_0000_0000_0000_u64),
        (39_u8, 0x0000_0001_u32, 0x0000_0000_0000_0001_u64),
    ] {
        let input = [tag, 2, if tag == 38 { 1 } else { 2 }, 0, 0, 1];
        match round_trip(&input) {
            ValueContainer::Collection(values) if tag == 38 => {
                let values = values.get::<f32>().expect("f32 collection");
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].to_bits(), expected32);
            }
            ValueContainer::Collection(values) => {
                let values = values.get::<f64>().expect("f64 collection");
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].to_bits(), expected64);
            }
            value => panic!("expected collection: {value:?}"),
        }
    }
}

#[test]
fn test_non_finite_float_rejection_covers_scalar_and_nonempty_collections() {
    for input in [&[0_u8][..], &[1, 0xff, 0x12, 0x34][..], &[0x80; 16][..]] {
        check_non_finite(input);
    }
}

#[test]
fn test_date_and_utc_instant_generators_cover_leap_and_year_boundaries() {
    for (selector, expected_year) in [(0_u8, 2024), (1, 0), (2, 9999)] {
        for tag in [17_u8, 20] {
            let mut input = vec![tag, 0, 0, 0, 0, 0, 0, 0, selector];
            input.extend([1, 2, 3, 4]);
            let value = round_trip(&input);
            let date = match value {
                ValueContainer::Scalar(value) => match value.view() {
                    ValueRef::Date(date) => *date,
                    ValueRef::Instant(instant) => instant.date_naive(),
                    _ => panic!("unexpected scalar type"),
                },
                value => panic!("unexpected value: {value:?}"),
            };
            assert_eq!(date.year(), expected_year);
            if selector == 0 {
                assert_eq!((date.month(), date.day()), (2, 29));
            }
        }
    }
}

#[test]
fn test_unsupported_numeric_and_instant_boundaries_are_rejected() {
    for scale in [-150_001_i64, 150_001] {
        let value = ValueContainer::Scalar(Value::BigDecimal(BigDecimal::new(BigInt::from(1), scale)));
        assert!(matches!(
            ValueWireV1::try_from(value),
            Err(ValueWireEncodeError::BigDecimalScaleTooLarge {
                scale: rejected,
                ..
            }) if rejected == scale
        ));
    }
    for value in [
        Value::Float32(f32::NAN),
        Value::Float32(f32::INFINITY),
        Value::Float64(f64::NEG_INFINITY),
    ] {
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Scalar(value)),
            Err(ValueWireEncodeError::NonFiniteFloat { .. })
        ));
    }
    for year in [-1, 10_000] {
        let instant = Utc
            .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
            .single()
            .expect("test year is representable");
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Scalar(Value::Instant(instant))),
            Err(ValueWireEncodeError::InstantYearOutOfRange { year: rejected })
                if rejected == year
        ));
    }
}
