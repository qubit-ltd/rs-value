// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `ValueWireDecodeError`.

use std::error::Error;

use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::json::JsonResource;
use qubit_json::decode::JsonDecoder;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireJsonError;
use qubit_value::ValueWireV1;

#[test]
fn test_safe_wire_json_error_preserves_category_and_position() {
    for input in ["{", "[1,", "not-json"] {
        let source = serde_json::from_str::<serde_json::Value>(input).unwrap_err();
        let expected = (source.classify(), source.line(), source.column());
        let mapped = ValueWireDecodeError::from(source);
        let ValueWireDecodeError::InvalidJson(safe) = mapped else {
            panic!("direct Serde errors must use InvalidJson");
        };
        assert_eq!((safe.classify(), safe.line(), safe.column()), expected);
        match expected.0 {
            serde_json::error::Category::Io => assert!(safe.is_io()),
            serde_json::error::Category::Syntax => assert!(safe.is_syntax()),
            serde_json::error::Category::Data => assert!(safe.is_data()),
            serde_json::error::Category::Eof => assert!(safe.is_eof()),
        }
    }
}

#[test]
fn test_safe_wire_json_data_error_preserves_facts_without_input() {
    let source = serde_json::from_str::<u64>("\"TOP_SECRET\"").unwrap_err();
    let expected = (source.classify(), source.line(), source.column());
    let mapped = ValueWireDecodeError::from(source);
    let ValueWireDecodeError::InvalidJson(safe) = &mapped else {
        panic!("direct Serde errors must use InvalidJson");
    };
    assert_eq!((safe.classify(), safe.line(), safe.column()), expected);
    assert!(safe.is_data());

    assert!(!mapped.to_string().contains("TOP_SECRET"));
    assert!(!format!("{mapped:?}").contains("TOP_SECRET"));
    let mut source = mapped.source();
    while let Some(error) = source {
        assert!(!error.to_string().contains("TOP_SECRET"));
        source = error.source();
    }

    let source = mapped.source().expect("safe metadata remains in the source chain");
    assert!(source.downcast_ref::<ValueWireJsonError>().is_some());
    assert!(source.source().is_none(), "safe metadata has no nested source");
}

#[test]
fn test_wire_json_error_without_upstream_position_uses_zero() {
    let mut decoder = JsonDecoder::unlimited();
    let source = decoder
        .decode_unique_utf8::<u64>(br#""TOP_SECRET""#)
        .expect_err("a JSON string is not a u64 target");
    assert!(source.line().is_none() || source.line() == Some(0));
    assert!(source.column().is_none() || source.column() == Some(0));
    let error = ValueWireDecodeError::from(source);
    let ValueWireDecodeError::InvalidJson(safe) = error else {
        panic!("unique target-deserialization failures must map to InvalidJson");
    };

    assert!(safe.is_data());
    assert_eq!((safe.line(), safe.column()), (0, 0));
}

#[test]
fn test_value_wire_decode_error_preserves_budget_source() {
    let error = ValueWireDecodeError::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::InputBytes,
        observed: Observation::Exact(9),
        maximum: 8,
    });

    assert!(
        error.to_string().contains("InputBytes"),
        "budget errors should identify the limited resource"
    );
    assert!(error.source().is_some(), "budget errors should preserve their source");
}

#[test]
fn test_value_wire_decode_error_preserves_json_source() {
    let source =
        serde_json::from_slice::<u64>(br#""TOP_SECRET""#).expect_err("a JSON string cannot deserialize into u64");
    let error = ValueWireDecodeError::from(source);

    assert!(
        matches!(&error, ValueWireDecodeError::InvalidJson(_)),
        "Serde failures should map to the invalid-JSON variant"
    );
    assert!(
        error
            .to_string()
            .starts_with("failed to decode V1 JSON wire input: JSON deserialization failed"),
        "Serde failures should retain the safe deserialization summary"
    );
    let source = error.source().expect("JSON errors expose safe metadata");
    assert!(
        source.to_string().starts_with("JSON deserialization failed"),
        "the source should expose safe JSON metadata"
    );
    assert!(
        !source.to_string().contains("TOP_SECRET"),
        "JSON diagnostics should not retain input contents"
    );
}

#[test]
fn test_value_wire_decode_error_maps_strict_deserialize_metadata() {
    let error =
        ValueWireV1::decode_json_slice(br#"{"version":1,"value":false}"#).expect_err("a boolean is not a V1 payload");

    assert!(
        matches!(&error, ValueWireDecodeError::InvalidJson(_)),
        "strict deserialization failures should map to invalid JSON"
    );
    let source = error.source().expect("JSON errors expose safe metadata");
    assert!(
        source.to_string().contains("JSON deserialization failed"),
        "unexpected safe JSON source: {source}"
    );
}

#[test]
fn test_value_wire_decode_error_reports_unsupported_version() {
    let error = ValueWireDecodeError::UnsupportedVersion { expected: 1, actual: 2 };

    assert_eq!(error.to_string(), "unsupported qubit-value wire version 2; expected 1");
    assert!(error.source().is_none(), "unsupported versions have no source error");
}

#[test]
fn test_value_wire_decode_error_preserves_quantity_failure() {
    let source = QuantityConversionError::new(QuantityMeasurement::U64(u64::MAX), "usize");
    let error = ValueWireDecodeError::Quantity {
        resource: JsonResource::StringBytes,
        source,
    };

    assert!(
        matches!(
            &error,
            ValueWireDecodeError::Quantity {
                resource: JsonResource::StringBytes,
                source: actual,
            } if *actual == source
        ),
        "quantity errors should preserve the resource and conversion source"
    );
    assert!(
        error.to_string().contains("StringBytes"),
        "quantity errors should identify the measured resource"
    );
    assert_eq!(
        error.source().map(ToString::to_string),
        Some(source.to_string()),
        "quantity errors should expose the original conversion error"
    );
}
