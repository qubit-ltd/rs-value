// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataConversionErrorKind;
use qubit_datatype::DataConverter;
use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;

#[test]
fn test_value_converter_converts_and_reports_invalid_input() {
    let value = Value::String("42".to_string());
    assert_eq!(value.to::<i32>().unwrap(), 42);

    let invalid = Value::String("not-a-number".to_string());
    assert!(matches!(
        invalid.to::<i32>(),
        Err(ValueError::Conversion(error)) if error.kind() == DataConversionErrorKind::InvalidValue
    ));
}

#[test]
fn test_value_converter_uses_default_only_for_empty_values() {
    let empty = Value::Unset(DataType::String);
    assert_eq!(empty.to_or::<String>("fallback").unwrap(), "fallback");

    let value = Value::String("actual".to_string());
    assert_eq!(value.to_or::<String>("fallback").unwrap(), "actual");
}

#[test]
fn test_value_borrows_as_data_converter_without_reimplementing_dispatch() {
    let value = Value::String("42".to_owned());
    let source = DataConverter::from(&value);

    assert_eq!(source.to::<i32>().unwrap(), 42);
}
