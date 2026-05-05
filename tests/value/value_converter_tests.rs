/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_value::{
    Value,
    ValueError,
};

#[test]
fn test_value_converter_converts_and_reports_invalid_input() {
    let value = Value::String("42".to_string());
    assert_eq!(value.to::<i32>().unwrap(), 42);

    let invalid = Value::String("not-a-number".to_string());
    assert!(matches!(
        invalid.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));
}

#[test]
fn test_value_converter_uses_default_only_for_empty_values() {
    let empty = Value::Empty(qubit_datatype::DataType::String);
    assert_eq!(empty.to_or::<String>("fallback").unwrap(), "fallback");

    let value = Value::String("actual".to_string());
    assert_eq!(value.to_or::<String>("fallback").unwrap(), "actual");
}
