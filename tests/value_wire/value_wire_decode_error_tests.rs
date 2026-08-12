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
use qubit_json::JsonResource;
use qubit_value::ValueWireDecodeError;

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
    let source = serde_json::from_slice::<serde_json::Value>(b"not JSON")
        .expect_err("malformed JSON should fail");
    let error = ValueWireDecodeError::from(source);

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
    assert!(error.source().is_some());
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
