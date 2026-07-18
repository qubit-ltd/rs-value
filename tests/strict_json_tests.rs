// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for strict JSON serialization through public APIs.

use qubit_value::Value;

/// Verifies bounded preallocation does not limit emitted sequence length.
#[test]
fn test_from_serializable_preserves_large_sequences() {
    let source: Vec<_> = (0..2_048).collect();
    let Value::Json(actual) = Value::from_serializable(&source)
        .expect("large sequence must serialize")
    else {
        panic!("serializable input must produce a JSON value");
    };
    let actual = actual
        .as_array()
        .expect("serialized sequence must remain an array");

    assert_eq!(actual.len(), source.len());
    assert_eq!(actual.first().and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(
        actual.last().and_then(serde_json::Value::as_i64),
        Some(2_047),
    );
}
