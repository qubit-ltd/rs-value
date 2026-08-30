// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::ValueError;
use qubit_value::ValueMissing;

#[test]
fn test_multi_values_getter_is_strict() {
    let values = MultiValues::Int32(vec![1, 2]);
    assert_eq!(values.get::<i32>().unwrap(), vec![1, 2]);
    assert!(matches!(
        values.get::<String>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
}

#[test]
fn test_multi_values_first_read_reports_precise_missing_state() {
    assert_eq!(
        MultiValues::Unset(DataType::Int32).get_first::<i32>(),
        Err(ValueError::Missing(ValueMissing::UnsetCollection {
            data_type: DataType::Int32,
        })),
    );
    assert_eq!(
        MultiValues::Int32(Vec::new()).get_first::<i32>(),
        Err(ValueError::Missing(ValueMissing::EmptyCollection {
            data_type: DataType::Int32,
        })),
    );
}

#[cfg(feature = "big-integer")]
#[test]
fn test_multi_values_first_big_integer_getter() {
    let values = MultiValues::BigInteger(vec![num_bigint::BigInt::from(7)]);
    assert_eq!(values.get_first_biginteger().unwrap(), num_bigint::BigInt::from(7));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_multi_values_first_big_decimal_getter() {
    let decimal = "7.5".parse::<bigdecimal::BigDecimal>().unwrap();
    let values = MultiValues::BigDecimal(vec![decimal.clone()]);
    assert_eq!(values.get_first_bigdecimal().unwrap(), decimal);
}

#[cfg(feature = "url")]
#[test]
fn test_multi_values_first_url_getter() {
    let url = url::Url::parse("https://example.com").unwrap();
    let values = MultiValues::Url(vec![url.clone()]);
    assert_eq!(values.get_first_url().unwrap(), url);
}

#[test]
fn test_multi_values_first_string_map_getter() {
    let map = std::collections::HashMap::from([("key".to_string(), "value".to_string())]);
    let values = MultiValues::StringMap(vec![map.clone()]);
    assert_eq!(values.get_first_string_map().unwrap(), map);
}

#[cfg(feature = "json")]
#[test]
fn test_multi_values_first_json_getter() {
    let json = serde_json::json!({"key": 1});
    let values = MultiValues::Json(vec![json.clone()]);
    assert_eq!(values.get_first_json().unwrap(), json);
}

#[test]
fn test_multi_values_try_from_getters_report_type_mismatch() {
    let values = MultiValues::String(vec!["text".to_string()]);
    assert!(matches!(
        i32::try_from(&values),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::String,
        })
    ));
    assert!(matches!(
        Vec::<i32>::try_from(&values),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::String,
        })
    ));
    assert!(matches!(
        i32::try_from(&MultiValues::Unset(DataType::Int32)),
        Err(ValueError::Missing(ValueMissing::UnsetCollection {
            data_type: DataType::Int32,
        }))
    ));
}
