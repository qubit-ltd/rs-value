// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `WireLimits`.

use std::collections::HashMap;
use std::time::Duration;

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    Value,
    ValueContainer,
    ValueWireDecodeError,
    ValueWirePayloadV1,
    WireLimits,
};
#[cfg(feature = "big-decimal")]
use {
    bigdecimal::BigDecimal,
    num_bigint::BigInt,
};

#[test]
fn test_value_wire_limits_default_uses_documented_byte_budget() {
    let limits = WireLimits::default();

    assert_eq!(
        limits.max_input_bytes(),
        WireLimits::DEFAULT_MAX_INPUT_BYTES
    );
    assert_eq!(limits.max_input_bytes(), 1_048_576);
}

#[test]
fn test_value_wire_limits_new_preserves_custom_byte_budget() {
    let limits = WireLimits::new(64 * 1024);

    assert_eq!(limits.max_input_bytes(), 65_536);
}

#[test]
fn test_value_wire_limits_check_json_bytes_enforces_public_budget() {
    let limits = WireLimits::new(8);

    limits
        .check_json_bytes(8)
        .expect("input at the byte budget should be accepted");
    assert!(matches!(
        limits.check_json_bytes(9),
        Err(ValueWireDecodeError::InputTooLarge {
            input_bytes: 9,
            max_input_bytes: 8,
        })
    ));
}

#[test]
fn test_wire_budget_checks_scalar_at_embedding_depth() {
    let value = Value::Int32(42);
    let mut budget = WireLimits::new(0)
        .with_max_depth(2)
        .begin(0)
        .expect("the empty input budget should start");

    assert!(matches!(
        budget.check_value_at(&value, 3),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Depth,
            value: 3,
            maximum: 2,
        })
    ));
}

#[test]
fn test_wire_limits_reject_collection_items() {
    let payload = ValueWirePayloadV1::try_from(MultiValues::Int32(vec![1, 2]))
        .expect("the finite collection should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_collection_items(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::CollectionItems,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_budget_counts_multibyte_char_collection_bytes() {
    let values = MultiValues::Char(vec!['é']);
    let mut budget = WireLimits::new(0)
        .with_max_string_bytes(1)
        .begin(0)
        .expect("the empty input budget should start");

    assert!(matches!(
        budget.check_multi_values(&values),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[cfg(feature = "json")]
#[test]
fn test_json_decode_applies_string_limit_to_multibyte_char_collection() {
    let payload = ValueWirePayloadV1::try_from(MultiValues::Char(vec!['é']))
        .expect("the character collection should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_string_bytes(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_limits_reject_string_bytes() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::Scalar(
        Value::String("hello".to_owned()),
    ))
    .expect("the string should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_string_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 5,
            maximum: 4,
        })
    ));
}

#[test]
fn test_wire_budget_accumulates_nodes_across_values() {
    let mut budget = WireLimits::new(0)
        .with_max_nodes(1)
        .begin(0)
        .expect("the empty input budget should start");

    budget
        .check_value(&Value::Int32(1))
        .expect("the first value should fit the node limit");
    assert!(matches!(
        budget.check_value(&Value::Int32(2)),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_budget_counts_string_map_values_as_nodes() {
    let value = Value::StringMap(HashMap::from([(
        "key".to_owned(),
        "value".to_owned(),
    )]));
    let mut budget = WireLimits::new(0)
        .with_max_nodes(1)
        .begin(0)
        .expect("the empty input budget should start");

    assert!(matches!(
        budget.check_value(&value),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_budget_counts_string_map_collection_nodes_globally() {
    let values = MultiValues::StringMap(vec![HashMap::from([(
        "key".to_owned(),
        "value".to_owned(),
    )])]);
    let mut budget = WireLimits::new(0)
        .with_max_nodes(2)
        .begin(0)
        .expect("the empty input budget should start");

    assert!(matches!(
        budget.check_container(&ValueContainer::Collection(values)),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 3,
            maximum: 2,
        })
    ));
}

#[cfg(feature = "json")]
#[test]
fn test_wire_budget_saturates_depth_at_usize_maximum() {
    let value = ValueContainer::Scalar(Value::Json(serde_json::json!([1])));
    let mut budget = WireLimits::new(0)
        .with_max_depth(usize::MAX)
        .begin(0)
        .expect("the empty input budget should start");

    budget
        .check_container_at(&value, usize::MAX)
        .expect("depth accounting must not overflow at usize::MAX");
}

#[test]
fn test_wire_limits_reject_numeric_payload_length() {
    let payload = ValueWirePayloadV1::try_from(Value::Int128(12_345))
        .expect("the integer should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_numeric_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::NumericBytes,
            value: 5,
            maximum: 4,
        })
    ));
}

#[test]
fn test_wire_limits_cover_char_url_chrono_and_duration_payloads() {
    let cases = [
        (
            Value::Char('\u{00e9}'),
            WireLimits::new(0).with_max_string_bytes(1),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Url(
                url::Url::parse("https://example.com/long-path")
                    .expect("the URL fixture should parse"),
            ),
            WireLimits::new(0).with_max_string_bytes(8),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Date(
                chrono::NaiveDate::from_ymd_opt(2026, 8, 6)
                    .expect("the date fixture should be valid"),
            ),
            WireLimits::new(0).with_max_string_bytes(4),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Duration(Duration::from_secs(123)),
            WireLimits::new(0).with_max_numeric_bytes(2),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
    ];

    for (value, limits, expected_kind) in cases {
        let mut budget = limits
            .begin(0)
            .expect("the empty input budget should start");
        assert!(matches!(
            budget.check_value(&value),
            Err(ValueWireDecodeError::LimitExceeded { kind, .. })
                if kind == expected_kind
        ));
    }
}

#[test]
fn test_wire_budget_matrix_covers_every_runtime_data_type() {
    let date = chrono::NaiveDate::from_ymd_opt(2026, 8, 6)
        .expect("the date fixture should be valid");
    let time = chrono::NaiveTime::from_hms_milli_opt(12, 34, 56, 789)
        .expect("the time fixture should be valid");
    let cases = vec![
        (Value::Bool(true), qubit_value::ValueWireLimitKind::Nodes),
        (
            Value::Char('\u{00e9}'),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Int8(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Int16(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Int32(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Int64(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Int128(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::UInt8(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::UInt16(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::UInt32(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::UInt64(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::UInt128(12),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Float32(12.5),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Float64(12.5),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::BigInteger(BigInt::from(12)),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::BigDecimal(BigDecimal::from(12)),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::String("ab".to_owned()),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Date(date),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Time(time),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::DateTime(date.and_time(time)),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Instant(date.and_time(time).and_utc()),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::Duration(Duration::from_secs(12)),
            qubit_value::ValueWireLimitKind::NumericBytes,
        ),
        (
            Value::Url(
                url::Url::parse("https://example.com")
                    .expect("the URL fixture should parse"),
            ),
            qubit_value::ValueWireLimitKind::StringBytes,
        ),
        (
            Value::StringMap(HashMap::from([(
                "key".to_owned(),
                "value".to_owned(),
            )])),
            qubit_value::ValueWireLimitKind::MapEntries,
        ),
        (
            Value::Json(serde_json::json!([1])),
            qubit_value::ValueWireLimitKind::CollectionItems,
        ),
    ];

    let covered_types = cases
        .iter()
        .map(|(value, _)| value.data_type())
        .collect::<Vec<_>>();
    assert_eq!(covered_types.len(), DataType::ALL.len());
    for data_type in DataType::ALL {
        assert_eq!(
            covered_types
                .iter()
                .filter(|covered| *covered == data_type)
                .count(),
            1,
            "the budget matrix must cover {data_type} exactly once",
        );
    }

    for (value, expected_kind) in cases {
        let limits = match expected_kind {
            qubit_value::ValueWireLimitKind::Nodes => {
                WireLimits::new(0).with_max_nodes(0)
            }
            qubit_value::ValueWireLimitKind::CollectionItems => {
                WireLimits::new(0).with_max_collection_items(0)
            }
            qubit_value::ValueWireLimitKind::MapEntries => {
                WireLimits::new(0).with_max_map_entries(0)
            }
            qubit_value::ValueWireLimitKind::StringBytes => {
                WireLimits::new(0).with_max_string_bytes(1)
            }
            qubit_value::ValueWireLimitKind::NumericBytes => {
                WireLimits::new(0).with_max_numeric_bytes(1)
            }
            _ => panic!("unsupported matrix limit kind"),
        };
        let mut budget = limits
            .begin(0)
            .expect("the empty input budget should start");
        assert!(matches!(
            budget.check_value(&value),
            Err(ValueWireDecodeError::LimitExceeded { kind, .. })
                if kind == expected_kind
        ));
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_decode_applies_numeric_limit_to_runtime_value() {
    let input = br#"{"scalar":{"json":1.234567}}"#;
    let limits = WireLimits::new(input.len()).with_max_numeric_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::NumericBytes,
            value: 8,
            maximum: 4,
        })
    ));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_wire_budget_counts_big_decimal_coefficient_without_expanding_scale() {
    let value = Value::BigDecimal(BigDecimal::new(BigInt::from(1), -150_000));
    let mut budget = WireLimits::new(0)
        .with_max_numeric_bytes(1)
        .begin(0)
        .expect("the empty input budget should start");

    budget
        .check_value(&value)
        .expect("the one-digit coefficient should fit without scale expansion");
}

#[test]
fn test_json_decode_applies_string_limit_to_runtime_value() {
    let payload = ValueWirePayloadV1::try_from(Value::String("x".repeat(65)))
        .expect("the string should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_string_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 65,
            maximum: 4,
        })
    ));
}

#[test]
fn test_json_decode_applies_map_limit_to_runtime_value() {
    let value = Value::Json(serde_json::Value::Object(
        (0..17)
            .map(|index| (format!("key{index}"), serde_json::json!(index)))
            .collect(),
    ));
    let payload = ValueWirePayloadV1::try_from(value)
        .expect("the JSON object should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_map_entries(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::MapEntries,
            value: 17,
            maximum: 1,
        })
    ));
}

#[test]
fn test_json_decode_applies_node_limit_to_runtime_value() {
    let payload =
        ValueWirePayloadV1::try_from(MultiValues::Int32((0..70).collect()))
            .expect("the collection should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len())
        .with_max_nodes(1)
        .with_max_collection_items(70);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_json_decode_applies_depth_limit_to_runtime_value() {
    let mut json = serde_json::json!(0);
    for _ in 0..18 {
        json = serde_json::json!([json]);
    }
    let payload = ValueWirePayloadV1::try_from(Value::Json(json))
        .expect("the nested JSON should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_depth(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Depth,
            value: 2,
            maximum: 1,
        })
    ));
}
