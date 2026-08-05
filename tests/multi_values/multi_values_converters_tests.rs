// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::{
    DataConversionError,
    DataType,
    InvalidValueReason,
};
use qubit_value::{
    MultiValues,
    Value,
    ValueError,
    ValueMissing,
};

#[test]
fn test_multi_values_converters_convert_first_list_and_value() {
    let values = MultiValues::String(vec!["1".to_string(), "2".to_string()]);
    assert_eq!(values.to_first::<i32>().unwrap(), 1);
    assert_eq!(values.to_list::<i32>().unwrap(), vec![1, 2]);
    assert_eq!(values.first_value(), Value::String("1".to_string()));
}

#[test]
fn test_multi_values_converters_report_list_conversion_index() {
    let values = MultiValues::String(vec!["1".to_string(), "bad".to_string()]);
    let error = values.to_list::<i32>().unwrap_err();
    assert!(matches!(
        error,
        ValueError::ListConversion(ref error)
            if error.source_index() == 1
                && error.conversion_error() == &DataConversionError::invalid(
                    DataType::String,
                    DataType::Int32,
                    InvalidValueReason::InvalidSyntax {
                        expected: "integer",
                    },
                )
    ));
}

#[test]
fn test_multi_values_empty_conversion_preserves_conversion_semantics() {
    let values = MultiValues::String(Vec::new());
    let error = values
        .to_first::<i32>()
        .expect_err("empty collection has no first converted value");

    let ValueError::Missing(ValueMissing::EmptyCollectionConversion { to }) =
        error
    else {
        panic!("expected an empty collection conversion error");
    };
    assert_eq!(to, qubit_datatype::DataType::Int32);
}
