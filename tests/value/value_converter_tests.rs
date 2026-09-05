// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionSession;
use qubit_datatype::DataConversionErrorKind;
use qubit_datatype::DataConverter;
use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;

/// Value adapters preserve direct converter outcomes and cumulative accounting.
#[test]
fn test_value_converter_matches_direct_session_contract() {
    let limits = ConversionLimits::builder()
        .operation_limits(
            ConversionOperationLimits::builder()
                .max_items(4)
                .max_output_bytes(4)
                .build(),
        )
        .build();
    for policy in [ConversionPolicy::strict(), ConversionPolicy::env_friendly()] {
        let mut direct = ConversionSession::new(&policy, &limits);
        let mut adapted = ConversionSession::new(&policy, &limits);
        for text in ["42", " 3 ", "3.9", "bad", "1"] {
            let expected = DataConverter::from(text)
                .to_in::<u8>(&mut direct)
                .map_err(ValueError::from);
            let actual = Value::String(text.to_owned()).to_in::<u8>(&mut adapted);
            assert_eq!(actual, expected, "numeric adapter drift for {text:?}");
            assert_eq!(adapted.items_used(), direct.items_used());
            assert_eq!(adapted.input_bytes_used(), direct.input_bytes_used());
            assert_eq!(adapted.output_bytes_used(), direct.output_bytes_used());
        }
        let mut direct = ConversionSession::new(&policy, &limits);
        let mut adapted = ConversionSession::new(&policy, &limits);
        for text in ["é", " abc ", "xy", "z"] {
            let expected = DataConverter::from(text)
                .to_in::<String>(&mut direct)
                .map_err(ValueError::from);
            let actual = Value::String(text.to_owned()).to_in::<String>(&mut adapted);
            assert_eq!(actual, expected, "text adapter drift for {text:?}");
            assert_eq!(adapted.items_used(), direct.items_used());
            assert_eq!(adapted.input_bytes_used(), direct.input_bytes_used());
            assert_eq!(adapted.output_bytes_used(), direct.output_bytes_used());
        }
    }
}

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
fn test_value_converter_uses_default_for_cross_type_unset_values() {
    let value = Value::Unset(DataType::StringMap);

    assert_eq!(value.to_or::<bool>(false), Ok(false));
}

#[test]
fn test_value_borrows_as_data_converter_without_reimplementing_dispatch() {
    let value = Value::String("42".to_owned());
    let source = DataConverter::from(&value);

    assert_eq!(source.to::<i32>().unwrap(), 42);
}
