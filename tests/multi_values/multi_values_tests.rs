/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MultiValues Unit Tests
//!
//! Tests various functionalities of the multi values container。
//!
//! # Author
//!
//! Haixing Hu

use bigdecimal::BigDecimal;
use chrono::{
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    TimeZone,
    Utc,
};
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use qubit_value::{
    MultiValues,
    Value,
    ValueError,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

// ============================================================================
// Test macros - used for batch generation of repetitive test cases
// ============================================================================

/// Batch test type mismatch errors for add_xxx methods
macro_rules! test_add_type_mismatch {
    ($($method:ident, $value:expr, $desc:expr);* $(;)?) => {
        $(
            {
                let mut mv = MultiValues::Int32(vec![1, 2, 3]);
                let result = mv.$method($value);
                assert!(
                    matches!(result, Err(ValueError::TypeMismatch { .. })),
                    "Expected TypeMismatch for {}", $desc
                );
            }
        )*
    };
}

/// Batch test type conversion for Empty + add_xxx
#[allow(unused_macros)]
macro_rules! test_empty_add_conversion {
    ($($data_type:expr, $method:ident, $value:expr, $get_method:ident, $expected:expr);* $(;)?) => {
        $(
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$method($value).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), 1);
                assert_eq!(mv.$get_method().unwrap(), $expected);
            }
        )*
    };
}

/// Batch test type conversion for Empty + add_xxxs (Vec)
#[allow(unused_macros)]
macro_rules! test_empty_add_vec_conversion {
    ($($data_type:expr, $method:ident, $values:expr);* $(;)?) => {
        $(
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$method($values).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), $values.len());
            }
        )*
    };
}

/// Batch test type conversion for Empty + add_xxxs_slice
#[allow(unused_macros)]
macro_rules! test_empty_add_slice_conversion {
    ($($data_type:expr, $method:ident, $values:expr);* $(;)?) => {
        $(
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$method($values).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), $values.len());
            }
        )*
    };
}

/// Batch test type mismatch errors for get_xxx methods
macro_rules! test_get_type_mismatch {
    ($($method:ident);* $(;)?) => {
        $(
            {
                let mv = MultiValues::Int32(vec![1, 2, 3]);
                let result = mv.$method();
                assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
            }
        )*
    };
}

/// Batch test type mismatch errors for get_first_xxx methods
macro_rules! test_get_first_type_mismatch {
    ($($method:ident);* $(;)?) => {
        $(
            {
                let mv = MultiValues::Int32(vec![1, 2, 3]);
                let result = mv.$method();
                assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
            }
        )*
    };
}

/// Batch test correct conversion from Empty to various types (all integer types)
macro_rules! test_empty_to_all_int_types {
    (
        $( $data_type:expr, $add_method:ident, $add_vec_method:ident, $add_slice_method:ident,
           $get_first_method:ident, $value:expr, $vec_values:expr );* $(;)?
    ) => {
        $(
            // Test add single value
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$add_method($value).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), 1);
                assert_eq!(mv.$get_first_method().unwrap(), $value);
            }

            // Test add_xxxs (Vec)
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$add_vec_method($vec_values.clone()).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), $vec_values.len());
            }

            // Test add_xxxs_slice
            {
                let mut mv = MultiValues::Empty($data_type);
                mv.$add_slice_method(&$vec_values[..]).unwrap();
                assert_eq!(mv.data_type(), $data_type);
                assert_eq!(mv.count(), $vec_values.len());
            }
        )*
    };
}

/// Batch test Empty type mismatch errors
macro_rules! test_empty_type_mismatch_errors {
    (
        $base_type:expr, $base_empty:expr;
        $($method:ident, $value:expr);* $(;)?
    ) => {
        $(
            {
                let mut mv = MultiValues::Empty($base_empty);
                let result = mv.$method($value);
                assert!(
                    matches!(result, Err(ValueError::TypeMismatch { .. })),
                    "Expected TypeMismatch for {} on Empty({:?})",
                    stringify!($method), $base_empty
                );
            }
        )*
    };
}

/// Batch test get_first returns NoValue from Empty
macro_rules! test_get_first_empty_no_value {
    ($($data_type:expr, $get_first_method:ident);* $(;)?) => {
        $(
            {
                let mv = MultiValues::Empty($data_type);
                let result = mv.$get_first_method();
                assert!(matches!(result, Err(ValueError::NoValue)));
            }
        )*
    };
}

/// Batch test adding elements to existing MultiValues (non-Empty)
macro_rules! test_add_to_existing {
    (
        $( $constructor:expr, $add_method:ident, $value:expr,
           $add_vec_method:ident, $vec_values:expr,
           $add_slice_method:ident, $expected_count:expr );* $(;)?
    ) => {
        $(
            // Test add single value
            {
                let mut mv = $constructor;
                let initial_count = mv.count();
                mv.$add_method($value).unwrap();
                assert_eq!(mv.count(), initial_count + 1);
            }

            // Test add_xxxs (Vec)
            {
                let mut mv = $constructor;
                let initial_count = mv.count();
                mv.$add_vec_method($vec_values.clone()).unwrap();
                assert_eq!(mv.count(), initial_count + $vec_values.len());
            }

            // Test add_xxxs_slice
            {
                let mut mv = $constructor;
                let initial_count = mv.count();
                mv.$add_slice_method(&$vec_values[..]).unwrap();
                assert_eq!(mv.count(), initial_count + $vec_values.len());
            }
        )*
    };
}

/// Batch test set_single method (set single value, replace entire list)
#[allow(unused_macros)]
macro_rules! test_set_single_comprehensive {
    (
        $( $set_method:ident, $get_method:ident, $value:expr );* $(;)?
    ) => {
        $(
            // Test setting single value on Empty
            {
                let mut mv = MultiValues::Empty(DataType::Int32);
                mv.$set_method($value).unwrap();
                assert_eq!(mv.count(), 1);
                assert_eq!(mv.$get_method().unwrap(), &[$value]);
            }

            // Test setting single value on existing values (will replace entire list)
            {
                let mut mv = MultiValues::Int32(vec![1, 2, 3, 4, 5]);
                mv.$set_method($value).unwrap();
                assert_eq!(mv.count(), 1);
                assert_eq!(mv.$get_method().unwrap(), &[$value]);
            }
        )*
    };
}

/// Batch test set method (Vec/slice/single value)
#[allow(unused_macros)]
macro_rules! test_set_comprehensive {
    (
        $( $set_vec_method:ident, $set_slice_method:ident, $get_method:ident,
           $single_value:expr, $vec_values:expr );* $(;)?
    ) => {
        $(
            // Test set_xxxs (Vec)
            {
                let mut mv = MultiValues::Empty(DataType::Int32);
                mv.$set_vec_method($vec_values.clone()).unwrap();
                assert_eq!(mv.count(), $vec_values.len());
                assert_eq!(mv.$get_method().unwrap(), &$vec_values[..]);
            }

            // Test set_xxxs_slice
            {
                let mut mv = MultiValues::Int32(vec![999, 888, 777]);
                mv.$set_slice_method(&$vec_values[..]).unwrap();
                assert_eq!(mv.count(), $vec_values.len());
                assert_eq!(mv.$get_method().unwrap(), &$vec_values[..]);
            }
        )*
    };
}

#[test]
fn test_multi_value_creation() {
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    assert_eq!(mv.data_type(), DataType::Int32);
    assert_eq!(mv.count(), 3);
    assert!(!mv.is_empty());
}

#[test]
fn test_multi_value_empty() {
    let mv = MultiValues::Empty(DataType::String);
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.count(), 0);
    assert!(mv.is_empty());
}

#[test]
fn test_multi_value_clear() {
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int32);
}

#[test]
fn test_multi_value_first() {
    let mv = MultiValues::Int32(vec![42, 100, 200]);
    assert_eq!(mv.get_first_int32().unwrap(), 42);

    let empty = MultiValues::Int32(vec![]);
    assert!(matches!(empty.get_first_int32(), Err(ValueError::NoValue)));
}

#[test]
fn test_multi_value_as_slice() {
    let mv = MultiValues::Int32(vec![1, 2, 3, 4, 5]);
    let slice = mv.get_int32s().unwrap();
    assert_eq!(slice, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_multi_value_add() {
    let mut mv = MultiValues::Int32(vec![1, 2]);
    mv.add_int32(3).unwrap();
    mv.add_int32(4).unwrap();
    assert_eq!(mv.count(), 4);
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2, 3, 4]);
}

#[test]
fn test_multi_value_add_to_empty() {
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add_int32(42).unwrap();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int32().unwrap(), 42);
}

#[test]
fn test_multi_value_type_mismatch() {
    let mut mv = MultiValues::Int32(vec![1, 2]);
    assert!(matches!(
        mv.add_bool(true),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_multi_value_merge() {
    let mut a = MultiValues::Int32(vec![1, 2]);
    let b = MultiValues::Int32(vec![3, 4]);
    a.merge(&b).unwrap();
    assert_eq!(a.count(), 4);
    assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);
}

#[test]
fn test_multi_value_merge_type_mismatch() {
    let mut a = MultiValues::Int32(vec![1, 2]);
    let b = MultiValues::String(vec!["hello".to_string()]);
    assert!(matches!(a.merge(&b), Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_value_merge_type_mismatch_keeps_left_unchanged() {
    let mut a = MultiValues::Int32(vec![1, 2, 3]);
    let original = a.clone();
    let b = MultiValues::String(vec!["x".to_string()]);
    let err = a.merge(&b);
    assert!(matches!(err, Err(ValueError::TypeMismatch { .. })));
    assert_eq!(a, original);
}

#[test]
fn test_multi_value_merge_empty_rhs_is_noop() {
    let mut values = MultiValues::Int32(vec![1, 2]);
    let empty = MultiValues::Empty(DataType::Int32);

    values
        .merge(&empty)
        .expect("merging same-typed empty values should succeed");

    assert_eq!(values.get_int32s().unwrap(), &[1, 2]);
}

#[test]
fn test_multi_value_from_value() {
    let v = Value::Int32(42);
    let mv: MultiValues = v.into();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int32().unwrap(), 42);
}

#[test]
fn test_multi_value_strings() {
    let mv = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_first_string().unwrap(), "hello");
    let all = mv.get_strings().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0], "hello");
    assert_eq!(all[1], "world");
}

#[test]
fn test_multi_value_default() {
    let mv: MultiValues = Default::default();
    assert_eq!(mv.data_type(), DataType::String);
    assert!(mv.is_empty());
}

#[test]
fn test_multi_value_generic_get() {
    // Test type inference
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let nums: Vec<i32> = mv.get().unwrap();
    assert_eq!(nums, vec![1, 2, 3]);

    // Test explicit type parameter
    let mv = MultiValues::Int64(vec![10, 20, 30]);
    let nums = mv.get::<i64>().unwrap();
    assert_eq!(nums, vec![10, 20, 30]);

    // Test strings
    let mv = MultiValues::String(vec!["a".to_string(), "b".to_string()]);
    let strs: Vec<String> = mv.get().unwrap();
    assert_eq!(strs, vec!["a", "b"]);
}

#[test]
fn test_multi_value_generic_get_type_mismatch() {
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result: Result<Vec<bool>, _> = mv.get();
    assert!(result.is_err());
}

#[test]
fn test_multi_value_generic_to_converts_first_value() {
    let mv = MultiValues::String(vec!["1".to_string()]);
    let flag: bool = mv.to().unwrap();
    assert!(flag);

    let mv = MultiValues::String(vec!["FALSE".to_string()]);
    let flag: bool = mv.to().unwrap();
    assert!(!flag);
}

#[test]
fn test_multi_value_generic_to_converts_first_value_for_all_variants_to_string() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
    let time = NaiveTime::from_hms_opt(3, 4, 5).unwrap();
    let datetime = date.and_time(time);
    let instant = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
    let duration = Duration::from_millis(250);
    let url = Url::parse("https://example.com/path").unwrap();

    let cases: Vec<(MultiValues, String)> = vec![
        (MultiValues::Bool(vec![true]), "true".to_string()),
        (MultiValues::Char(vec!['x']), "x".to_string()),
        (MultiValues::Int8(vec![-1]), "-1".to_string()),
        (MultiValues::Int16(vec![-10]), "-10".to_string()),
        (MultiValues::Int32(vec![-100]), "-100".to_string()),
        (MultiValues::Int64(vec![-1000]), "-1000".to_string()),
        (MultiValues::Int128(vec![-10000]), "-10000".to_string()),
        (MultiValues::UInt8(vec![1]), "1".to_string()),
        (MultiValues::UInt16(vec![10]), "10".to_string()),
        (MultiValues::UInt32(vec![100]), "100".to_string()),
        (MultiValues::UInt64(vec![1000]), "1000".to_string()),
        (MultiValues::UInt128(vec![10000]), "10000".to_string()),
        (MultiValues::IntSize(vec![-7]), "-7".to_string()),
        (MultiValues::UIntSize(vec![7]), "7".to_string()),
        (MultiValues::Float32(vec![1.5]), "1.5".to_string()),
        (MultiValues::Float64(vec![3.5]), "3.5".to_string()),
        (
            MultiValues::BigInteger(vec![BigInt::from(123)]),
            "123".to_string(),
        ),
        (
            MultiValues::BigDecimal(vec![BigDecimal::from_str("12.5").unwrap()]),
            "12.5".to_string(),
        ),
        (
            MultiValues::String(vec!["alpha".to_string()]),
            "alpha".to_string(),
        ),
        (MultiValues::Date(vec![date]), date.to_string()),
        (MultiValues::Time(vec![time]), time.to_string()),
        (MultiValues::DateTime(vec![datetime]), datetime.to_string()),
        (MultiValues::Instant(vec![instant]), instant.to_rfc3339()),
        (
            MultiValues::Duration(vec![duration]),
            format!("{}ns", duration.as_nanos()),
        ),
        (MultiValues::Url(vec![url.clone()]), url.to_string()),
    ];

    for (values, expected) in cases {
        let actual: String = values.to().unwrap();
        assert_eq!(actual, expected);
    }

    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let map_text: String = MultiValues::StringMap(vec![map.clone()]).to().unwrap();
    let parsed_map: serde_json::Value = serde_json::from_str(&map_text).unwrap();
    assert_eq!(parsed_map["key"], serde_json::json!("value"));

    let json_value = serde_json::json!({"flag": true});
    let json_text: String = MultiValues::Json(vec![json_value]).to().unwrap();
    assert_eq!(json_text, r#"{"flag":true}"#);
}

#[test]
fn test_multi_value_generic_to_list_converts_all_values() {
    let mv = MultiValues::String(vec![
        "1".to_string(),
        "0".to_string(),
        "true".to_string(),
        "FALSE".to_string(),
    ]);
    let flags: Vec<bool> = mv.to_list().unwrap();
    assert_eq!(flags, vec![true, false, true, false]);
}

#[test]
fn test_multi_value_generic_to_list_converts_all_variants_to_string() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
    let time = NaiveTime::from_hms_opt(3, 4, 5).unwrap();
    let datetime = date.and_time(time);
    let instant = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
    let duration = Duration::from_millis(250);
    let url = Url::parse("https://example.com/path").unwrap();

    let cases: Vec<(MultiValues, Vec<String>)> = vec![
        (
            MultiValues::Bool(vec![true, false]),
            vec!["true".to_string(), "false".to_string()],
        ),
        (
            MultiValues::Char(vec!['x', 'y']),
            vec!["x".to_string(), "y".to_string()],
        ),
        (
            MultiValues::Int8(vec![-1, 2]),
            vec!["-1".to_string(), "2".to_string()],
        ),
        (
            MultiValues::Int16(vec![-10, 20]),
            vec!["-10".to_string(), "20".to_string()],
        ),
        (
            MultiValues::Int32(vec![-100, 200]),
            vec!["-100".to_string(), "200".to_string()],
        ),
        (
            MultiValues::Int64(vec![-1000, 2000]),
            vec!["-1000".to_string(), "2000".to_string()],
        ),
        (
            MultiValues::Int128(vec![-10000, 20000]),
            vec!["-10000".to_string(), "20000".to_string()],
        ),
        (
            MultiValues::UInt8(vec![1, 2]),
            vec!["1".to_string(), "2".to_string()],
        ),
        (
            MultiValues::UInt16(vec![10, 20]),
            vec!["10".to_string(), "20".to_string()],
        ),
        (
            MultiValues::UInt32(vec![100, 200]),
            vec!["100".to_string(), "200".to_string()],
        ),
        (
            MultiValues::UInt64(vec![1000, 2000]),
            vec!["1000".to_string(), "2000".to_string()],
        ),
        (
            MultiValues::UInt128(vec![10000, 20000]),
            vec!["10000".to_string(), "20000".to_string()],
        ),
        (
            MultiValues::IntSize(vec![-7, 8]),
            vec!["-7".to_string(), "8".to_string()],
        ),
        (
            MultiValues::UIntSize(vec![7, 8]),
            vec!["7".to_string(), "8".to_string()],
        ),
        (
            MultiValues::Float32(vec![1.5, 2.5]),
            vec!["1.5".to_string(), "2.5".to_string()],
        ),
        (
            MultiValues::Float64(vec![3.5, 4.5]),
            vec!["3.5".to_string(), "4.5".to_string()],
        ),
        (
            MultiValues::BigInteger(vec![BigInt::from(123), BigInt::from(456)]),
            vec!["123".to_string(), "456".to_string()],
        ),
        (
            MultiValues::BigDecimal(vec![
                BigDecimal::from_str("12.5").unwrap(),
                BigDecimal::from_str("34.5").unwrap(),
            ]),
            vec!["12.5".to_string(), "34.5".to_string()],
        ),
        (
            MultiValues::String(vec!["alpha".to_string(), "beta".to_string()]),
            vec!["alpha".to_string(), "beta".to_string()],
        ),
        (MultiValues::Date(vec![date]), vec![date.to_string()]),
        (MultiValues::Time(vec![time]), vec![time.to_string()]),
        (
            MultiValues::DateTime(vec![datetime]),
            vec![datetime.to_string()],
        ),
        (
            MultiValues::Instant(vec![instant]),
            vec![instant.to_rfc3339()],
        ),
        (
            MultiValues::Duration(vec![duration]),
            vec![format!("{}ns", duration.as_nanos())],
        ),
        (MultiValues::Url(vec![url.clone()]), vec![url.to_string()]),
    ];

    for (values, expected) in cases {
        let actual: Vec<String> = values.to_list().unwrap();
        assert_eq!(actual, expected);
    }

    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let maps: Vec<String> = MultiValues::StringMap(vec![map.clone()]).to_list().unwrap();
    assert_eq!(maps.len(), 1);
    let parsed_map: serde_json::Value = serde_json::from_str(&maps[0]).unwrap();
    assert_eq!(parsed_map["key"], serde_json::json!("value"));

    let json_value = serde_json::json!({"flag": true});
    let jsons: Vec<String> = MultiValues::Json(vec![json_value]).to_list().unwrap();
    assert_eq!(jsons, vec![r#"{"flag":true}"#]);

    let empty: Vec<String> = MultiValues::Empty(DataType::String).to_list().unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_multi_value_generic_to_reports_no_value_and_conversion_errors() {
    let empty = MultiValues::Empty(DataType::String);
    assert!(matches!(empty.to::<bool>(), Err(ValueError::NoValue)));

    let empty = MultiValues::String(Vec::new());
    assert!(matches!(empty.to::<bool>(), Err(ValueError::NoValue)));

    let invalid = MultiValues::String(vec!["yes".to_string()]);
    assert!(matches!(
        invalid.to_list::<bool>(),
        Err(ValueError::ConversionError(_))
    ));

    let unsupported = MultiValues::Date(vec![NaiveDate::from_ymd_opt(2026, 1, 2).unwrap()]);
    assert!(matches!(
        unsupported.to::<bool>(),
        Err(ValueError::ConversionFailed {
            from: DataType::Date,
            to: DataType::Bool
        })
    ));

    let invalid_json = MultiValues::String(vec!["{".to_string()]);
    assert!(matches!(
        invalid_json.to::<JsonValue>(),
        Err(ValueError::JsonDeserializationError(_))
    ));
}

#[test]
fn test_multi_value_generic_to_list_reports_failing_index() {
    let invalid = MultiValues::String(vec!["1".to_string(), "bad".to_string(), "0".to_string()]);

    match invalid.to_list::<bool>() {
        Err(ValueError::ConversionError(message)) => {
            assert!(
                message.contains("index 1"),
                "error should include failing index: {message}"
            );
        }
        other => panic!("expected indexed conversion error, got {other:?}"),
    }
}

#[test]
fn test_multi_value_new() {
    // Test generic new() method
    let mv = MultiValues::new(vec![1, 2, 3]);
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.data_type(), DataType::Int32);

    let mv = MultiValues::new(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.data_type(), DataType::String);

    let mv = MultiValues::new(vec![1u8, 2, 3, 4]);
    assert_eq!(mv.count(), 4);
    assert_eq!(mv.data_type(), DataType::UInt8);
}

#[test]
fn test_multi_value_generic_get_first() {
    // Test integer types
    let mv = MultiValues::Int32(vec![42, 100, 200]);
    let first: i32 = mv.get_first().unwrap();
    assert_eq!(first, 42);

    // Test strings type
    let mv = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    let first: String = mv.get_first().unwrap();
    assert_eq!(first, "hello");

    // Test boolean types
    let mv = MultiValues::Bool(vec![true, false, true]);
    let first: bool = mv.get_first().unwrap();
    assert!(first);

    // Test empty values
    let empty = MultiValues::Int32(vec![]);
    assert!(matches!(empty.get_first::<i32>(), Err(ValueError::NoValue)));

    // Test type mismatch
    let mv = MultiValues::Int32(vec![42]);
    assert!(matches!(
        mv.get_first::<String>(),
        Err(ValueError::TypeMismatch { .. })
    ));

    let mv = MultiValues::UInt8(vec![11u8, 22u8]);
    assert_eq!(mv.get_first::<u8>().unwrap(), 11u8);
}

#[test]
fn test_multi_value_set_methods() {
    // Test set method for basic types
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32s(vec![42, 100, 200]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);

    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bools(vec![true, false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.set_strings(vec!["hello".to_string(), "world".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);
}

#[test]
fn test_multi_value_generic_set() {
    // Test generic set method
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set(vec![42, 100, 200]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.set(vec!["hello".to_string(), "world".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);

    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set(vec![true, false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set(vec![1u8, 2, 3]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3]);
}

#[test]
fn test_multi_value_generic_set_slice() {
    // Test generic set with slice parameter
    let mut mv = MultiValues::Empty(DataType::Int32);
    let data = [42, 100, 200];
    mv.set(&data[..]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);

    let mut mv = MultiValues::Empty(DataType::String);
    let s1 = "hello".to_string();
    let s2 = "world".to_string();
    let arr = [s1.clone(), s2.clone()];
    mv.set(&arr[..]).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &[s1, s2]);
}

#[test]
fn test_multi_value_slice_methods() {
    // Test set_[xxx]s_slice and add_[xxx]s_slice
    let mut mv = MultiValues::Empty(DataType::Bool);
    let bools = [true, false, true];
    mv.set_bools_slice(&bools[..]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &bools);

    let mut mv = MultiValues::Int32(vec![1]);
    let more = [2, 3];
    mv.add_int32s_slice(&more[..]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::String);
    let a = "a".to_string();
    let b = "b".to_string();
    let arr = [a.clone(), b.clone()];
    mv.set_strings_slice(&arr[..]).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &[a, b]);
}

#[test]
fn test_multi_value_set_all_types() {
    // Test set method for all types
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bools(vec![true, false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set_chars(vec!['A', 'B', 'C']).unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['A', 'B', 'C']);

    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set_int8s(vec![1i8, 2, 3]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.set_int16s(vec![1000i16, 2000, 3000]).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1000, 2000, 3000]);

    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32s(vec![100000i32, 200000, 300000]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[100000, 200000, 300000]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set_int64s(vec![1000000000i64, 2000000000, 3000000000])
        .unwrap();
    assert_eq!(
        mv.get_int64s().unwrap(),
        &[1000000000, 2000000000, 3000000000]
    );

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set_uint8s(vec![255u8, 128, 64]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[255, 128, 64]);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.set_uint16s(vec![65535u16, 32768, 16384]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[65535, 32768, 16384]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.set_uint32s(vec![4294967295u32, 2147483648, 1073741824])
        .unwrap();
    assert_eq!(
        mv.get_uint32s().unwrap(),
        &[4294967295, 2147483648, 1073741824]
    );

    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set_float32s(vec![
        std::f32::consts::PI,
        std::f32::consts::E,
        std::f32::consts::SQRT_2,
    ])
    .unwrap();
    assert_eq!(
        mv.get_float32s().unwrap(),
        &[
            std::f32::consts::PI,
            std::f32::consts::E,
            std::f32::consts::SQRT_2
        ]
    );

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set_float64s(vec![
        std::f64::consts::PI,
        std::f64::consts::E,
        std::f64::consts::SQRT_2,
    ])
    .unwrap();
    assert_eq!(
        mv.get_float64s().unwrap(),
        &[
            std::f64::consts::PI,
            std::f64::consts::E,
            std::f64::consts::SQRT_2
        ]
    );
}

#[test]
fn test_multi_value_single_set_methods() {
    // Test single value set method for basic types
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32(42).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42]);

    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bool(true).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.set_string("hello".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello"]);
}

#[test]
fn test_multi_value_generic_single_set() {
    // Test generic single value set method
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set(42).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.set("hello".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello"]);

    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set(true).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set(42u8).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![42u8]);
}

#[test]
fn test_multi_value_single_set_all_types() {
    // Test single value set method for all types
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bool(true).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true]);

    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set_char('A').unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['A']);

    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set_int8(1i8).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1]);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.set_int16(1000i16).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1000]);

    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32(100000i32).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[100000]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set_int64(1000000000i64).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[1000000000]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set_uint8(255u8).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[255]);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.set_uint16(65535u16).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[65535]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.set_uint32(4294967295u32).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[4294967295]);

    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set_float32(std::f32::consts::PI).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[std::f32::consts::PI]);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set_float64(std::f64::consts::PI).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[std::f64::consts::PI]);
}

#[test]
fn test_multi_value_generic_add() {
    // Test generic add method
    let mut mv = MultiValues::Int32(vec![42]);
    mv.add(100).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100]);

    let mut mv = MultiValues::String(vec!["hello".to_string()]);
    mv.add("world".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);

    let mut mv = MultiValues::Bool(vec![true]);
    mv.add(false).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false]);

    let mut mv = MultiValues::UInt8(vec![1, 2, 3]);
    mv.add(4u8).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3, 4]);

    // Test adding from empty value
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add(42).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.add("hello".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello"]);
}

#[test]
fn test_multi_value_generic_add_all_types() {
    // Test generic add method for all types
    let mut mv = MultiValues::Bool(vec![true]);
    mv.add(false).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false]);

    let mut mv = MultiValues::Char(vec!['A']);
    mv.add('B').unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['A', 'B']);

    let mut mv = MultiValues::Int8(vec![1i8]);
    mv.add(2i8).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Int16(vec![1000i16]);
    mv.add(2000i16).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1000, 2000]);

    let mut mv = MultiValues::Int32(vec![100000i32]);
    mv.add(200000i32).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[100000, 200000]);

    let mut mv = MultiValues::Int64(vec![1000000000i64]);
    mv.add(2000000000i64).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[1000000000, 2000000000]);

    let mut mv = MultiValues::UInt8(vec![255u8]);
    mv.add_uint8(128u8).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[255, 128]);

    let mut mv = MultiValues::UInt16(vec![65535u16]);
    mv.add(32768u16).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[65535, 32768]);

    let mut mv = MultiValues::UInt32(vec![4294967295u32]);
    mv.add(2147483648u32).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[4294967295, 2147483648]);

    let mut mv = MultiValues::Float32(vec![std::f32::consts::PI]);
    mv.add(std::f32::consts::E).unwrap();
    assert_eq!(
        mv.get_float32s().unwrap(),
        &[std::f32::consts::PI, std::f32::consts::E]
    );

    let mut mv = MultiValues::Float64(vec![std::f64::consts::PI]);
    mv.add(std::f64::consts::E).unwrap();
    assert_eq!(
        mv.get_float64s().unwrap(),
        &[std::f64::consts::PI, std::f64::consts::E]
    );
}

#[test]
fn test_multi_value_multi_add_methods() {
    // Test multiple values add method for basic types
    let mut mv = MultiValues::Int32(vec![42]);
    mv.add_int32s(vec![100, 200]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);

    let mut mv = MultiValues::Bool(vec![true]);
    mv.add_bools(vec![false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::String(vec!["hello".to_string()]);
    mv.add_strings(vec!["world".to_string(), "rust".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world", "rust"]);
}

#[test]
fn test_multi_value_generic_multi_add() {
    // Test generic multiple values add method
    let mut mv = MultiValues::Int32(vec![42]);
    mv.add(vec![100, 200]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);

    let mut mv = MultiValues::String(vec!["hello".to_string()]);
    mv.add(vec!["world".to_string(), "rust".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world", "rust"]);

    let mut mv = MultiValues::Bool(vec![true]);
    mv.add(vec![false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::UInt8(vec![1]);
    mv.add(vec![2u8, 3, 4]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3, 4]);

    // Test adding from empty value
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add(vec![42, 100]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[42, 100]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.add(vec!["hello".to_string(), "world".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);
}

#[test]
fn test_multi_value_multi_add_all_types() {
    // Test multiple values add method for all types
    let mut mv = MultiValues::Bool(vec![true]);
    mv.add_bools(vec![false, true]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    let mut mv = MultiValues::Char(vec!['A']);
    mv.add_chars(vec!['B', 'C']).unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['A', 'B', 'C']);

    let mut mv = MultiValues::Int8(vec![1i8]);
    mv.add_int8s(vec![2i8, 3i8]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1, 2, 3]);

    let mut mv = MultiValues::Int16(vec![1000i16]);
    mv.add_int16s(vec![2000i16, 3000i16]).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1000, 2000, 3000]);

    let mut mv = MultiValues::Int32(vec![100000i32]);
    mv.add_int32s(vec![200000i32, 300000i32]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[100000, 200000, 300000]);

    let mut mv = MultiValues::Int64(vec![1000000000i64]);
    mv.add_int64s(vec![2000000000i64, 3000000000i64]).unwrap();
    assert_eq!(
        mv.get_int64s().unwrap(),
        &[1000000000, 2000000000, 3000000000]
    );

    let mut mv = MultiValues::UInt8(vec![255u8]);
    mv.add_uint8s(vec![128u8, 64u8]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[255, 128, 64]);

    let mut mv = MultiValues::UInt16(vec![65535u16]);
    mv.add_uint16s(vec![32768u16, 16384u16]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[65535, 32768, 16384]);

    let mut mv = MultiValues::UInt32(vec![4294967295u32]);
    mv.add_uint32s(vec![2147483648u32, 1073741824u32]).unwrap();
    assert_eq!(
        mv.get_uint32s().unwrap(),
        &[4294967295, 2147483648, 1073741824]
    );

    let mut mv = MultiValues::Float32(vec![std::f32::consts::PI]);
    mv.add_float32s(vec![std::f32::consts::E, std::f32::consts::SQRT_2])
        .unwrap();
    assert_eq!(
        mv.get_float32s().unwrap(),
        &[
            std::f32::consts::PI,
            std::f32::consts::E,
            std::f32::consts::SQRT_2
        ]
    );

    let mut mv = MultiValues::Float64(vec![std::f64::consts::PI]);
    mv.add_float64s(vec![std::f64::consts::E, std::f64::consts::SQRT_2])
        .unwrap();
    assert_eq!(
        mv.get_float64s().unwrap(),
        &[
            std::f64::consts::PI,
            std::f64::consts::E,
            std::f64::consts::SQRT_2
        ]
    );
}

#[test]
fn test_biginteger_multivalue() {
    // Test MultiValues BigInteger
    let mut multi =
        MultiValues::BigInteger(vec![BigInt::from(1), BigInt::from(2), BigInt::from(3)]);

    assert_eq!(multi.count(), 3);
    assert_eq!(multi.data_type(), DataType::BigInteger);

    // Get all values
    let values = multi.get_bigintegers().unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], BigInt::from(1));
    assert_eq!(values[1], BigInt::from(2));
    assert_eq!(values[2], BigInt::from(3));

    // Get first value
    let first = multi.get_first_biginteger().unwrap();
    assert_eq!(first, BigInt::from(1));

    // Add single value
    multi.add_biginteger(BigInt::from(4)).unwrap();
    assert_eq!(multi.count(), 4);

    // Add multiple values
    multi
        .add_bigintegers(vec![BigInt::from(5), BigInt::from(6)])
        .unwrap();
    assert_eq!(multi.count(), 6);

    // Add values via slice
    let new_values = vec![BigInt::from(7), BigInt::from(8)];
    multi.add_bigintegers_slice(&new_values).unwrap();
    assert_eq!(multi.count(), 8);

    // Test setting values
    let new_values = vec![
        BigInt::from_str("123456789").unwrap(),
        BigInt::from_str("987654321").unwrap(),
    ];
    multi.set_bigintegers(new_values.clone()).unwrap();
    assert_eq!(multi.count(), 2);
    assert_eq!(multi.get_bigintegers().unwrap(), &new_values);

    // Test setting values via slice
    let slice_values = vec![BigInt::from_str("111111111").unwrap()];
    multi.set_bigintegers_slice(&slice_values).unwrap();
    assert_eq!(multi.count(), 1);
    assert_eq!(
        multi.get_first_biginteger().unwrap(),
        BigInt::from_str("111111111").unwrap()
    );

    // Test setting single value
    multi
        .set_biginteger(BigInt::from_str("999999999").unwrap())
        .unwrap();
    assert_eq!(multi.count(), 1);
    assert_eq!(
        multi.get_first_biginteger().unwrap(),
        BigInt::from_str("999999999").unwrap()
    );
}

#[test]
fn test_bigdecimal_multivalue() {
    // Test MultiValues BigDecimal
    let mut multi = MultiValues::BigDecimal(vec![
        BigDecimal::from_str("1.1").unwrap(),
        BigDecimal::from_str("2.2").unwrap(),
        BigDecimal::from_str("3.3").unwrap(),
    ]);

    assert_eq!(multi.count(), 3);
    assert_eq!(multi.data_type(), DataType::BigDecimal);

    // Get all values
    let values = multi.get_bigdecimals().unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], BigDecimal::from_str("1.1").unwrap());
    assert_eq!(values[1], BigDecimal::from_str("2.2").unwrap());
    assert_eq!(values[2], BigDecimal::from_str("3.3").unwrap());

    // Get first value
    let first = multi.get_first_bigdecimal().unwrap();
    assert_eq!(first, BigDecimal::from_str("1.1").unwrap());

    // Add single value
    multi
        .add_bigdecimal(BigDecimal::from_str("4.4").unwrap())
        .unwrap();
    assert_eq!(multi.count(), 4);

    // Add multiple values
    multi
        .add_bigdecimals(vec![
            BigDecimal::from_str("5.5").unwrap(),
            BigDecimal::from_str("6.6").unwrap(),
        ])
        .unwrap();
    assert_eq!(multi.count(), 6);

    // Add values via slice
    let new_values = vec![
        BigDecimal::from_str("7.7").unwrap(),
        BigDecimal::from_str("8.8").unwrap(),
    ];
    multi.add_bigdecimals_slice(&new_values).unwrap();
    assert_eq!(multi.count(), 8);

    // Test setting values
    let new_values = vec![
        BigDecimal::from_str("123.456").unwrap(),
        BigDecimal::from_str("789.012").unwrap(),
    ];
    multi.set_bigdecimals(new_values.clone()).unwrap();
    assert_eq!(multi.count(), 2);
    assert_eq!(multi.get_bigdecimals().unwrap(), &new_values);

    // Test setting values via slice
    let slice_values = vec![BigDecimal::from_str("111.111").unwrap()];
    multi.set_bigdecimals_slice(&slice_values).unwrap();
    assert_eq!(multi.count(), 1);
    assert_eq!(
        multi.get_first_bigdecimal().unwrap(),
        BigDecimal::from_str("111.111").unwrap()
    );

    // Test setting single value
    multi
        .set_bigdecimal(BigDecimal::from_str("999.999").unwrap())
        .unwrap();
    assert_eq!(multi.count(), 1);
    assert_eq!(
        multi.get_first_bigdecimal().unwrap(),
        BigDecimal::from_str("999.999").unwrap()
    );
}

#[test]
fn test_value_to_multivalue_conversion_bigint_bigdecimal() {
    // Test Value to MultiValues conversion
    let big_int = BigInt::from_str("123456789").unwrap();
    let value = Value::BigInteger(big_int.clone());
    let multi: MultiValues = value.into();

    assert_eq!(multi.count(), 1);
    assert_eq!(multi.data_type(), DataType::BigInteger);
    assert_eq!(multi.get_first_biginteger().unwrap(), big_int);

    let big_decimal = BigDecimal::from_str("3.14159").unwrap();
    let value = Value::BigDecimal(big_decimal.clone());
    let multi: MultiValues = value.into();

    assert_eq!(multi.count(), 1);
    assert_eq!(multi.data_type(), DataType::BigDecimal);
    assert_eq!(multi.get_first_bigdecimal().unwrap(), big_decimal);
}

#[test]
fn test_biginteger_bigdecimal_merge() {
    // Test BigInteger merge
    let mut mv1 = MultiValues::BigInteger(vec![BigInt::from(1), BigInt::from(2)]);
    let mv2 = MultiValues::BigInteger(vec![BigInt::from(3), BigInt::from(4)]);

    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(
        mv1.get_bigintegers().unwrap(),
        &[
            BigInt::from(1),
            BigInt::from(2),
            BigInt::from(3),
            BigInt::from(4)
        ]
    );

    // Test BigDecimal merge
    let mut mv1 = MultiValues::BigDecimal(vec![
        BigDecimal::from_str("1.1").unwrap(),
        BigDecimal::from_str("2.2").unwrap(),
    ]);
    let mv2 = MultiValues::BigDecimal(vec![
        BigDecimal::from_str("3.3").unwrap(),
        BigDecimal::from_str("4.4").unwrap(),
    ]);

    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    let values = mv1.get_bigdecimals().unwrap();
    assert_eq!(values[0], BigDecimal::from_str("1.1").unwrap());
    assert_eq!(values[1], BigDecimal::from_str("2.2").unwrap());
    assert_eq!(values[2], BigDecimal::from_str("3.3").unwrap());
    assert_eq!(values[3], BigDecimal::from_str("4.4").unwrap());
}

// ========================================================================
// Tests to increase coverage
// ========================================================================

#[test]
fn test_multi_value_all_datetime_types() {
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Test Date multiple values
    let dates = vec![
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 20).unwrap(),
    ];
    let mut mv = MultiValues::Empty(DataType::Date);
    mv.set_dates(dates.clone()).unwrap();
    assert_eq!(mv.get_dates().unwrap(), dates.as_slice());
    assert_eq!(mv.get_first_date().unwrap(), dates[0]);

    // Test Time multiple values
    let times = vec![
        NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        NaiveTime::from_hms_opt(14, 45, 30).unwrap(),
    ];
    let mut mv = MultiValues::Empty(DataType::Time);
    mv.set_times(times.clone()).unwrap();
    assert_eq!(mv.get_times().unwrap(), times.as_slice());
    assert_eq!(mv.get_first_time().unwrap(), times[0]);

    // Test DateTime multiple values
    let datetimes = vec![
        NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 20)
            .unwrap()
            .and_hms_opt(14, 45, 30)
            .unwrap(),
    ];
    let mut mv = MultiValues::Empty(DataType::DateTime);
    mv.set_datetimes(datetimes.clone()).unwrap();
    assert_eq!(mv.get_datetimes().unwrap(), datetimes.as_slice());
    assert_eq!(mv.get_first_datetime().unwrap(), datetimes[0]);

    // Test Instant multiple values
    let instant = Utc::now();
    let mut mv = MultiValues::Empty(DataType::Instant);
    mv.set_instant(instant).unwrap();
    assert_eq!(mv.get_first_instant().unwrap(), instant);
}

#[test]
fn test_multi_value_set_type() {
    // Test setting different type clears data
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    mv.set_type(DataType::String);
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::String);

    // Test setting same type does not clear data
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    mv.set_type(DataType::Int32);
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_value_clear_all_types() {
    // Test clear method for all types

    // Empty type
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int32);

    // Bool type
    let mut mv = MultiValues::Bool(vec![true, false]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Bool);

    // Char type
    let mut mv = MultiValues::Char(vec!['a', 'b']);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Char);

    // Int8 type
    let mut mv = MultiValues::Int8(vec![1i8, 2i8, 3i8]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int8);

    // Int16 type
    let mut mv = MultiValues::Int16(vec![1i16, 2i16, 3i16]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int16);

    // Int32 type
    let mut mv = MultiValues::Int32(vec![1i32, 2i32, 3i32]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int32);

    // Int64 type
    let mut mv = MultiValues::Int64(vec![1i64, 2i64, 3i64]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int64);

    // Int128 type
    let mut mv = MultiValues::Int128(vec![1i128, 2i128, 3i128]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Int128);

    // UInt8 type
    let mut mv = MultiValues::UInt8(vec![1u8, 2u8, 3u8]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UInt8);

    // UInt16 type
    let mut mv = MultiValues::UInt16(vec![1u16, 2u16, 3u16]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UInt16);

    // UInt32 type
    let mut mv = MultiValues::UInt32(vec![1u32, 2u32, 3u32]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UInt32);

    // UInt64 type
    let mut mv = MultiValues::UInt64(vec![1u64, 2u64, 3u64]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UInt64);

    // UInt128 type
    let mut mv = MultiValues::UInt128(vec![1u128, 2u128, 3u128]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UInt128);

    // Float32 type
    let mut mv = MultiValues::Float32(vec![1.0f32, 2.0f32]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Float32);

    // Float64 type
    let mut mv = MultiValues::Float64(vec![1.0f64, 2.0f64]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Float64);

    // String type
    let mut mv = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::String);

    // Date type
    use chrono::NaiveDate;
    let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();
    let mut mv = MultiValues::Date(vec![date1, date2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Date);

    // Time type
    use chrono::NaiveTime;
    let time1 = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let time2 = NaiveTime::from_hms_opt(11, 0, 0).unwrap();
    let mut mv = MultiValues::Time(vec![time1, time2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Time);

    // DateTime type
    let dt1 = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let dt2 = NaiveDate::from_ymd_opt(2023, 1, 2)
        .unwrap()
        .and_hms_opt(11, 0, 0)
        .unwrap();
    let mut mv = MultiValues::DateTime(vec![dt1, dt2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::DateTime);

    // Instant type
    use chrono::{
        DateTime,
        Utc,
    };
    let instant1 = DateTime::parse_from_rfc3339("2023-01-01T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let instant2 = DateTime::parse_from_rfc3339("2023-01-02T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let mut mv = MultiValues::Instant(vec![instant1, instant2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Instant);

    // BigInteger type
    let big1 = BigInt::from_str("123456789012345678901234567890").unwrap();
    let big2 = BigInt::from_str("987654321098765432109876543210").unwrap();
    let mut mv = MultiValues::BigInteger(vec![big1, big2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::BigInteger);

    // BigDecimal type
    let dec1 = BigDecimal::from_str("123.456789012345678901234567890").unwrap();
    let dec2 = BigDecimal::from_str("987.654321098765432109876543210").unwrap();
    let mut mv = MultiValues::BigDecimal(vec![dec1, dec2]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::BigDecimal);
}

#[test]
fn test_multi_value_merge_all_integer_types() {
    // Test merge for all integer types
    let mut mv1 = MultiValues::Int8(vec![1i8, 2]);
    let mv2 = MultiValues::Int8(vec![3i8, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::Int16(vec![1i16, 2]);
    let mv2 = MultiValues::Int16(vec![3i16, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::Int64(vec![1i64, 2]);
    let mv2 = MultiValues::Int64(vec![3i64, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::Int128(vec![1i128, 2]);
    let mv2 = MultiValues::Int128(vec![3i128, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::UInt8(vec![1u8, 2]);
    let mv2 = MultiValues::UInt8(vec![3u8, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::UInt16(vec![1u16, 2]);
    let mv2 = MultiValues::UInt16(vec![3u16, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::UInt32(vec![1u32, 2]);
    let mv2 = MultiValues::UInt32(vec![3u32, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::UInt64(vec![1u64, 2]);
    let mv2 = MultiValues::UInt64(vec![3u64, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::UInt128(vec![1u128, 2]);
    let mv2 = MultiValues::UInt128(vec![3u128, 4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::Float32(vec![1.0f32, 2.0]);
    let mv2 = MultiValues::Float32(vec![3.0f32, 4.0]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);

    let mut mv1 = MultiValues::Float64(vec![1.0f64, 2.0]);
    let mv2 = MultiValues::Float64(vec![3.0f64, 4.0]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
}

#[test]
fn test_multi_value_merge_all_other_types() {
    use chrono::{
        DateTime,
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Bool type
    let mut mv1 = MultiValues::Bool(vec![true, false]);
    let mv2 = MultiValues::Bool(vec![true, true]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(mv1.get_bools().unwrap(), &[true, false, true, true]);

    // Char type
    let mut mv1 = MultiValues::Char(vec!['a', 'b']);
    let mv2 = MultiValues::Char(vec!['c', 'd']);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(mv1.get_chars().unwrap(), &['a', 'b', 'c', 'd']);

    // String type
    let mut mv1 = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    let mv2 = MultiValues::String(vec!["foo".to_string(), "bar".to_string()]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(
        mv1.get_strings().unwrap(),
        &["hello", "world", "foo", "bar"]
    );

    // Date type
    let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();
    let date3 = NaiveDate::from_ymd_opt(2023, 1, 3).unwrap();
    let date4 = NaiveDate::from_ymd_opt(2023, 1, 4).unwrap();
    let mut mv1 = MultiValues::Date(vec![date1, date2]);
    let mv2 = MultiValues::Date(vec![date3, date4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(mv1.get_dates().unwrap(), &[date1, date2, date3, date4]);

    // Time type
    let time1 = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let time2 = NaiveTime::from_hms_opt(11, 0, 0).unwrap();
    let time3 = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let time4 = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
    let mut mv1 = MultiValues::Time(vec![time1, time2]);
    let mv2 = MultiValues::Time(vec![time3, time4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(mv1.get_times().unwrap(), &[time1, time2, time3, time4]);

    // DateTime type
    let dt1 = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let dt2 = NaiveDate::from_ymd_opt(2023, 1, 2)
        .unwrap()
        .and_hms_opt(11, 0, 0)
        .unwrap();
    let dt3 = NaiveDate::from_ymd_opt(2023, 1, 3)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let dt4 = NaiveDate::from_ymd_opt(2023, 1, 4)
        .unwrap()
        .and_hms_opt(13, 0, 0)
        .unwrap();
    let mut mv1 = MultiValues::DateTime(vec![dt1, dt2]);
    let mv2 = MultiValues::DateTime(vec![dt3, dt4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(mv1.get_datetimes().unwrap(), &[dt1, dt2, dt3, dt4]);

    // Instant type
    let instant1 = DateTime::parse_from_rfc3339("2023-01-01T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let instant2 = DateTime::parse_from_rfc3339("2023-01-02T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let instant3 = DateTime::parse_from_rfc3339("2023-01-03T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let instant4 = DateTime::parse_from_rfc3339("2023-01-04T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let mut mv1 = MultiValues::Instant(vec![instant1, instant2]);
    let mv2 = MultiValues::Instant(vec![instant3, instant4]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 4);
    assert_eq!(
        mv1.get_instants().unwrap(),
        &[instant1, instant2, instant3, instant4]
    );
}

#[test]
fn test_multi_value_get_first_empty_all_types() {
    // Test all types return NoValue when getting first element from empty
    assert!(matches!(
        MultiValues::Int8(vec![]).get_first_int8(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::Int16(vec![]).get_first_int16(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::Int64(vec![]).get_first_int64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::Int128(vec![]).get_first_int128(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::UInt8(vec![]).get_first_uint8(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::UInt16(vec![]).get_first_uint16(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::UInt32(vec![]).get_first_uint32(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::UInt64(vec![]).get_first_uint64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::UInt128(vec![]).get_first_uint128(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::Float32(vec![]).get_first_float32(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::Float64(vec![]).get_first_float64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        MultiValues::String(vec![]).get_first_string(),
        Err(ValueError::NoValue)
    ));
}

#[test]
fn test_multi_value_add_slice_all_types() {
    // Test adding via slice for all types
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.add_int8s_slice(&[1i8, 2, 3]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1i8, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.add_int16s_slice(&[1i16, 2, 3]).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1i16, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.add_int64s_slice(&[1i64, 2, 3]).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[1i64, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.add_int128s_slice(&[1i128, 2, 3]).unwrap();
    assert_eq!(mv.get_int128s().unwrap(), &[1i128, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.add_uint8s_slice(&[1u8, 2, 3]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[1u8, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.add_uint16s_slice(&[1u16, 2, 3]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[1u16, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.add_uint32s_slice(&[1u32, 2, 3]).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[1u32, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.add_uint64s_slice(&[1u64, 2, 3]).unwrap();
    assert_eq!(mv.get_uint64s().unwrap(), &[1u64, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.add_uint128s_slice(&[1u128, 2, 3]).unwrap();
    assert_eq!(mv.get_uint128s().unwrap(), &[1u128, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.add_float32s_slice(&[1.0f32, 2.0, 3.0]).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[1.0f32, 2.0, 3.0]);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.add_float64s_slice(&[1.0f64, 2.0, 3.0]).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[1.0f64, 2.0, 3.0]);
}

#[test]
fn test_multi_value_set_single_all_types() {
    // Test single value setting for all types
    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set_char('A').unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['A']);

    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set_float32(3.5).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[3.5]);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set_float64(2.5).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[2.5]);
}

#[test]
fn test_multi_value_count_all_types() {
    // Test count method for all types
    assert_eq!(MultiValues::Bool(vec![true, false]).count(), 2);
    assert_eq!(MultiValues::Char(vec!['a', 'b', 'c']).count(), 3);
    assert_eq!(MultiValues::Int8(vec![1, 2]).count(), 2);
    assert_eq!(MultiValues::Int16(vec![1, 2, 3]).count(), 3);
    assert_eq!(MultiValues::Int64(vec![1]).count(), 1);
    assert_eq!(MultiValues::Int128(vec![1, 2, 3, 4]).count(), 4);
    assert_eq!(MultiValues::UInt8(vec![1, 2]).count(), 2);
    assert_eq!(MultiValues::UInt16(vec![1, 2, 3]).count(), 3);
    assert_eq!(MultiValues::UInt32(vec![1, 2, 3, 4]).count(), 4);
    assert_eq!(MultiValues::UInt64(vec![1, 2, 3, 4, 5]).count(), 5);
    assert_eq!(MultiValues::UInt128(vec![1, 2]).count(), 2);
    assert_eq!(MultiValues::Float32(vec![1.0, 2.0]).count(), 2);
    assert_eq!(MultiValues::Float64(vec![1.0, 2.0, 3.0]).count(), 3);
    assert_eq!(MultiValues::Empty(DataType::Int32).count(), 0);
}

#[test]
fn test_multi_value_data_type_all_variants() {
    // Test data_type method for all types
    assert_eq!(MultiValues::Bool(vec![]).data_type(), DataType::Bool);
    assert_eq!(MultiValues::Char(vec![]).data_type(), DataType::Char);
    assert_eq!(MultiValues::Int8(vec![]).data_type(), DataType::Int8);
    assert_eq!(MultiValues::Int16(vec![]).data_type(), DataType::Int16);
    assert_eq!(MultiValues::Int32(vec![]).data_type(), DataType::Int32);
    assert_eq!(MultiValues::Int64(vec![]).data_type(), DataType::Int64);
    assert_eq!(MultiValues::Int128(vec![]).data_type(), DataType::Int128);
    assert_eq!(MultiValues::UInt8(vec![]).data_type(), DataType::UInt8);
    assert_eq!(MultiValues::UInt16(vec![]).data_type(), DataType::UInt16);
    assert_eq!(MultiValues::UInt32(vec![]).data_type(), DataType::UInt32);
    assert_eq!(MultiValues::UInt64(vec![]).data_type(), DataType::UInt64);
    assert_eq!(MultiValues::UInt128(vec![]).data_type(), DataType::UInt128);
    assert_eq!(MultiValues::Float32(vec![]).data_type(), DataType::Float32);
    assert_eq!(MultiValues::Float64(vec![]).data_type(), DataType::Float64);
    assert_eq!(MultiValues::String(vec![]).data_type(), DataType::String);
    assert_eq!(
        MultiValues::Empty(DataType::Int32).data_type(),
        DataType::Int32
    );
}

#[test]
fn test_multi_value_from_value_all_types() {
    use chrono::{
        DateTime,
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Test conversion from Value to MultiValues

    // Empty type
    let v = Value::Empty(DataType::String);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.count(), 0);

    // Bool type
    let v = Value::Bool(true);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Bool);
    assert_eq!(mv.count(), 1);
    assert!(mv.get_first_bool().unwrap());

    // Char type
    let v = Value::Char('X');
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Char);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_char().unwrap(), 'X');

    // Int8 type
    let v = Value::Int8(42);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Int8);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int8().unwrap(), 42);

    // Int16 type
    let v = Value::Int16(100);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Int16);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int16().unwrap(), 100);

    // Int32 type
    let v = Value::Int32(200);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Int32);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int32().unwrap(), 200);

    // Int64 type
    let v = Value::Int64(1000);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Int64);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int64().unwrap(), 1000);

    // Int128 type
    let v = Value::Int128(10000);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Int128);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int128().unwrap(), 10000);

    // UInt8 type
    let v = Value::UInt8(255);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::UInt8);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint8().unwrap(), 255);

    // UInt16 type
    let v = Value::UInt16(65535);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::UInt16);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint16().unwrap(), 65535);

    // UInt32 type
    let v = Value::UInt32(4294967295);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::UInt32);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint32().unwrap(), 4294967295);

    // UInt64 type
    let v = Value::UInt64(18446744073709551615);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::UInt64);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint64().unwrap(), 18446744073709551615);

    // UInt128 type
    let v = Value::UInt128(340282366920938463463374607431768211455);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::UInt128);
    assert_eq!(mv.count(), 1);
    assert_eq!(
        mv.get_first_uint128().unwrap(),
        340282366920938463463374607431768211455
    );

    // Float32 type
    let v = Value::Float32(3.5);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Float32);
    assert_eq!(mv.count(), 1);
    assert!((mv.get_first_float32().unwrap() - 3.5).abs() < 0.01);

    // Float64 type
    let v = Value::Float64(2.5);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Float64);
    assert_eq!(mv.count(), 1);
    assert!((mv.get_first_float64().unwrap() - 2.5).abs() < 0.001);

    // String type
    let v = Value::String("hello".to_string());
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_string().unwrap(), "hello");

    // Date type
    let date = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
    let v = Value::Date(date);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Date);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_date().unwrap(), date);

    // Time type
    let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    let v = Value::Time(time);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Time);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_time().unwrap(), time);

    // DateTime type
    let datetime = NaiveDate::from_ymd_opt(2023, 1, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let v = Value::DateTime(datetime);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::DateTime);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_datetime().unwrap(), datetime);

    // Instant type
    let instant = DateTime::parse_from_rfc3339("2023-01-15T14:30:45Z")
        .unwrap()
        .with_timezone(&Utc);
    let v = Value::Instant(instant);
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::Instant);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_instant().unwrap(), instant);

    // BigInteger type
    let big_int = BigInt::from_str("123456789012345678901234567890").unwrap();
    let v = Value::BigInteger(big_int.clone());
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::BigInteger);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_biginteger().unwrap(), big_int);

    // BigDecimal type
    let big_dec = BigDecimal::from_str("123.456789012345678901234567890").unwrap();
    let v = Value::BigDecimal(big_dec.clone());
    let mv: MultiValues = v.into();
    assert_eq!(mv.data_type(), DataType::BigDecimal);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_bigdecimal().unwrap(), big_dec);
}

// ========================================================================
// TypeMismatch error branch tests (batch generated using macros)
// ========================================================================

#[test]
fn test_multi_value_add_all_types_mismatch_with_macro() {
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Batch test type mismatch for all add_xxx methods using macros
    test_add_type_mismatch! {
        add_bool, true, "bool";
        add_char, 'a', "char";
        add_int8, 10i8, "int8";
        add_int16, 100i16, "int16";
        add_int64, 1000i64, "int64";
        add_int128, 99999i128, "int128";
        add_uint8, 10u8, "uint8";
        add_uint16, 100u16, "uint16";
        add_uint32, 42u32, "uint32";
        add_uint64, 1000u64, "uint64";
        add_uint128, 99999u128, "uint128";
        add_float32, 3.5f32, "float32";
        add_float64, 2.71f64, "float64";
        add_string, "test".to_string(), "string";
        add_date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), "date";
        add_time, NaiveTime::from_hms_opt(10, 30, 0).unwrap(), "time";
        add_datetime, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(10, 30, 0).unwrap(), "datetime";
        add_instant, Utc::now(), "instant";
        add_biginteger, BigInt::from(12345), "biginteger";
        add_bigdecimal, BigDecimal::from_str("123.456").unwrap(), "bigdecimal";
    }
}

#[test]
fn test_multi_value_add_vec_all_types_mismatch_with_macro() {
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Test add_xxxs (Vec) method type mismatch (excluding Int32 type, as the test MultiValues is Int32)
    test_add_type_mismatch! {
        add_bools, vec![true, false], "bools";
        add_int8s, vec![1i8, 2], "int8s";
        add_int16s, vec![10i16, 20], "int16s";
        add_int64s, vec![1000i64, 2000], "int64s";
        add_int128s, vec![10000i128, 20000], "int128s";
        add_uint8s, vec![1u8, 2], "uint8s";
        add_uint16s, vec![10u16, 20], "uint16s";
        add_uint32s, vec![100u32, 200], "uint32s";
        add_uint64s, vec![1000u64, 2000], "uint64s";
        add_uint128s, vec![10000u128, 20000], "uint128s";
        add_float32s, vec![1.0f32, 2.0], "float32s";
        add_float64s, vec![1.0f64, 2.0], "float64s";
        add_strings, vec!["a".to_string(), "b".to_string()], "strings";
        add_dates, vec![NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()], "dates";
        add_times, vec![NaiveTime::from_hms_opt(10, 30, 0).unwrap()], "times";
        add_datetimes, vec![NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(10, 30, 0).unwrap()], "datetimes";
        add_instants, vec![Utc::now()], "instants";
        add_bigintegers, vec![BigInt::from(12345)], "bigintegers";
        add_bigdecimals, vec![BigDecimal::from_str("123.456").unwrap()], "bigdecimals";
    }
}

#[test]
fn test_multi_value_add_slice_all_types_mismatch_with_macro() {
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

    // Test add_xxxs_slice method type mismatch
    let bool_slice = &[true, false];
    let int8_slice = &[1i8, 2];
    let int16_slice = &[10i16, 20];
    let _int32_slice = &[42i32, 100];
    let int64_slice = &[1000i64, 2000];
    let int128_slice = &[10000i128, 20000];
    let uint8_slice = &[1u8, 2];
    let uint16_slice = &[10u16, 20];
    let uint32_slice = &[100u32, 200];
    let uint64_slice = &[1000u64, 2000];
    let uint128_slice = &[10000u128, 20000];
    let float32_slice = &[1.0f32, 2.0];
    let float64_slice = &[1.0f64, 2.0];
    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    let datetime = NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap();
    let instant = Utc::now();
    let big_int = BigInt::from(12345);
    let big_dec = BigDecimal::from_str("123.456").unwrap();

    test_add_type_mismatch! {
        add_bools_slice, bool_slice, "bools_slice";
        add_int8s_slice, int8_slice, "int8s_slice";
        add_int16s_slice, int16_slice, "int16s_slice";
        add_int64s_slice, int64_slice, "int64s_slice";
        add_int128s_slice, int128_slice, "int128s_slice";
        add_uint8s_slice, uint8_slice, "uint8s_slice";
        add_uint16s_slice, uint16_slice, "uint16s_slice";
        add_uint32s_slice, uint32_slice, "uint32s_slice";
        add_uint64s_slice, uint64_slice, "uint64s_slice";
        add_uint128s_slice, uint128_slice, "uint128s_slice";
        add_float32s_slice, float32_slice, "float32s_slice";
        add_float64s_slice, float64_slice, "float64s_slice";
        add_dates_slice, &[date], "dates_slice";
        add_times_slice, &[time], "times_slice";
        add_datetimes_slice, &[datetime], "datetimes_slice";
        add_instants_slice, &[instant], "instants_slice";
        add_bigintegers_slice, &[big_int], "bigintegers_slice";
        add_bigdecimals_slice, &[big_dec], "bigdecimals_slice";
    }
}

#[test]
fn test_multi_value_get_all_types_mismatch_with_macro() {
    // Batch test type mismatch for all get_xxx methods
    test_get_type_mismatch! {
        get_bools;
        get_chars;
        get_int8s;
        get_int16s;
        get_int64s;
        get_int128s;
        get_uint8s;
        get_uint16s;
        get_uint32s;
        get_uint64s;
        get_uint128s;
        get_float32s;
        get_float64s;
        get_strings;
        get_dates;
        get_times;
        get_datetimes;
        get_instants;
        get_bigintegers;
        get_bigdecimals;
    }
}

#[test]
fn test_multi_value_get_first_all_types_mismatch_with_macro() {
    // Batch test type mismatch for all get_first_xxx methods
    test_get_first_type_mismatch! {
        get_first_bool;
        get_first_char;
        get_first_int8;
        get_first_int16;
        get_first_int64;
        get_first_int128;
        get_first_uint8;
        get_first_uint16;
        get_first_uint32;
        get_first_uint64;
        get_first_uint128;
        get_first_float32;
        get_first_float64;
        get_first_string;
        get_first_date;
        get_first_time;
        get_first_datetime;
        get_first_instant;
        get_first_biginteger;
        get_first_bigdecimal;
    }
}

// ========================================================================
// Empty conversion full coverage tests (batch generated using macros)
// ========================================================================

#[test]
fn test_empty_to_all_integer_types_comprehensive() {
    // Batch test Empty conversion for all integer types using macros
    test_empty_to_all_int_types! {
        DataType::Int8, add_int8, add_int8s, add_int8s_slice, get_first_int8, 42i8, vec![1i8, 2, 3];
        DataType::Int16, add_int16, add_int16s, add_int16s_slice, get_first_int16, 100i16, vec![10i16, 20, 30];
        DataType::Int32, add_int32, add_int32s, add_int32s_slice, get_first_int32, 200i32, vec![100i32, 200, 300];
        DataType::Int64, add_int64, add_int64s, add_int64s_slice, get_first_int64, 1000i64, vec![500i64, 1000, 1500];
        DataType::Int128, add_int128, add_int128s, add_int128s_slice, get_first_int128, 9999i128, vec![1000i128, 2000, 3000];
        DataType::UInt8, add_uint8, add_uint8s, add_uint8s_slice, get_first_uint8, 255u8, vec![1u8, 2, 3];
        DataType::UInt16, add_uint16, add_uint16s, add_uint16s_slice, get_first_uint16, 65535u16, vec![100u16, 200, 300];
        DataType::UInt32, add_uint32, add_uint32s, add_uint32s_slice, get_first_uint32, 99999u32, vec![1000u32, 2000, 3000];
        DataType::UInt64, add_uint64, add_uint64s, add_uint64s_slice, get_first_uint64, 999999u64, vec![10000u64, 20000, 30000];
        DataType::UInt128, add_uint128, add_uint128s, add_uint128s_slice, get_first_uint128, 888888u128, vec![100000u128, 200000, 300000];
        DataType::Float32, add_float32, add_float32s, add_float32s_slice, get_first_float32, 3.5f32, vec![1.1f32, 2.2, 3.3];
        DataType::Float64, add_float64, add_float64s, add_float64s_slice, get_first_float64, 2.5f64, vec![1.11f64, 2.22, 3.33];
    }
}

#[test]
fn test_empty_type_mismatch_comprehensive() {
    // Test adding other types to Empty(Int32) should error
    test_empty_type_mismatch_errors! {
        DataType::Int32, DataType::Int32;
        add_bool, true;
        add_char, 'x';
        add_int8, 10i8;
        add_int16, 100i16;
        add_int64, 1000i64;
        add_int128, 10000i128;
        add_uint8, 10u8;
        add_uint16, 100u16;
        add_uint32, 1000u32;
        add_uint64, 10000u64;
        add_uint128, 100000u128;
        add_float32, 3.5f32;
        add_float64, 2.5f64;
        add_string, "test".to_string();
    }

    // Test adding numeric types to Empty(String) should error
    test_empty_type_mismatch_errors! {
        DataType::String, DataType::String;
        add_bool, true;
        add_int32, 42i32;
        add_int64, 1000i64;
        add_float64, 3.5f64;
    }

    // Test adding other types to Empty(Bool) should error
    test_empty_type_mismatch_errors! {
        DataType::Bool, DataType::Bool;
        add_int32, 42i32;
        add_string, "test".to_string();
        add_float64, 3.5f64;
    }
}

#[test]
fn test_get_first_from_empty_all_types() {
    // Batch test getting first from Empty returns NoValue
    test_get_first_empty_no_value! {
        DataType::Bool, get_first_bool;
        DataType::Char, get_first_char;
        DataType::Int8, get_first_int8;
        DataType::Int16, get_first_int16;
        DataType::Int32, get_first_int32;
        DataType::Int64, get_first_int64;
        DataType::Int128, get_first_int128;
        DataType::UInt8, get_first_uint8;
        DataType::UInt16, get_first_uint16;
        DataType::UInt32, get_first_uint32;
        DataType::UInt64, get_first_uint64;
        DataType::UInt128, get_first_uint128;
        DataType::Float32, get_first_float32;
        DataType::Float64, get_first_float64;
        DataType::String, get_first_string;
        DataType::Date, get_first_date;
        DataType::Time, get_first_time;
        DataType::DateTime, get_first_datetime;
        DataType::Instant, get_first_instant;
        DataType::BigInteger, get_first_biginteger;
        DataType::BigDecimal, get_first_bigdecimal;
    }
}

#[test]
fn test_get_from_empty_returns_empty_vec() {
    // For Empty with matching declared type, getting list returns empty Vec.
    let mv = MultiValues::Empty(DataType::Int32);
    assert_eq!(mv.get_int32s().unwrap(), &[] as &[i32]);

    let mv = MultiValues::Empty(DataType::String);
    assert_eq!(mv.get_strings().unwrap(), &[] as &[String]);

    let mv = MultiValues::Empty(DataType::Bool);
    assert_eq!(mv.get_bools().unwrap(), &[] as &[bool]);

    let mv = MultiValues::Empty(DataType::Float64);
    assert_eq!(mv.get_float64s().unwrap(), &[] as &[f64]);

    let mv = MultiValues::Empty(DataType::Int8);
    assert_eq!(mv.get_int8s().unwrap(), &[] as &[i8]);

    let mv = MultiValues::Empty(DataType::UInt64);
    assert_eq!(mv.get_uint64s().unwrap(), &[] as &[u64]);
}

#[test]
fn test_get_from_empty_mismatched_type_returns_error() {
    let mv = MultiValues::Empty(DataType::Int32);
    assert!(matches!(
        mv.get_strings(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        mv.get_bools(),
        Err(ValueError::TypeMismatch { .. })
    ));

    let mv = MultiValues::Empty(DataType::String);
    assert!(matches!(
        mv.get_int32s(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_add_to_existing_multivalue_comprehensive() {
    // Batch test adding elements to existing MultiValues using macros
    test_add_to_existing! {
        MultiValues::Int8(vec![1i8]), add_int8, 10i8,
            add_int8s, vec![20i8, 30i8], add_int8s_slice, 2;
        MultiValues::Int16(vec![100i16]), add_int16, 200i16,
            add_int16s, vec![300i16, 400i16], add_int16s_slice, 2;
        MultiValues::Int32(vec![1000]), add_int32, 2000,
            add_int32s, vec![3000, 4000], add_int32s_slice, 2;
        MultiValues::Int64(vec![10000i64]), add_int64, 20000i64,
            add_int64s, vec![30000i64, 40000i64], add_int64s_slice, 2;
        MultiValues::Int128(vec![100000i128]), add_int128, 200000i128,
            add_int128s, vec![300000i128, 400000i128], add_int128s_slice, 2;
        MultiValues::UInt8(vec![1u8]), add_uint8, 10u8,
            add_uint8s, vec![20u8, 30u8], add_uint8s_slice, 2;
        MultiValues::UInt16(vec![100u16]), add_uint16, 200u16,
            add_uint16s, vec![300u16, 400u16], add_uint16s_slice, 2;
        MultiValues::UInt32(vec![1000u32]), add_uint32, 2000u32,
            add_uint32s, vec![3000u32, 4000u32], add_uint32s_slice, 2;
        MultiValues::UInt64(vec![10000u64]), add_uint64, 20000u64,
            add_uint64s, vec![30000u64, 40000u64], add_uint64s_slice, 2;
        MultiValues::UInt128(vec![100000u128]), add_uint128, 200000u128,
            add_uint128s, vec![300000u128, 400000u128], add_uint128s_slice, 2;
        MultiValues::Float32(vec![1.1f32]), add_float32, 2.2f32,
            add_float32s, vec![3.3f32, 4.4f32], add_float32s_slice, 2;
        MultiValues::Float64(vec![1.11f64]), add_float64, 2.22f64,
            add_float64s, vec![3.33f64, 4.44f64], add_float64s_slice, 2;
        MultiValues::String(vec!["hello".to_string()]), add_string, "world".to_string(),
            add_strings, vec!["foo".to_string(), "bar".to_string()], add_strings_slice, 2;
        MultiValues::Bool(vec![true]), add_bool, false,
            add_bools, vec![true, false], add_bools_slice, 2;
        MultiValues::Char(vec!['a']), add_char, 'b',
            add_chars, vec!['c', 'd'], add_chars_slice, 2;
    }
}

#[test]
fn test_set_single_all_types() {
    // Test set_bool
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bool(true).unwrap();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_bools().unwrap(), &[true]);

    // Test setting on existing data (will replace)
    let mut mv = MultiValues::Bool(vec![true, false, true]);
    mv.set_bool(false).unwrap();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_bools().unwrap(), &[false]);

    // Test all integer types
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set_int8(42i8).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[42i8]);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.set_int16(100i16).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[100i16]);

    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32(1000).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[1000]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set_int64(10000i64).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[10000i64]);

    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.set_int128(100000i128).unwrap();
    assert_eq!(mv.get_int128s().unwrap(), &[100000i128]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set_uint8(255u8).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[255u8]);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.set_uint16(65535u16).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[65535u16]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.set_uint32(99999u32).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[99999u32]);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.set_uint64(999999u64).unwrap();
    assert_eq!(mv.get_uint64s().unwrap(), &[999999u64]);

    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.set_uint128(9999999u128).unwrap();
    assert_eq!(mv.get_uint128s().unwrap(), &[9999999u128]);

    // Test floating point types
    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set_float32(3.5f32).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[3.5f32]);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set_float64(2.5f64).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[2.5f64]);

    // Test strings type
    let mut mv = MultiValues::Empty(DataType::String);
    mv.set_string("hello".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello"]);

    // Test character types
    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set_char('x').unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['x']);
}

#[test]
fn test_set_vec_and_slice_all_types() {
    // Test set_bools (Vec)
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set_bools(vec![true, false, true]).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    // Test set_bools_slice
    let mut mv = MultiValues::Bool(vec![false]);
    let values = [true, true, false];
    mv.set_bools_slice(&values[..]).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_bools().unwrap(), &[true, true, false]);

    // Test integer types Vec
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set_int8s(vec![1i8, 2, 3]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1i8, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set_int32s(vec![100, 200, 300]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[100, 200, 300]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set_int64s(vec![1000i64, 2000, 3000]).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[1000i64, 2000, 3000]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set_uint8s(vec![10u8, 20, 30]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[10u8, 20, 30]);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.set_uint64s(vec![10000u64, 20000, 30000]).unwrap();
    assert_eq!(mv.get_uint64s().unwrap(), &[10000u64, 20000, 30000]);

    // Test integer types slice
    let mut mv = MultiValues::Int16(vec![999i16]);
    let values = [10i16, 20, 30];
    mv.set_int16s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[10i16, 20, 30]);

    let mut mv = MultiValues::Int128(vec![888i128]);
    let values = [100i128, 200, 300];
    mv.set_int128s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_int128s().unwrap(), &[100i128, 200, 300]);

    let mut mv = MultiValues::UInt16(vec![777u16]);
    let values = [10u16, 20, 30];
    mv.set_uint16s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[10u16, 20, 30]);

    let mut mv = MultiValues::UInt32(vec![666u32]);
    let values = [100u32, 200, 300];
    mv.set_uint32s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[100u32, 200, 300]);

    let mut mv = MultiValues::UInt128(vec![555u128]);
    let values = [1000u128, 2000, 3000];
    mv.set_uint128s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_uint128s().unwrap(), &[1000u128, 2000, 3000]);

    // Test floating point types
    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set_float32s(vec![1.1f32, 2.2, 3.3]).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[1.1f32, 2.2, 3.3]);

    let mut mv = MultiValues::Float64(vec![9.9f64]);
    let values = [1.11f64, 2.22, 3.33];
    mv.set_float64s_slice(&values[..]).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[1.11f64, 2.22, 3.33]);

    // Test strings type
    let mut mv = MultiValues::Empty(DataType::String);
    mv.set_strings(vec!["hello".to_string(), "world".to_string()])
        .unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);

    let mut mv = MultiValues::String(vec!["old".to_string()]);
    let values = ["foo".to_string(), "bar".to_string()];
    mv.set_strings_slice(&values[..]).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["foo", "bar"]);

    // Test character types
    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set_chars(vec!['a', 'b', 'c']).unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['a', 'b', 'c']);

    let mut mv = MultiValues::Char(vec!['z']);
    let values = ['x', 'y'];
    mv.set_chars_slice(&values[..]).unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['x', 'y']);
}

#[test]
fn test_generic_set_method_comprehensive() {
    // Test generic set method in three forms: Vec, slice, single value

    // 1. Vec<T> form
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set(vec![1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2, 3]);

    // 2. &[T] form
    let mut mv = MultiValues::Empty(DataType::Bool);
    let slice = &[true, false, true][..];
    mv.set(slice).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    // 3. single T form
    let mut mv = MultiValues::Empty(DataType::String);
    mv.set("hello".to_string()).unwrap();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_strings().unwrap(), &["hello"]);

    // Test more types
    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set(vec![1.1, 2.2, 3.3]).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[1.1, 2.2, 3.3]);

    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set('x').unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['x']);

    // Test integer types
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set(vec![1i8, 2, 3]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1i8, 2, 3]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    let slice = &[100i64, 200, 300][..];
    mv.set(slice).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[100i64, 200, 300]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.set(42u32).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[42u32]);
}

#[test]
fn test_generic_add_method_comprehensive() {
    // Test generic add method in three forms: Vec, slice, single value

    // 1. Vec<T> form to Empty
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add(vec![1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2, 3]);

    // 2. Vec<T> form to existing list
    let mut mv = MultiValues::Int32(vec![10]);
    mv.add(vec![20, 30]).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_int32s().unwrap(), &[10, 20, 30]);

    // 3. &[T] form
    let mut mv = MultiValues::Empty(DataType::Bool);
    let slice = &[true, false][..];
    mv.add(slice).unwrap();
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_bools().unwrap(), &[true, false]);

    // 4. single T form
    let mut mv = MultiValues::Empty(DataType::String);
    mv.add("hello".to_string()).unwrap();
    assert_eq!(mv.count(), 1);

    mv.add("world".to_string()).unwrap();
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);

    // Test more types
    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.add(1.1f64).unwrap();
    mv.add(vec![2.2, 3.3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    let slice = &[100u64, 200u64][..];
    mv.add(slice).unwrap();
    mv.add(300u64).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_uint64s().unwrap(), &[100u64, 200u64, 300u64]);
}

// Keep original detailed tests as supplements
#[test]
fn test_multi_value_add_type_mismatch_errors() {
    // Test add_bool type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_bool(true);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_char type mismatch
    let mut mv = MultiValues::String(vec!["test".to_string()]);
    let result = mv.add_char('a');
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int8 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_int8(10);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int16 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_int16(100);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int32 type mismatch
    let mut mv = MultiValues::Int64(vec![1, 2, 3]);
    let result = mv.add_int32(42);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int64 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_int64(1000);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int128 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_int128(99999);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_uint8 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_uint8(10);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_uint16 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_uint16(100);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_uint32 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_uint32(42);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_uint64 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_uint64(1000);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_uint128 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_uint128(99999);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_float32 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_float32(3.5);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_float64 type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_float64(2.71);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_string type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_string("test".to_string());
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_date type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let result = mv.add_date(date);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_time type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let time = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    let result = mv.add_time(time);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_datetime type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let datetime = chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap();
    let result = mv.add_datetime(datetime);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_instant type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let instant = chrono::Utc::now();
    let result = mv.add_instant(instant);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_biginteger type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_int = BigInt::from(12345);
    let result = mv.add_biginteger(big_int);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_bigdecimal type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_dec = BigDecimal::from_str("123.456").unwrap();
    let result = mv.add_bigdecimal(big_dec);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_value_add_multi_type_mismatch_errors() {
    // Test add_bools type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_bools(vec![true, false]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int32s type mismatch
    let mut mv = MultiValues::Int64(vec![1, 2, 3]);
    let result = mv.add_int32s(vec![42, 100]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int64s type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_int64s(vec![1000, 2000]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_strings type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add_strings(vec!["a".to_string(), "b".to_string()]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_dates type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let result = mv.add_dates(vec![date]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_times type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let time = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    let result = mv.add_times(vec![time]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_datetimes type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let datetime = chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap();
    let result = mv.add_datetimes(vec![datetime]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_instants type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let instant = chrono::Utc::now();
    let result = mv.add_instants(vec![instant]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_bigintegers type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_int = BigInt::from(12345);
    let result = mv.add_bigintegers(vec![big_int]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_bigdecimals type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_dec = BigDecimal::from_str("123.456").unwrap();
    let result = mv.add_bigdecimals(vec![big_dec]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_value_add_slice_type_mismatch_errors() {
    // Test add_bools_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let values = &[true, false];
    let result = mv.add_bools_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_int32s_slice type mismatch
    let mut mv = MultiValues::Int64(vec![1, 2, 3]);
    let values = &[42, 100];
    let result = mv.add_int32s_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_float32s_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let values = &[1.0f32, 2.0f32];
    let result = mv.add_float32s_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_dates_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let values = &[date];
    let result = mv.add_dates_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_times_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let time = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    let values = &[time];
    let result = mv.add_times_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_datetimes_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let datetime = chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap();
    let values = &[datetime];
    let result = mv.add_datetimes_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_instants_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let instant = chrono::Utc::now();
    let values = &[instant];
    let result = mv.add_instants_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_bigintegers_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_int = BigInt::from(12345);
    let values = &[big_int];
    let result = mv.add_bigintegers_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Test add_bigdecimals_slice type mismatch
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let big_dec = BigDecimal::from_str("123.456").unwrap();
    let values = &[big_dec];
    let result = mv.add_bigdecimals_slice(values);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

// ========================================================================
// Empty type conversion tests
// ========================================================================

#[test]
fn test_multi_value_empty_add_single_conversion() {
    // Empty(Bool) + add_bool -> Bool(vec![value])
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.add_bool(true).unwrap();
    assert_eq!(mv.data_type(), DataType::Bool);
    assert_eq!(mv.count(), 1);
    assert!(mv.get_first_bool().unwrap());

    // Empty(Int32) + add_int32 -> Int32(vec![value])
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add_int32(42).unwrap();
    assert_eq!(mv.data_type(), DataType::Int32);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int32().unwrap(), 42);

    // Empty(String) + add_string -> String(vec![value])
    let mut mv = MultiValues::Empty(DataType::String);
    mv.add_string("hello".to_string()).unwrap();
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_string().unwrap(), "hello");

    // Empty(Char) + add_char -> Char(vec![value])
    let mut mv = MultiValues::Empty(DataType::Char);
    mv.add_char('x').unwrap();
    assert_eq!(mv.data_type(), DataType::Char);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_char().unwrap(), 'x');

    // Empty(Float64) + add_float64 -> Float64(vec![value])
    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.add_float64(3.5).unwrap();
    assert_eq!(mv.data_type(), DataType::Float64);
    assert_eq!(mv.count(), 1);
    assert!((mv.get_first_float64().unwrap() - 3.5).abs() < 1e-10);

    // Empty(Date) + add_date -> Date(vec![value])
    let mut mv = MultiValues::Empty(DataType::Date);
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    mv.add_date(date).unwrap();
    assert_eq!(mv.data_type(), DataType::Date);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_date().unwrap(), date);

    // Empty(Time) + add_time -> Time(vec![value])
    let mut mv = MultiValues::Empty(DataType::Time);
    let time = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    mv.add_time(time).unwrap();
    assert_eq!(mv.data_type(), DataType::Time);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_time().unwrap(), time);

    // Empty(DateTime) + add_datetime -> DateTime(vec![value])
    let mut mv = MultiValues::Empty(DataType::DateTime);
    let datetime = chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .and_hms_opt(10, 30, 0)
        .unwrap();
    mv.add_datetime(datetime).unwrap();
    assert_eq!(mv.data_type(), DataType::DateTime);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_datetime().unwrap(), datetime);

    // Empty(Instant) + add_instant -> Instant(vec![value])
    let mut mv = MultiValues::Empty(DataType::Instant);
    let instant = chrono::Utc::now();
    mv.add_instant(instant).unwrap();
    assert_eq!(mv.data_type(), DataType::Instant);
    assert_eq!(mv.count(), 1);

    // Empty(BigInteger) + add_biginteger -> BigInteger(vec![value])
    let mut mv = MultiValues::Empty(DataType::BigInteger);
    let big_int = BigInt::from(12345);
    mv.add_biginteger(big_int.clone()).unwrap();
    assert_eq!(mv.data_type(), DataType::BigInteger);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_biginteger().unwrap(), big_int);

    // Empty(BigDecimal) + add_bigdecimal -> BigDecimal(vec![value])
    let mut mv = MultiValues::Empty(DataType::BigDecimal);
    let big_dec = BigDecimal::from_str("123.456").unwrap();
    mv.add_bigdecimal(big_dec.clone()).unwrap();
    assert_eq!(mv.data_type(), DataType::BigDecimal);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_bigdecimal().unwrap(), big_dec);

    // Test Empty conversion for all integer types
    // Empty(Int8) + add_int8 -> Int8(vec![value])
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.add_int8(42).unwrap();
    assert_eq!(mv.data_type(), DataType::Int8);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int8().unwrap(), 42);

    // Empty(Int16) + add_int16 -> Int16(vec![value])
    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.add_int16(1000).unwrap();
    assert_eq!(mv.data_type(), DataType::Int16);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int16().unwrap(), 1000);

    // Empty(Int64) + add_int64 -> Int64(vec![value])
    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.add_int64(999999).unwrap();
    assert_eq!(mv.data_type(), DataType::Int64);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int64().unwrap(), 999999);

    // Empty(Int128) + add_int128 -> Int128(vec![value])
    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.add_int128(123456789).unwrap();
    assert_eq!(mv.data_type(), DataType::Int128);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_int128().unwrap(), 123456789);

    // Empty(UInt8) + add_uint8 -> UInt8(vec![value])
    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.add_uint8(255).unwrap();
    assert_eq!(mv.data_type(), DataType::UInt8);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint8().unwrap(), 255);

    // Empty(UInt16) + add_uint16 -> UInt16(vec![value])
    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.add_uint16(65535).unwrap();
    assert_eq!(mv.data_type(), DataType::UInt16);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint16().unwrap(), 65535);

    // Empty(UInt32) + add_uint32 -> UInt32(vec![value])
    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.add_uint32(4294967295).unwrap();
    assert_eq!(mv.data_type(), DataType::UInt32);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint32().unwrap(), 4294967295);

    // Empty(UInt64) + add_uint64 -> UInt64(vec![value])
    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.add_uint64(9999999999).unwrap();
    assert_eq!(mv.data_type(), DataType::UInt64);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint64().unwrap(), 9999999999);

    // Empty(UInt128) + add_uint128 -> UInt128(vec![value])
    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.add_uint128(123456789012345).unwrap();
    assert_eq!(mv.data_type(), DataType::UInt128);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_uint128().unwrap(), 123456789012345);

    // Empty(Float32) + add_float32 -> Float32(vec![value])
    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.add_float32(2.5).unwrap();
    assert_eq!(mv.data_type(), DataType::Float32);
    assert_eq!(mv.count(), 1);
    assert!((mv.get_first_float32().unwrap() - 2.5).abs() < 1e-3);
}

#[test]
fn test_multi_value_empty_add_multi_conversion() {
    // Empty(Bool) + add_bools -> Bool(vec![values])
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.add_bools(vec![true, false, true]).unwrap();
    assert_eq!(mv.data_type(), DataType::Bool);
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_bools().unwrap(), &[true, false, true]);

    // Empty(Int32) + add_int32s -> Int32(vec![values])
    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.add_int32s(vec![1, 2, 3]).unwrap();
    assert_eq!(mv.data_type(), DataType::Int32);
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2, 3]);

    // Empty(String) + add_strings -> String(vec![values])
    let mut mv = MultiValues::Empty(DataType::String);
    mv.add_strings(vec!["a".to_string(), "b".to_string()])
        .unwrap();
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_strings().unwrap(), &["a", "b"]);

    // Test Empty + add_xxxs for all integer types
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.add_int8s(vec![1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.add_int16s(vec![10, 20, 30]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.add_int64s(vec![100, 200, 300]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.add_int128s(vec![1000, 2000, 3000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.add_uint8s(vec![1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.add_uint16s(vec![10, 20, 30]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.add_uint32s(vec![100, 200, 300]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.add_uint64s(vec![1000, 2000, 3000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.add_uint128s(vec![10000, 20000, 30000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.add_float32s(vec![1.1, 2.2, 3.3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.add_float64s(vec![1.11, 2.22, 3.33]).unwrap();
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_value_empty_add_slice_conversion() {
    // Empty(Int32) + add_int32s_slice -> Int32(vec![values])
    let mut mv = MultiValues::Empty(DataType::Int32);
    let values = &[10, 20, 30];
    mv.add_int32s_slice(values).unwrap();
    assert_eq!(mv.data_type(), DataType::Int32);
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_int32s().unwrap(), &[10, 20, 30]);

    // Empty(Float32) + add_float32s_slice -> Float32(vec![values])
    let mut mv = MultiValues::Empty(DataType::Float32);
    let values = &[1.0f32, 2.0f32, 3.0f32];
    mv.add_float32s_slice(values).unwrap();
    assert_eq!(mv.data_type(), DataType::Float32);
    assert_eq!(mv.count(), 3);

    // Test Empty + add_xxxs for all integer types_slice
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.add_int8s_slice(&[1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.add_int16s_slice(&[10, 20, 30]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.add_int64s_slice(&[100, 200, 300]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.add_int128s_slice(&[1000, 2000, 3000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.add_uint8s_slice(&[1, 2, 3]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.add_uint16s_slice(&[10, 20, 30]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.add_uint32s_slice(&[100, 200, 300]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.add_uint64s_slice(&[1000, 2000, 3000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.add_uint128s_slice(&[10000, 20000, 30000]).unwrap();
    assert_eq!(mv.count(), 3);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.add_float64s_slice(&[1.11, 2.22, 3.33]).unwrap();
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_value_empty_type_mismatch() {
    // Empty(Int32) + add_bool should error
    let mut mv = MultiValues::Empty(DataType::Int32);
    let result = mv.add_bool(true);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Empty(String) + add_int32 should error
    let mut mv = MultiValues::Empty(DataType::String);
    let result = mv.add_int32(42);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Empty(Bool) + add_strings should error
    let mut mv = MultiValues::Empty(DataType::Bool);
    let result = mv.add_strings(vec!["test".to_string()]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

// ========================================================================
// Generic method error branch tests
// ========================================================================

#[test]
fn test_multi_value_direct_get_type_mismatch() {
    // Create Int32 MultiValues, try to get with String type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_strings();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create String MultiValues, try to get with i32 type
    let mv = MultiValues::String(vec!["a".to_string(), "b".to_string()]);
    let result = mv.get_int32s();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Bool MultiValues, try to get with i64 type
    let mv = MultiValues::Bool(vec![true, false]);
    let result = mv.get_int64s();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Date type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_dates();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Time type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_times();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with DateTime type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_datetimes();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Instant type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_instants();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with BigInteger type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_bigintegers();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with BigDecimal type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_bigdecimals();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_value_direct_get_first_type_mismatch() {
    // Create Int32 MultiValues, try to get_first with bool type
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_bool();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create String MultiValues, try to get with i32 type_first
    let mv = MultiValues::String(vec!["test".to_string()]);
    let result = mv.get_first_int32();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Date type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_date();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Time type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_time();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with DateTime type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_datetime();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with Instant type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_instant();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with BigInteger type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_biginteger();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Int32 MultiValues, try to get with BigDecimal type_first
    let mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.get_first_bigdecimal();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_value_direct_add_type_mismatch() {
    // Create Int32 MultiValues, try to add String
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    let result = mv.add("test".to_string());
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create Bool MultiValues, try to add i32
    let mut mv = MultiValues::Bool(vec![true]);
    let result = mv.add(42i32);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    // Create String MultiValues, try to add f64
    let mut mv = MultiValues::String(vec!["a".to_string()]);
    let result = mv.add(3.5f64);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

// ============================================================================
// Tests for uncovered branches
// ============================================================================

#[test]
fn test_multi_value_merge_empty_branch() {
    // Empty + Empty keeps empty type.
    let mut mv1 = MultiValues::Empty(DataType::Int32);
    let mv2 = MultiValues::Empty(DataType::Int32);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 0);
    assert_eq!(mv1.data_type(), DataType::Int32);

    // Empty + Filled should absorb values from the other side.
    let mut mv1 = MultiValues::Empty(DataType::String);
    let mv2 = MultiValues::String(vec!["a".to_string(), "b".to_string()]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 2);
    assert_eq!(mv1.data_type(), DataType::String);
    assert_eq!(mv1.get_strings().unwrap(), &["a", "b"]);
}

#[test]
fn test_multi_values_getter_empty_branch() {
    // For Empty with matching declared type, generic getter returns empty Vec.
    let mv = MultiValues::Empty(DataType::Int32);
    let result: Vec<i32> = mv.get().unwrap();
    assert_eq!(result.len(), 0);

    let mv = MultiValues::Empty(DataType::String);
    let result: Vec<String> = mv.get().unwrap();
    assert_eq!(result.len(), 0);

    let mv = MultiValues::Empty(DataType::Bool);
    let result: Vec<bool> = mv.get().unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_multi_values_getter_empty_type_mismatch_branch() {
    let mv = MultiValues::Empty(DataType::Int32);
    let result: Result<Vec<String>, ValueError> = mv.get();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    let result: Result<bool, ValueError> = mv.get_first();
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_values_first_getter_non_empty_branch() {
    // Test normal branch of v[0] in MultiValuesFirstGetter
    // Ensure all types can correctly get first element

    let mv = MultiValues::Int32(vec![42, 100]);
    let result: i32 = mv.get_first().unwrap();
    assert_eq!(result, 42);

    let mv = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    let result: String = mv.get_first().unwrap();
    assert_eq!(result, "hello");

    let mv = MultiValues::Bool(vec![true, false, true]);
    let result: bool = mv.get_first().unwrap();
    assert!(result);

    let mv = MultiValues::Float64(vec![3.5, 2.5]);
    let result: f64 = mv.get_first().unwrap();
    assert!((result - 3.5).abs() < 0.001);
}

#[test]
fn test_multi_values_adder_type_mismatch_branch() {
    // Test _ branch of MultiValuesMultiAdder (type mismatch)
    let mut mv = MultiValues::Int32(vec![1, 2]);
    // Try to add Vec<String> to Int32 MultiValues
    let result = mv.add(vec!["a".to_string(), "b".to_string()]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    let mut mv = MultiValues::Bool(vec![true]);
    // Try to add Vec<f64> to Bool MultiValues
    let result = mv.add(vec![1.0f64, 2.0f64]);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_values_adder_slice_type_mismatch_branch() {
    // Test _ branch of MultiValuesMultiAdderSlice (type mismatch)
    let mut mv = MultiValues::String(vec!["hello".to_string()]);
    let int_slice: &[i32] = &[42i32, 100];
    // Try to add &[i32] to String MultiValues
    let result = mv.add(int_slice);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));

    let mut mv = MultiValues::Int32(vec![1, 2]);
    let bool_slice: &[bool] = &[true, false];
    // Try to add &[bool] to Int32 MultiValues
    let result = mv.add(bool_slice);
    assert!(matches!(result, Err(ValueError::TypeMismatch { .. })));
}

#[test]
fn test_multi_values_set_with_str_slice() {
    // Test impl<'b, 'a> MultiValuesSetArg<'a> for &'b [&'b str]
    let mut mv = MultiValues::Empty(DataType::String);
    let str_slice: &[&str] = &["hello", "world", "test"];
    mv.set(str_slice).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world", "test"]);

    // Replace existing values
    let mut mv = MultiValues::String(vec!["old".to_string()]);
    let str_slice: &[&str] = &["new1", "new2"];
    mv.set(str_slice).unwrap();
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_strings().unwrap(), &["new1", "new2"]);
}

#[test]
fn test_multi_values_set_with_str_vec() {
    // Test impl<'b, 'a> MultiValuesSetArg<'a> for Vec<&'b str>
    // Especially the line into_iter().map(|s| s.to_string()).collect()
    let mut mv = MultiValues::Empty(DataType::String);
    let str_vec: Vec<&str> = vec!["alpha", "beta", "gamma"];
    mv.set(str_vec).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_strings().unwrap(), &["alpha", "beta", "gamma"]);

    // Test conversion of multiple elements
    let mut mv = MultiValues::String(vec!["old1".to_string(), "old2".to_string()]);
    let str_vec: Vec<&str> = vec!["foo", "bar", "baz", "qux"];
    mv.set(str_vec).unwrap();
    assert_eq!(mv.count(), 4);
    assert_eq!(mv.get_strings().unwrap(), &["foo", "bar", "baz", "qux"]);
}

#[test]
fn test_multi_values_add_with_str_vec() {
    // Test impl<'b, 'a> MultiValuesAddArg<'a> for Vec<&'b str>
    let mut mv = MultiValues::String(vec!["hello".to_string()]);
    let str_vec: Vec<&str> = vec!["world", "rust"];
    mv.add(str_vec).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_strings().unwrap(), &["hello", "world", "rust"]);

    // Test adding starting from Empty
    let mut mv = MultiValues::Empty(DataType::String);
    let str_vec: Vec<&str> = vec!["first", "second", "third"];
    mv.add(str_vec).unwrap();
    assert_eq!(mv.count(), 3);
    assert_eq!(mv.get_strings().unwrap(), &["first", "second", "third"]);
}

#[test]
fn test_multi_values_add_with_str_slice() {
    // Test impl<'b, 'a> MultiValuesAddArg<'a> for &'b [&'b str]
    let mut mv = MultiValues::String(vec!["existing".to_string()]);
    let str_slice: &[&str] = &["new1", "new2", "new3"];
    mv.add(str_slice).unwrap();
    assert_eq!(mv.count(), 4);
    assert_eq!(
        mv.get_strings().unwrap(),
        &["existing", "new1", "new2", "new3"]
    );

    // Test adding starting from Empty
    let mut mv = MultiValues::Empty(DataType::String);
    let str_slice: &[&str] = &["a", "b"];
    mv.add(str_slice).unwrap();
    assert_eq!(mv.count(), 2);
    assert_eq!(mv.get_strings().unwrap(), &["a", "b"]);
}

/// Test MultiValues set operation using &str
#[test]
fn test_multi_values_set_str_ref() {
    let mut mv = MultiValues::Empty(DataType::String);

    // Set single value using &str
    mv.set("hello").unwrap();
    assert_eq!(mv.get_first_string().unwrap(), "hello");
    assert_eq!(mv.count(), 1);
}

/// Test MultiValues set operation using Vec<&str>
#[test]
fn test_multi_values_set_vec_str_ref() {
    let mut mv = MultiValues::Empty(DataType::String);

    // Set multiple values using Vec<&str>
    let str_vec = vec!["apple", "banana", "cherry"];
    mv.set(str_vec).unwrap();

    let strings = mv.get_strings().unwrap();
    assert_eq!(strings.len(), 3);
    assert_eq!(strings[0], "apple");
    assert_eq!(strings[1], "banana");
    assert_eq!(strings[2], "cherry");
}

/// Test MultiValues set operation using &[&str]
#[test]
fn test_multi_values_set_str_slice_ref() {
    let mut mv = MultiValues::Empty(DataType::String);

    // Set multiple values using &[&str]
    let str_slice: &[&str] = &["one", "two", "three"];
    mv.set(str_slice).unwrap();

    let strings = mv.get_strings().unwrap();
    assert_eq!(strings.len(), 3);
    assert_eq!(strings[0], "one");
    assert_eq!(strings[1], "two");
    assert_eq!(strings[2], "three");
}

/// Test MultiValues add operation using &str
#[test]
fn test_multi_values_add_single_str_ref() {
    let mut mv = MultiValues::String(vec!["existing".to_string()]);

    // Use &str Add single value
    mv.add("new_value").unwrap();

    let strings = mv.get_strings().unwrap();
    assert_eq!(strings.len(), 2);
    assert_eq!(strings[0], "existing");
    assert_eq!(strings[1], "new_value");
}

/// Test case where self is Empty in merge operation
#[test]
fn test_multi_values_merge_when_self_is_empty() {
    // When self is Empty, merge should absorb all values from the other side.
    let mut mv_empty = MultiValues::Empty(DataType::String);
    let mv_filled = MultiValues::String(vec!["test".to_string()]);

    mv_empty.merge(&mv_filled).unwrap();
    assert_eq!(mv_empty.get_strings().unwrap(), &["test"]);
    assert_eq!(mv_empty.count(), 1);

    // Empty merge Empty
    let mut mv1 = MultiValues::Empty(DataType::Int32);
    let mv2 = MultiValues::Empty(DataType::Int32);
    mv1.merge(&mv2).unwrap();
    assert!(mv1.is_empty());

    // Empty merge another Empty
    let mut mv3 = MultiValues::Empty(DataType::Bool);
    let mv4 = MultiValues::Empty(DataType::Bool);
    mv3.merge(&mv4).unwrap();
    assert_eq!(mv3.data_type(), DataType::Bool);
    assert!(mv3.is_empty());
}

#[test]
fn test_multi_values_to_value_takes_first_element() {
    let numbers = MultiValues::Int32(vec![10, 20, 30]);
    assert_eq!(numbers.to_value(), Value::Int32(10));

    let strings = MultiValues::String(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(strings.to_value(), Value::String("a".to_string()));
}

#[test]
fn test_multi_values_to_value_on_empty_preserves_type() {
    let empty_declared = MultiValues::Empty(DataType::UInt64);
    assert_eq!(empty_declared.to_value(), Value::Empty(DataType::UInt64));

    let empty_vec = MultiValues::Int32(vec![]);
    assert_eq!(empty_vec.to_value(), Value::Empty(DataType::Int32));
}

#[test]
fn test_multi_values_to_value_extended_types() {
    let uint_size = MultiValues::UIntSize(vec![12usize, 34]);
    assert_eq!(uint_size.to_value(), Value::UIntSize(12));

    let duration = MultiValues::Duration(vec![Duration::from_secs(30), Duration::from_secs(45)]);
    assert_eq!(
        duration.to_value(),
        Value::Duration(Duration::from_secs(30))
    );

    let url = Url::parse("https://example.com/query?a=1").unwrap();
    let other_url = Url::parse("https://rust-lang.org/").unwrap();
    let urls = MultiValues::Url(vec![url.clone(), other_url]);
    assert_eq!(urls.to_value(), Value::Url(url));

    let mut map = HashMap::new();
    map.insert("k1".to_string(), "v1".to_string());
    let map2 = map.clone();
    map.insert("k2".to_string(), "v2".to_string());
    let string_maps = MultiValues::StringMap(vec![map.clone(), map2]);
    assert_eq!(string_maps.to_value(), Value::StringMap(map));

    let json_value = JsonValue::Object(serde_json::json!({"a": 1}).as_object().unwrap().clone());
    let another_json = JsonValue::Bool(true);
    let jsons = MultiValues::Json(vec![json_value.clone(), another_json]);
    assert_eq!(jsons.to_value(), Value::Json(json_value));

    let empty_uint_size = MultiValues::UIntSize(vec![]);
    assert_eq!(empty_uint_size.to_value(), Value::Empty(DataType::UIntSize));

    let empty_duration = MultiValues::Duration(vec![]);
    assert_eq!(empty_duration.to_value(), Value::Empty(DataType::Duration));

    let empty_url = MultiValues::Url(vec![]);
    assert_eq!(empty_url.to_value(), Value::Empty(DataType::Url));

    let empty_string_map = MultiValues::StringMap(vec![]);
    assert_eq!(
        empty_string_map.to_value(),
        Value::Empty(DataType::StringMap)
    );

    let empty_json = MultiValues::Json(vec![]);
    assert_eq!(empty_json.to_value(), Value::Empty(DataType::Json));
}

#[test]
fn test_multi_values_to_value_all_variants() {
    assert_eq!(
        MultiValues::Bool(vec![true, false]).to_value(),
        Value::Bool(true)
    );
    assert_eq!(
        MultiValues::Char(vec!['a', 'b']).to_value(),
        Value::Char('a')
    );
    assert_eq!(MultiValues::Int8(vec![1, 2]).to_value(), Value::Int8(1));
    assert_eq!(
        MultiValues::Int16(vec![10, 20]).to_value(),
        Value::Int16(10)
    );
    assert_eq!(
        MultiValues::Int32(vec![10, 20]).to_value(),
        Value::Int32(10)
    );
    assert_eq!(
        MultiValues::Int64(vec![100, 200]).to_value(),
        Value::Int64(100)
    );
    assert_eq!(
        MultiValues::Int128(vec![1000, 2000]).to_value(),
        Value::Int128(1000)
    );
    assert_eq!(MultiValues::UInt8(vec![1, 2]).to_value(), Value::UInt8(1));
    assert_eq!(
        MultiValues::UInt16(vec![10, 20]).to_value(),
        Value::UInt16(10)
    );
    assert_eq!(
        MultiValues::UInt32(vec![100, 200]).to_value(),
        Value::UInt32(100)
    );
    assert_eq!(
        MultiValues::UInt64(vec![1000, 2000]).to_value(),
        Value::UInt64(1000)
    );
    assert_eq!(
        MultiValues::UInt128(vec![10000, 20000]).to_value(),
        Value::UInt128(10000)
    );
    assert_eq!(
        MultiValues::IntSize(vec![1isize, 2]).to_value(),
        Value::IntSize(1)
    );
    assert_eq!(
        MultiValues::UIntSize(vec![1usize, 2]).to_value(),
        Value::UIntSize(1)
    );
    assert_eq!(
        MultiValues::Float32(vec![1.5f32, 2.5f32]).to_value(),
        Value::Float32(1.5f32)
    );
    assert_eq!(
        MultiValues::Float64(vec![1.5f64, 2.5f64]).to_value(),
        Value::Float64(1.5f64)
    );

    let big_int = BigInt::from_str("123456789012345678901234567890").unwrap();
    assert_eq!(
        MultiValues::BigInteger(vec![big_int.clone(), BigInt::from(42)]).to_value(),
        Value::BigInteger(big_int)
    );

    let big_decimal = BigDecimal::from_str("12.25").unwrap();
    assert_eq!(
        MultiValues::BigDecimal(vec![big_decimal.clone(), BigDecimal::from(3)]).to_value(),
        Value::BigDecimal(big_decimal)
    );

    let date = NaiveDate::from_ymd_opt(2026, 4, 17).unwrap();
    assert_eq!(MultiValues::Date(vec![date]).to_value(), Value::Date(date));

    let time = NaiveTime::from_hms_opt(8, 30, 15).unwrap();
    assert_eq!(MultiValues::Time(vec![time]).to_value(), Value::Time(time));

    let datetime = NaiveDateTime::new(date, time);
    assert_eq!(
        MultiValues::DateTime(vec![datetime]).to_value(),
        Value::DateTime(datetime)
    );

    let instant = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
    assert_eq!(
        MultiValues::Instant(vec![instant]).to_value(),
        Value::Instant(instant)
    );
}

#[test]
fn test_multi_values_to_value_empty_for_all_variants() {
    assert_eq!(
        MultiValues::Bool(vec![]).to_value(),
        Value::Empty(DataType::Bool)
    );
    assert_eq!(
        MultiValues::Char(vec![]).to_value(),
        Value::Empty(DataType::Char)
    );
    assert_eq!(
        MultiValues::Int8(vec![]).to_value(),
        Value::Empty(DataType::Int8)
    );
    assert_eq!(
        MultiValues::Int16(vec![]).to_value(),
        Value::Empty(DataType::Int16)
    );
    assert_eq!(
        MultiValues::Int32(vec![]).to_value(),
        Value::Empty(DataType::Int32)
    );
    assert_eq!(
        MultiValues::Int64(vec![]).to_value(),
        Value::Empty(DataType::Int64)
    );
    assert_eq!(
        MultiValues::Int128(vec![]).to_value(),
        Value::Empty(DataType::Int128)
    );
    assert_eq!(
        MultiValues::UInt8(vec![]).to_value(),
        Value::Empty(DataType::UInt8)
    );
    assert_eq!(
        MultiValues::UInt16(vec![]).to_value(),
        Value::Empty(DataType::UInt16)
    );
    assert_eq!(
        MultiValues::UInt32(vec![]).to_value(),
        Value::Empty(DataType::UInt32)
    );
    assert_eq!(
        MultiValues::UInt64(vec![]).to_value(),
        Value::Empty(DataType::UInt64)
    );
    assert_eq!(
        MultiValues::UInt128(vec![]).to_value(),
        Value::Empty(DataType::UInt128)
    );
    assert_eq!(
        MultiValues::IntSize(vec![]).to_value(),
        Value::Empty(DataType::IntSize)
    );
    assert_eq!(
        MultiValues::UIntSize(vec![]).to_value(),
        Value::Empty(DataType::UIntSize)
    );
    assert_eq!(
        MultiValues::Float32(vec![]).to_value(),
        Value::Empty(DataType::Float32)
    );
    assert_eq!(
        MultiValues::Float64(vec![]).to_value(),
        Value::Empty(DataType::Float64)
    );
    assert_eq!(
        MultiValues::BigInteger(vec![]).to_value(),
        Value::Empty(DataType::BigInteger)
    );
    assert_eq!(
        MultiValues::BigDecimal(vec![]).to_value(),
        Value::Empty(DataType::BigDecimal)
    );

    assert_eq!(
        MultiValues::Date(vec![]).to_value(),
        Value::Empty(DataType::Date)
    );
    assert_eq!(
        MultiValues::Time(vec![]).to_value(),
        Value::Empty(DataType::Time)
    );
    assert_eq!(
        MultiValues::DateTime(vec![]).to_value(),
        Value::Empty(DataType::DateTime)
    );
    assert_eq!(
        MultiValues::Instant(vec![]).to_value(),
        Value::Empty(DataType::Instant)
    );
    assert_eq!(
        MultiValues::Duration(vec![]).to_value(),
        Value::Empty(DataType::Duration)
    );
}

#[test]
fn test_multi_values_set_retypes_existing_values() {
    let mut mv = MultiValues::Int32(vec![1, 2, 3]);
    mv.set(vec!["a".to_string(), "b".to_string()]).unwrap();
    assert_eq!(mv.data_type(), DataType::String);
    assert_eq!(mv.get_strings().unwrap(), &["a", "b"]);

    mv.set(true).unwrap();
    assert_eq!(mv.data_type(), DataType::Bool);
    assert_eq!(mv.get_bools().unwrap(), &[true]);
}
