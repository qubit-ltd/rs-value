// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_datatype::{DataConversionError, DataListConversionError, DataType, InvalidValueReason};
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
}

#[test]
fn test_value_error_variants_compare_by_payload() {
    assert_eq!(ValueError::NoValue, ValueError::NoValue);
    let source = DataConversionError::InvalidValue {
        from: DataType::String,
        to: DataType::Int32,
        reason: InvalidValueReason::OutOfRange,
    };
    let single = ValueError::DataConversion(source.clone());
    assert_eq!(
        single.source().and_then(|error| error.downcast_ref()),
        Some(&source),
    );

    let list_source = DataListConversionError {
        source_index: 2,
        source,
    };
    let list = ValueError::DataListConversion(list_source.clone());
    assert_eq!(
        list.source().and_then(|error| error.downcast_ref()),
        Some(&list_source),
    );
}
