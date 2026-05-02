/*****************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Accessors Unit Tests
//!
//! Tests for access-style getters/setters and raw type errors.
//!
//! # Author
//!
//! Haixing Hu

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use qubit_value::{
    Value,
    ValueError,
};
use std::str::FromStr;

#[test]
fn test_value_type_check() {
    let v = Value::Bool(true);
    assert!(v.get_bool().unwrap());
    assert!(matches!(
        v.get_int32(),
        Err(ValueError::TypeMismatch { .. })
    ));
}
#[test]
fn test_value_generic_get() {
    // Test type inference
    let v = Value::Int32(42);
    let num: i32 = v.get().unwrap();
    assert_eq!(num, 42);

    // Test explicit type parameters
    let v = Value::Int64(100);
    let num = v.get::<i64>().unwrap();
    assert_eq!(num, 100);

    // Test boolean values
    let v = Value::Bool(true);
    let b: bool = v.get().unwrap();
    assert!(b);

    // Test strings
    let v = Value::String("hello".to_string());
    let s: String = v.get().unwrap();
    assert_eq!(s, "hello");

    // Test floating point
    let v = Value::Float64(3.5);
    let f: f64 = v.get().unwrap();
    assert!((f - 3.5).abs() < 0.001);
}
#[test]
fn test_value_generic_get_type_mismatch() {
    let v = Value::Int32(42);
    let result: Result<bool, _> = v.get();
    assert!(result.is_err());
}
#[test]
fn test_value_generic_get_all_types() {
    // Test all basic types
    assert!(Value::Bool(true).get::<bool>().unwrap());
    assert_eq!(Value::Char('A').get::<char>().unwrap(), 'A');
    assert_eq!(Value::Int8(8).get::<i8>().unwrap(), 8);
    assert_eq!(Value::Int16(16).get::<i16>().unwrap(), 16);
    assert_eq!(Value::Int32(32).get::<i32>().unwrap(), 32);
    assert_eq!(Value::Int64(64).get::<i64>().unwrap(), 64);
    assert_eq!(Value::Int128(128).get::<i128>().unwrap(), 128);
    assert_eq!(Value::UInt8(8).get::<u8>().unwrap(), 8);
    assert_eq!(Value::UInt16(16).get::<u16>().unwrap(), 16);
    assert_eq!(Value::UInt32(32).get::<u32>().unwrap(), 32);
    assert_eq!(Value::UInt64(64).get::<u64>().unwrap(), 64);
    assert_eq!(Value::UInt128(128).get::<u128>().unwrap(), 128);
    assert!((Value::Float32(3.5).get::<f32>().unwrap() - 3.5).abs() < 0.001);
    assert!((Value::Float64(3.5).get::<f64>().unwrap() - 3.5).abs() < 0.001);
    assert_eq!(
        Value::String("test".to_string()).get::<String>().unwrap(),
        "test"
    );
}
#[test]
fn test_value_set_methods() {
    // Test set method for basic types
    let mut value = Value::Empty(DataType::Int32);
    value.set_int32(42).unwrap();
    assert_eq!(value.get_int32().unwrap(), 42);

    let mut value = Value::Empty(DataType::Bool);
    value.set_bool(true).unwrap();
    assert!(value.get_bool().unwrap());

    let mut value = Value::Empty(DataType::String);
    value.set_string("hello".to_string()).unwrap();
    assert_eq!(value.get_string().unwrap(), "hello");
}
#[test]
fn test_value_generic_set() {
    // Test generic set method
    let mut value = Value::Empty(DataType::Int32);
    value.set(42i32).unwrap();
    assert_eq!(value.get_int32().unwrap(), 42);

    let mut value = Value::Empty(DataType::String);
    value.set("hello".to_string()).unwrap();
    assert_eq!(value.get_string().unwrap(), "hello");

    let mut value = Value::Empty(DataType::Bool);
    value.set(true).unwrap();
    assert!(value.get_bool().unwrap());
}
#[test]
fn test_value_set_all_types() {
    // Test set method for all types
    let mut value = Value::Empty(DataType::Bool);
    value.set_bool(true).unwrap();
    assert!(value.get_bool().unwrap());

    let mut value = Value::Empty(DataType::Char);
    value.set_char('A').unwrap();
    assert_eq!(value.get_char().unwrap(), 'A');

    let mut value = Value::Empty(DataType::Int8);
    value.set_int8(42i8).unwrap();
    assert_eq!(value.get_int8().unwrap(), 42);

    let mut value = Value::Empty(DataType::Int16);
    value.set_int16(1000i16).unwrap();
    assert_eq!(value.get_int16().unwrap(), 1000);

    let mut value = Value::Empty(DataType::Int32);
    value.set_int32(100000i32).unwrap();
    assert_eq!(value.get_int32().unwrap(), 100000);

    let mut value = Value::Empty(DataType::Int64);
    value.set_int64(1000000000i64).unwrap();
    assert_eq!(value.get_int64().unwrap(), 1000000000);

    let mut value = Value::Empty(DataType::UInt8);
    value.set_uint8(255u8).unwrap();
    assert_eq!(value.get_uint8().unwrap(), 255);

    let mut value = Value::Empty(DataType::UInt16);
    value.set_uint16(65535u16).unwrap();
    assert_eq!(value.get_uint16().unwrap(), 65535);

    let mut value = Value::Empty(DataType::UInt32);
    value.set_uint32(4294967295u32).unwrap();
    assert_eq!(value.get_uint32().unwrap(), 4294967295);

    let mut value = Value::Empty(DataType::Float32);
    value.set_float32(3.5f32).unwrap();
    assert_eq!(value.get_float32().unwrap(), 3.5);

    let mut value = Value::Empty(DataType::Float64);
    value.set_float64(3.5f64).unwrap();
    assert_eq!(value.get_float64().unwrap(), 3.5);
}
#[test]
fn test_biginteger_value() {
    // Test BigInteger Value
    let big_int = BigInt::from_str("12345678901234567890").unwrap();
    let value = Value::BigInteger(big_int.clone());

    assert_eq!(value.data_type(), DataType::BigInteger);

    // Get value
    let retrieved = value.get_biginteger().unwrap();
    assert_eq!(retrieved, big_int);

    // Convert to string
    let string_repr = value.to::<String>().unwrap();
    assert_eq!(string_repr, "12345678901234567890");

    // Test setting value
    let mut value = Value::Empty(DataType::BigInteger);
    let new_big_int = BigInt::from_str("98765432109876543210").unwrap();
    value.set_biginteger(new_big_int.clone()).unwrap();
    assert_eq!(value.get_biginteger().unwrap(), new_big_int);

    // Test generic methods
    let mut value = Value::Empty(DataType::BigInteger);
    value.set(big_int.clone()).unwrap();
    let retrieved: BigInt = value.get().unwrap();
    assert_eq!(retrieved, big_int);
}
#[test]
fn test_bigdecimal_value() {
    // Test BigDecimal Value
    let big_decimal = BigDecimal::from_str("123.456789").unwrap();
    let value = Value::BigDecimal(big_decimal.clone());

    assert_eq!(value.data_type(), DataType::BigDecimal);

    // Get value
    let retrieved = value.get_bigdecimal().unwrap();
    assert_eq!(retrieved, big_decimal);

    // Convert to string
    let string_repr = value.to::<String>().unwrap();
    assert_eq!(string_repr, "123.456789");

    // Test setting value
    let mut value = Value::Empty(DataType::BigDecimal);
    let new_big_decimal = BigDecimal::from_str("987.654321").unwrap();
    value.set_bigdecimal(new_big_decimal.clone()).unwrap();
    assert_eq!(value.get_bigdecimal().unwrap(), new_big_decimal);

    // Test generic methods
    let mut value = Value::Empty(DataType::BigDecimal);
    value.set(big_decimal.clone()).unwrap();
    let retrieved: BigDecimal = value.get().unwrap();
    assert_eq!(retrieved, big_decimal);
}
#[test]
fn test_biginteger_bigdecimal_type_mismatch() {
    // Test type mismatch cases
    let big_int = BigInt::from_str("123456789").unwrap();
    let value = Value::BigInteger(big_int);

    // Attempting to get wrong type should fail
    assert!(matches!(
        value.get_bigdecimal(),
        Err(ValueError::TypeMismatch { .. })
    ));

    let big_decimal = BigDecimal::from_str("123.456").unwrap();
    let value = Value::BigDecimal(big_decimal);

    // Attempting to get wrong type should fail
    assert!(matches!(
        value.get_biginteger(),
        Err(ValueError::TypeMismatch { .. })
    ));
}
#[test]
fn test_value_set_all_integer_types() {
    // Test Int128
    let mut value = Value::Empty(DataType::Int128);
    value.set_int128(123456789012345678i128).unwrap();
    assert_eq!(value.get_int128().unwrap(), 123456789012345678i128);

    // Test UInt64
    let mut value = Value::Empty(DataType::UInt64);
    value.set_uint64(18446744073709551615u64).unwrap();
    assert_eq!(value.get_uint64().unwrap(), 18446744073709551615u64);

    // Test UInt128
    let mut value = Value::Empty(DataType::UInt128);
    value
        .set_uint128(340282366920938463463374607431768211455u128)
        .unwrap();
    assert_eq!(
        value.get_uint128().unwrap(),
        340282366920938463463374607431768211455u128
    );
}
#[test]
fn test_value_getter_empty_value_errors() {
    // Test Empty value get errors for all types
    assert!(matches!(
        Value::Empty(DataType::Bool).get_bool(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Char).get_char(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Int8).get_int8(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Int16).get_int16(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Int32).get_int32(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Int64).get_int64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Int128).get_int128(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::UInt8).get_uint8(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::UInt16).get_uint16(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::UInt32).get_uint32(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::UInt64).get_uint64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::UInt128).get_uint128(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Float32).get_float32(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Float64).get_float64(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Date).get_date(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Time).get_time(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::DateTime).get_datetime(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::Instant).get_instant(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::BigInteger).get_biginteger(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Empty(DataType::BigDecimal).get_bigdecimal(),
        Err(ValueError::NoValue)
    ));
}
#[test]
fn test_value_type_mismatch_errors() {
    // Test type mismatch errors
    let value = Value::Int32(42);

    assert!(matches!(
        value.get_bool(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        value.get_char(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        value.get_string(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        value.get_float64(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        value.get_date(),
        Err(ValueError::TypeMismatch { .. })
    ));
}
#[test]
fn test_value_setter_type_mismatch() {
    // Test all setters handling of Empty values
    let mut value = Value::Empty(DataType::String);
    value.set_bool(true).unwrap();
    assert!(value.get_bool().unwrap());

    let mut value = Value::Empty(DataType::Bool);
    value.set_char('X').unwrap();
    assert_eq!(value.get_char().unwrap(), 'X');

    let mut value = Value::Empty(DataType::Char);
    value.set_int8(10).unwrap();
    assert_eq!(value.get_int8().unwrap(), 10);

    let mut value = Value::Empty(DataType::Int8);
    value.set_int16(1000).unwrap();
    assert_eq!(value.get_int16().unwrap(), 1000);

    let mut value = Value::Empty(DataType::Int16);
    value.set_float32(3.5).unwrap();
    assert_eq!(value.get_float32().unwrap(), 3.5);

    let mut value = Value::Empty(DataType::Float32);
    value.set_float64(2.5).unwrap();
    assert_eq!(value.get_float64().unwrap(), 2.5);
}
#[test]
fn test_value_all_bigint_bigdecimal_getters() {
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;
    use std::str::FromStr;

    // Test creating BigInteger from String value
    let big_int = BigInt::from_str("999999999999999999999").unwrap();
    let value = Value::BigInteger(big_int.clone());
    assert_eq!(value.get_biginteger().unwrap(), big_int);
    assert_eq!(value.get_biginteger_ref().unwrap(), &big_int);

    // Test creating BigDecimal from String value
    let big_decimal = BigDecimal::from_str("123.456789012345").unwrap();
    let value = Value::BigDecimal(big_decimal.clone());
    assert_eq!(value.get_bigdecimal().unwrap(), big_decimal);
    assert_eq!(value.get_bigdecimal_ref().unwrap(), &big_decimal);

    // Test BigInteger to string conversion
    let big_int = BigInt::from_str("987654321098765432109876543210").unwrap();
    let value = Value::BigInteger(big_int);
    let str_repr = value.to::<String>().unwrap();
    assert_eq!(str_repr, "987654321098765432109876543210");

    // Test BigDecimal to string conversion
    let big_decimal = BigDecimal::from_str("999.888777666").unwrap();
    let value = Value::BigDecimal(big_decimal);
    let str_repr = value.to::<String>().unwrap();
    assert!(str_repr.contains("999"));
}
#[test]
fn test_value_borrowing_getters_for_non_copy_types() {
    use serde_json::json;
    use std::collections::HashMap;
    use url::Url;

    let url = Url::parse("https://example.com").unwrap();
    let url_value = Value::Url(url.clone());
    assert_eq!(url_value.get_url_ref().unwrap(), &url);

    let mut map = HashMap::new();
    map.insert("k".to_string(), "v".to_string());
    let map_value = Value::StringMap(map.clone());
    assert_eq!(map_value.get_string_map_ref().unwrap(), &map);

    let json_value = json!({"k": "v"});
    let value = Value::Json(json_value.clone());
    assert_eq!(value.get_json_ref().unwrap(), &json_value);
}
#[test]
fn test_value_borrowing_getters_error_branches() {
    let empty_json = Value::Empty(DataType::Json);
    assert!(matches!(
        empty_json.get_json_ref(),
        Err(ValueError::NoValue)
    ));

    let empty_biginteger = Value::Empty(DataType::BigInteger);
    assert!(matches!(
        empty_biginteger.get_biginteger_ref(),
        Err(ValueError::NoValue)
    ));

    let empty_bigdecimal = Value::Empty(DataType::BigDecimal);
    assert!(matches!(
        empty_bigdecimal.get_bigdecimal_ref(),
        Err(ValueError::NoValue)
    ));

    let empty_url = Value::Empty(DataType::Url);
    assert!(matches!(empty_url.get_url_ref(), Err(ValueError::NoValue)));

    let empty_string_map = Value::Empty(DataType::StringMap);
    assert!(matches!(
        empty_string_map.get_string_map_ref(),
        Err(ValueError::NoValue)
    ));

    let wrong_type = Value::Bool(true);
    assert!(matches!(
        wrong_type.get_url_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_biginteger_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_bigdecimal_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_string_map_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));

    let wrong_type = Value::Int32(1);
    assert!(matches!(
        wrong_type.get_string_map_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_json_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_url_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        wrong_type.get_biginteger_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));

    let wrong_json_type = Value::String("42".to_string());
    assert!(matches!(
        wrong_json_type.get_json_ref(),
        Err(ValueError::TypeMismatch { .. })
    ));
}
#[test]
fn test_generic_set_for_coverage() {
    use chrono::NaiveDate;

    let mut v = Value::new('a');
    v.set('b').unwrap();
    assert_eq!(v.get_char().unwrap(), 'b');

    let mut v = Value::new(123u128);
    v.set(456u128).unwrap();
    assert_eq!(v.get_uint128().unwrap(), 456u128);

    let mut v = Value::new(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    v.set(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()).unwrap();
    assert_eq!(
        v.get_date().unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
    );
}
