// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for finite floating-point Serde adapters.

use qubit_value::{
    MultiValues,
    Value,
};

/// Verifies scalar and collection wire serialization rejects non-finite values.
#[test]
fn test_finite_float_adapters_reject_non_finite_values() {
    for value in [Value::Float32(f32::NAN), Value::Float64(f64::INFINITY)] {
        assert!(serde_json::to_value(value).is_err());
    }

    assert!(
        serde_json::to_value(MultiValues::Float32(vec![f32::NEG_INFINITY]))
            .is_err()
    );
    assert!(
        serde_json::to_value(MultiValues::Float64(vec![f64::NAN])).is_err()
    );
}
