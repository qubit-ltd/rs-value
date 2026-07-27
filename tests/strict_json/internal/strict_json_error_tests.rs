// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON error behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_reports_serialization_error() {
    use qubit_value::Value;

    assert!(Value::from_serializable(&u128::MAX).is_err());
}
