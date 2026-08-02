// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_datatype::{
    DataConversionError,
    DataListConversionError,
    DataType,
    InvalidValueReason,
};
use qubit_value::{
    ValueAbsence,
    ValueError,
};

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
    assert_eq!(
        ValueError::NoValue(ValueAbsence::UnsetScalar {
            data_type: DataType::String,
        }),
        ValueError::NoValue(ValueAbsence::UnsetScalar {
            data_type: DataType::String,
        }),
    );
    let source = DataConversionError::invalid(
        DataType::String,
        DataType::Int32,
        InvalidValueReason::OutOfRange,
    );
    let single = ValueError::DataConversion(source.clone());
    assert_eq!(
        single.source().and_then(|error| error.downcast_ref()),
        Some(&source),
    );

    let list_source = DataListConversionError::new(2, source);
    let list = ValueError::DataListConversion(list_source.clone());
    assert_eq!(
        list.source().and_then(|error| error.downcast_ref()),
        Some(&list_source),
    );
}

#[test]
fn test_value_absence_preserves_shape_state_and_declared_type() {
    let scalar = ValueAbsence::UnsetScalar {
        data_type: DataType::Int32,
    };
    let collection = ValueAbsence::UnsetCollection {
        data_type: DataType::String,
    };
    let empty = ValueAbsence::EmptyCollection {
        data_type: DataType::UInt64,
    };

    assert_eq!(scalar.data_type(), DataType::Int32);
    assert!(scalar.is_unset());
    assert!(!scalar.is_empty_collection());
    assert_eq!(collection.data_type(), DataType::String);
    assert!(collection.is_unset());
    assert!(!collection.is_empty_collection());
    assert_eq!(empty.data_type(), DataType::UInt64);
    assert!(!empty.is_unset());
    assert!(empty.is_empty_collection());
}

#[test]
fn test_value_error_clone_preserves_structured_source() {
    let source = DataConversionError::invalid(
        DataType::String,
        DataType::Int32,
        InvalidValueReason::OutOfRange,
    );
    let error =
        ValueError::DataListConversion(DataListConversionError::new(3, source));

    assert_eq!(error.clone(), error);
}
