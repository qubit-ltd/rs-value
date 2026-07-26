// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Behavioral tests for the shared finite-value predicate.

use qubit_value::{
    MultiValues,
    Value,
};

/// Verifies finite scalar and collection values survive wire round trips.
#[test]
fn test_finite_float_adapters_round_trip_finite_values() {
    for value in [Value::Float32(1.25), Value::Float64(-2.5)] {
        let json =
            serde_json::to_value(&value).expect("finite scalar must serialize");
        let actual: Value = serde_json::from_value(json)
            .expect("finite scalar must deserialize");
        assert_eq!(actual, value);
    }

    for values in [
        MultiValues::Float32(vec![1.25, -2.5]),
        MultiValues::Float64(vec![3.5, -4.75]),
    ] {
        let json = serde_json::to_value(&values)
            .expect("finite collection must serialize");
        let actual: MultiValues = serde_json::from_value(json)
            .expect("finite collection must deserialize");
        assert_eq!(actual, values);
    }
}
