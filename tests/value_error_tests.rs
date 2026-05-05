/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_datatype::DataType;
use qubit_value::ValueError;

#[test]
fn test_value_error_display_includes_context() {
    let mismatch = ValueError::TypeMismatch {
        expected: DataType::String,
        actual: DataType::Int32,
    };
    assert_eq!(
        mismatch.to_string(),
        "Type mismatch: expected string, actual int32"
    );

    let index = ValueError::IndexOutOfBounds { index: 3, len: 2 };
    assert_eq!(index.to_string(), "Index out of bounds: index 3, length 2");
}

#[test]
fn test_value_error_variants_compare_by_payload() {
    assert_eq!(ValueError::NoValue, ValueError::NoValue);
    assert_ne!(
        ValueError::ConversionError("bad bool".to_string()),
        ValueError::ConversionError("bad int".to_string())
    );
}
