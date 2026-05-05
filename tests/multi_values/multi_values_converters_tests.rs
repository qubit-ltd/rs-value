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
    MultiValues,
    Value,
    ValueError,
};

#[test]
fn test_multi_values_converters_convert_first_list_and_value() {
    let values = MultiValues::String(vec!["1".to_string(), "2".to_string()]);
    assert_eq!(values.to::<i32>().unwrap(), 1);
    assert_eq!(values.to_list::<i32>().unwrap(), vec![1, 2]);
    assert_eq!(values.to_value(), Value::String("1".to_string()));
}

#[test]
fn test_multi_values_converters_report_list_conversion_index() {
    let values = MultiValues::String(vec!["1".to_string(), "bad".to_string()]);
    let error = values.to_list::<i32>().unwrap_err();
    assert!(matches!(error, ValueError::ConversionError(_)));
    assert!(error.to_string().contains("index 1"));
}
