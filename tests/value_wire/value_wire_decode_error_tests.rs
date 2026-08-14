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
use qubit_budget::json::JsonResource;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

#[test]
fn test_value_wire_decode_error_preserves_budget_source() {
    let error = ValueWireDecodeError::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::InputBytes,
        observed: Observation::Exact(9),
        maximum: 8,
    });

    assert!(error.to_string().contains("InputBytes"));
    assert!(error.source().is_some());
}

#[test]
fn test_value_wire_decode_error_preserves_json_source() {
    let source = serde_json::from_slice::<u64>(br#""TOP_SECRET""#)
        .expect_err("a JSON string cannot deserialize into u64");
    let error = ValueWireDecodeError::from(source);

    assert!(matches!(&error, ValueWireDecodeError::InvalidJson(_)));
    assert!(error.to_string().starts_with(
        "failed to decode V1 JSON wire input: JSON deserialization failed"
    ));
    let source = error.source().expect("JSON errors expose safe metadata");
    assert!(
        source
            .to_string()
            .starts_with("JSON deserialization failed")
    );
    assert!(!source.to_string().contains("TOP_SECRET"));
}

#[test]
fn test_value_wire_decode_error_maps_strict_deserialize_metadata() {
    let error =
        ValueWireV1::decode_json_slice(br#"{"version":1,"value":false}"#)
            .expect_err("a boolean is not a V1 payload");

    assert!(matches!(&error, ValueWireDecodeError::InvalidJson(_)));
    let source = error.source().expect("JSON errors expose safe metadata");
    assert!(
        source.to_string().contains("JSON deserialization failed"),
        "unexpected safe JSON source: {source}"
    );
}

#[test]
fn test_value_wire_decode_error_reports_unsupported_version() {
    let error = ValueWireDecodeError::UnsupportedVersion {
        expected: 1,
        actual: 2,
    };

    assert_eq!(
        error.to_string(),
        "unsupported qubit-value wire version 2; expected 1"
    );
    assert!(error.source().is_none());
}
