// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON serialization behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_rejects_non_finite_float() {
    use qubit_value::Value;

    assert!(Value::from_serializable(&f64::NAN).is_err());
}
