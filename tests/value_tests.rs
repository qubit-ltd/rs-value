/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Unit Tests
//!
//! Tests various functionalities of the single value container。
//!
//! # Author
//!
//! Haixing Hu

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use qubit_value::{Value, ValueError};
use std::str::FromStr;

#[test]
fn test_value_creation() {
    let v = Value::Int32(42);
    assert_eq!(v.data_type(), DataType::Int32);
    assert!(!v.is_empty());
    assert_eq!(v.get_int32().unwrap(), 42);
}

#[test]
fn test_value_empty() {
    let v = Value::Empty(DataType::String);
    assert_eq!(v.data_type(), DataType::String);
    assert!(v.is_empty());
    assert!(matches!(v.get_string(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_clear() {
    let mut v = Value::Int32(42);
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::Int32);
}

#[test]
fn test_value_set_type() {
    let mut v = Value::Int32(42);
    v.set_type(DataType::String);
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::String);
}

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
fn test_value_type_conversion() {
    let v = Value::Int32(42);
    assert_eq!(v.as_int64().unwrap(), 42i64);
    assert_eq!(v.as_float64().unwrap(), 42.0f64);
    assert_eq!(v.as_string().unwrap(), "42");
}

#[test]
fn test_value_bool_conversion() {
    let v1 = Value::Int32(1);
    assert!(v1.as_bool().unwrap());

    let v2 = Value::Int32(0);
    assert!(!v2.as_bool().unwrap());

    let v3 = Value::String("true".to_string());
    assert!(v3.as_bool().unwrap());
}

#[test]
fn test_value_string_types() {
    let v = Value::String("hello".to_string());
    assert_eq!(v.get_string().unwrap(), "hello");
    assert_eq!(v.data_type(), DataType::String);
}

#[test]
fn test_value_numeric_types() {
    let v1 = Value::Int8(127);
    assert_eq!(v1.get_int8().unwrap(), 127);

    let v2 = Value::UInt32(12345);
    assert_eq!(v2.get_uint32().unwrap(), 12345);

    let v3 = Value::Float32(3.5);
    assert!((v3.get_float32().unwrap() - 3.5).abs() < 0.001);
}

#[test]
fn test_value_default() {
    let v: Value = Default::default();
    assert_eq!(v.data_type(), DataType::String);
    assert!(v.is_empty());
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
fn test_value_new() {
    // Test generic new() method
    let v = Value::new(42i32);
    assert_eq!(v.get_int32().unwrap(), 42);

    let v = Value::new(true);
    assert!(v.get_bool().unwrap());

    let v = Value::new("hello".to_string());
    assert_eq!(v.get_string().unwrap(), "hello");
}

#[test]
fn test_value_new_str() {
    // Test creation with &str
    let v = Value::new("hello");
    assert_eq!(v.get_string().unwrap(), "hello");

    let s: String = v.get().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn test_value_new_various_types() {
    // Test new() support for various types

    // Basic types
    assert!(Value::new(true).get_bool().unwrap());
    assert_eq!(Value::new('A').get_char().unwrap(), 'A');

    // Integers
    assert_eq!(Value::new(42i32).get_int32().unwrap(), 42);
    assert_eq!(Value::new(100u64).get_uint64().unwrap(), 100);

    // Floating point
    assert!((Value::new(3.5f32).get_float32().unwrap() - 3.5).abs() < 0.001);
    assert!((Value::new(2.5f64).get_float64().unwrap() - 2.5).abs() < 0.001);

    // Strings (String vs &str)
    assert_eq!(
        Value::new("hello".to_string()).get_string().unwrap(),
        "hello"
    );
    assert_eq!(Value::new("world").get_string().unwrap(), "world");
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
fn test_value_ref_types() {
    // Test generic methods for &str type
    let mut value = Value::Empty(DataType::String);
    value.set("hello").unwrap();
    assert_eq!(value.get_string().unwrap(), "hello");

    // Test creating Value from &str
    let value = Value::new("world");
    assert_eq!(value.get_string().unwrap(), "world");
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
    let string_repr = value.as_string().unwrap();
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
    let string_repr = value.as_string().unwrap();
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

// ========================================================================
// Tests to increase coverage
// ========================================================================

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
fn test_value_datetime_types() {
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Test Date
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let mut value = Value::Empty(DataType::Date);
    value.set_date(date).unwrap();
    assert_eq!(value.get_date().unwrap(), date);
    assert_eq!(value.data_type(), DataType::Date);

    // Test Time
    let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    let mut value = Value::Empty(DataType::Time);
    value.set_time(time).unwrap();
    assert_eq!(value.get_time().unwrap(), time);
    assert_eq!(value.data_type(), DataType::Time);

    // Test DateTime
    let datetime = NaiveDate::from_ymd_opt(2024, 1, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let mut value = Value::Empty(DataType::DateTime);
    value.set_datetime(datetime).unwrap();
    assert_eq!(value.get_datetime().unwrap(), datetime);
    assert_eq!(value.data_type(), DataType::DateTime);

    // Test Instant
    let instant = Utc::now();
    let mut value = Value::Empty(DataType::Instant);
    value.set_instant(instant).unwrap();
    assert_eq!(value.get_instant().unwrap(), instant);
    assert_eq!(value.data_type(), DataType::Instant);
}

#[test]
fn test_value_datetime_to_string() {
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Test Date to string conversion
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let value = Value::Date(date);
    let str_repr = value.as_string().unwrap();
    assert_eq!(str_repr, "2024-01-15");

    // Test Time to string conversion
    let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    let value = Value::Time(time);
    let str_repr = value.as_string().unwrap();
    assert_eq!(str_repr, "14:30:45");

    // Test DateTime to string conversion
    let datetime = NaiveDate::from_ymd_opt(2024, 1, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let value = Value::DateTime(datetime);
    let str_repr = value.as_string().unwrap();
    assert_eq!(str_repr, "2024-01-15 14:30:45");

    // Test Instant to string conversion
    let instant = Utc::now();
    let value = Value::Instant(instant);
    let str_repr = value.as_string().unwrap();
    assert!(str_repr.contains('T')); // RFC3339 format contains 'T'
}

#[test]
fn test_value_as_bool_all_branches() {
    // Test all integer types to boolean conversion
    assert!(Value::Int8(1).as_bool().unwrap());
    assert!(!Value::Int8(0).as_bool().unwrap());

    assert!(Value::Int16(1).as_bool().unwrap());
    assert!(!Value::Int16(0).as_bool().unwrap());

    assert!(Value::Int64(1).as_bool().unwrap());
    assert!(!Value::Int64(0).as_bool().unwrap());

    assert!(Value::Int128(1).as_bool().unwrap());
    assert!(!Value::Int128(0).as_bool().unwrap());

    assert!(Value::UInt8(1).as_bool().unwrap());
    assert!(!Value::UInt8(0).as_bool().unwrap());

    assert!(Value::UInt16(1).as_bool().unwrap());
    assert!(!Value::UInt16(0).as_bool().unwrap());

    assert!(Value::UInt32(1).as_bool().unwrap());
    assert!(!Value::UInt32(0).as_bool().unwrap());

    assert!(Value::UInt64(1).as_bool().unwrap());
    assert!(!Value::UInt64(0).as_bool().unwrap());

    assert!(Value::UInt128(1).as_bool().unwrap());
    assert!(!Value::UInt128(0).as_bool().unwrap());

    // Test string to boolean conversion failure cases
    let value = Value::String("invalid".to_string());
    assert!(value.as_bool().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Bool);
    assert!(matches!(value.as_bool(), Err(ValueError::NoValue)));

    // Test unsupported type conversions
    let value = Value::Char('a');
    assert!(matches!(
        value.as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));
}

#[test]
fn test_value_as_int32_all_branches() {
    // Test all types that can convert to i32
    assert_eq!(Value::Int8(42).as_int32().unwrap(), 42);
    assert_eq!(Value::Int16(1000).as_int32().unwrap(), 1000);
    assert_eq!(Value::Int32(100000).as_int32().unwrap(), 100000);

    // Test i64 to i32 conversion success
    assert_eq!(Value::Int64(42).as_int32().unwrap(), 42);

    // Test i64 to i32 overflow
    let value = Value::Int64(i64::MAX);
    assert!(value.as_int32().is_err());

    // Test i128 to i32 conversion success
    assert_eq!(Value::Int128(42).as_int32().unwrap(), 42);

    // Test i128 to i32 overflow
    let value = Value::Int128(i128::MAX);
    assert!(value.as_int32().is_err());

    // Test unsigned integer conversion
    assert_eq!(Value::UInt8(42).as_int32().unwrap(), 42);
    assert_eq!(Value::UInt16(1000).as_int32().unwrap(), 1000);

    // Test u32 to i32 conversion success
    assert_eq!(Value::UInt32(42).as_int32().unwrap(), 42);

    // Test u32 to i32 overflow
    let value = Value::UInt32(u32::MAX);
    assert!(value.as_int32().is_err());

    // Test string to i32 conversion
    assert_eq!(
        Value::String("12345".to_string()).as_int32().unwrap(),
        12345
    );

    // Test string to i32 conversion failure
    let value = Value::String("invalid".to_string());
    assert!(value.as_int32().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Int32);
    assert!(matches!(value.as_int32(), Err(ValueError::NoValue)));

    // Test Bool to i32 conversion
    let value = Value::Bool(true);
    assert_eq!(value.as_int32().unwrap(), 1);
    let value = Value::Bool(false);
    assert_eq!(value.as_int32().unwrap(), 0);

    // Test Char to i32 conversion
    let value = Value::Char('A');
    assert_eq!(value.as_int32().unwrap(), 65);

    // Test Float32/Float64 to i32 conversion
    let value = Value::Float32(42.7);
    assert_eq!(value.as_int32().unwrap(), 42);
    let value = Value::Float64(99.9);
    assert_eq!(value.as_int32().unwrap(), 99);

    // Test BigDecimal to i32 conversion
    let value = Value::BigDecimal(BigDecimal::from(123));
    assert_eq!(value.as_int32().unwrap(), 123);

    // Test unsupported time types
    let value = Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionFailed { .. })
    ));
}

#[test]
fn test_value_as_int64_all_branches() {
    // Test all types that can convert to i64
    assert_eq!(Value::Int8(42).as_int64().unwrap(), 42);
    assert_eq!(Value::Int16(1000).as_int64().unwrap(), 1000);
    assert_eq!(Value::Int32(100000).as_int64().unwrap(), 100000);
    assert_eq!(Value::Int64(1000000).as_int64().unwrap(), 1000000);

    // Test i128 to i64 conversion success
    assert_eq!(Value::Int128(42).as_int64().unwrap(), 42);

    // Test i128 to i64 overflow
    let value = Value::Int128(i128::MAX);
    assert!(value.as_int64().is_err());

    // Test unsigned integer conversion
    assert_eq!(Value::UInt8(42).as_int64().unwrap(), 42);
    assert_eq!(Value::UInt16(1000).as_int64().unwrap(), 1000);
    assert_eq!(Value::UInt32(100000).as_int64().unwrap(), 100000);

    // Test u64 to i64 conversion success
    assert_eq!(Value::UInt64(42).as_int64().unwrap(), 42);

    // Test u64 to i64 overflow
    let value = Value::UInt64(u64::MAX);
    assert!(value.as_int64().is_err());

    // Test string to i64 conversion
    assert_eq!(
        Value::String("123456789".to_string()).as_int64().unwrap(),
        123456789
    );

    // Test Empty value
    let value = Value::Empty(DataType::Int64);
    assert!(matches!(value.as_int64(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_as_float64_all_branches() {
    // Test floating point conversion
    assert_eq!(Value::Float32(3.5).as_float64().unwrap(), 3.5f32 as f64);
    assert_eq!(Value::Float64(2.5).as_float64().unwrap(), 2.5);

    // Test integers to floating point conversion
    assert_eq!(Value::Int8(42).as_float64().unwrap(), 42.0);
    assert_eq!(Value::Int16(1000).as_float64().unwrap(), 1000.0);
    assert_eq!(Value::Int32(100000).as_float64().unwrap(), 100000.0);
    assert_eq!(Value::Int64(1000000).as_float64().unwrap(), 1000000.0);

    // Test unsigned integers to floating point conversion
    assert_eq!(Value::UInt8(42).as_float64().unwrap(), 42.0);
    assert_eq!(Value::UInt16(1000).as_float64().unwrap(), 1000.0);
    assert_eq!(Value::UInt32(100000).as_float64().unwrap(), 100000.0);
    assert_eq!(Value::UInt64(1000000).as_float64().unwrap(), 1000000.0);

    // Test string to floating point conversion
    assert_eq!(Value::String("3.5".to_string()).as_float64().unwrap(), 3.5);

    // Test string to floating point conversion failure
    let value = Value::String("invalid".to_string());
    assert!(value.as_float64().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Float64);
    assert!(matches!(value.as_float64(), Err(ValueError::NoValue)));

    // Test Bool to f64 conversion
    let value = Value::Bool(true);
    assert_eq!(value.as_float64().unwrap(), 1.0);
    let value = Value::Bool(false);
    assert_eq!(value.as_float64().unwrap(), 0.0);

    // Test Char to f64 conversion
    let value = Value::Char('A');
    assert_eq!(value.as_float64().unwrap(), 65.0);

    // Test Int128/UInt128 to f64 conversion
    let value = Value::Int128(123);
    assert_eq!(value.as_float64().unwrap(), 123.0);
    let value = Value::UInt128(456);
    assert_eq!(value.as_float64().unwrap(), 456.0);

    // Test unsupported time types
    let value = Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(matches!(
        value.as_float64(),
        Err(ValueError::ConversionFailed { .. })
    ));
}

#[test]
fn test_value_as_string_all_types() {
    // Test all types to string conversion
    assert_eq!(Value::Bool(true).as_string().unwrap(), "true");
    assert_eq!(Value::Bool(false).as_string().unwrap(), "false");
    assert_eq!(Value::Char('A').as_string().unwrap(), "A");

    assert_eq!(Value::Int8(42).as_string().unwrap(), "42");
    assert_eq!(Value::Int16(1000).as_string().unwrap(), "1000");
    assert_eq!(Value::Int32(100000).as_string().unwrap(), "100000");
    assert_eq!(Value::Int64(1000000).as_string().unwrap(), "1000000");
    assert_eq!(Value::Int128(123456789).as_string().unwrap(), "123456789");

    assert_eq!(Value::UInt8(42).as_string().unwrap(), "42");
    assert_eq!(Value::UInt16(1000).as_string().unwrap(), "1000");
    assert_eq!(Value::UInt32(100000).as_string().unwrap(), "100000");
    assert_eq!(Value::UInt64(1000000).as_string().unwrap(), "1000000");
    assert_eq!(Value::UInt128(123456789).as_string().unwrap(), "123456789");

    assert!(Value::Float32(3.5).as_string().unwrap().starts_with("3.5"));
    assert!(Value::Float64(2.5).as_string().unwrap().starts_with("2.5"));

    assert_eq!(
        Value::String("hello".to_string()).as_string().unwrap(),
        "hello"
    );

    // Test Empty value
    let value = Value::Empty(DataType::String);
    assert!(matches!(value.as_string(), Err(ValueError::NoValue)));
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
fn test_value_set_type_same_type() {
    // Test setting same type does not clear value
    let mut value = Value::Int32(42);
    value.set_type(DataType::Int32);
    assert_eq!(value.get_int32().unwrap(), 42);
}

#[test]
fn test_value_as_int32_uint64_conversion() {
    // Test UInt64 to i32 - small values can convert successfully
    let value = Value::UInt64(100);
    assert_eq!(value.as_int32().unwrap(), 100);

    // Test UInt64 to i32 - large values will fail
    let value = Value::UInt64(u64::MAX);
    assert!(value.as_int32().is_err());

    // Test UInt128 to i32 - small values can convert successfully
    let value = Value::UInt128(200);
    assert_eq!(value.as_int32().unwrap(), 200);

    // Test UInt128 to i32 - large values will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.as_int32().is_err());
}

#[test]
fn test_value_as_int64_uint128_conversion() {
    // Test UInt128 to i64 - small values can convert successfully
    let value = Value::UInt128(1000);
    assert_eq!(value.as_int64().unwrap(), 1000);

    // Test UInt128 to i64 - large values will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.as_int64().is_err());

    // Test string to i64 conversion failure
    let value = Value::String("not a number".to_string());
    assert!(value.as_int64().is_err());
}

#[test]
fn test_value_as_float64_conversions() {
    // Test Int128 to f64 - now supports conversion
    let value = Value::Int128(999999);
    assert_eq!(value.as_float64().unwrap(), 999999.0);

    // Test UInt128 to f64 - now supports conversion
    let value = Value::UInt128(123456);
    assert_eq!(value.as_float64().unwrap(), 123456.0);

    // Test Bool to f64 conversion
    let value = Value::Bool(true);
    assert_eq!(value.as_float64().unwrap(), 1.0);
    let value = Value::Bool(false);
    assert_eq!(value.as_float64().unwrap(), 0.0);

    // Test Char to f64 conversion
    let value = Value::Char('B');
    assert_eq!(value.as_float64().unwrap(), 66.0);

    // Test string to f64 conversion failure
    let value = Value::String("invalid number".to_string());
    assert!(value.as_float64().is_err());
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

    // Test creating BigDecimal from String value
    let big_decimal = BigDecimal::from_str("123.456789012345").unwrap();
    let value = Value::BigDecimal(big_decimal.clone());
    assert_eq!(value.get_bigdecimal().unwrap(), big_decimal);

    // Test BigInteger to string conversion
    let big_int = BigInt::from_str("987654321098765432109876543210").unwrap();
    let value = Value::BigInteger(big_int);
    let str_repr = value.as_string().unwrap();
    assert_eq!(str_repr, "987654321098765432109876543210");

    // Test BigDecimal to string conversion
    let big_decimal = BigDecimal::from_str("999.888777666").unwrap();
    let value = Value::BigDecimal(big_decimal);
    let str_repr = value.as_string().unwrap();
    assert!(str_repr.contains("999"));
}

#[test]
fn test_big_type_conversions_for_coverage() {
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;
    use std::f64;
    use std::str::FromStr;

    // BigInt -> as_int32
    let v = Value::BigInteger(BigInt::from(123));
    assert_eq!(v.as_int32().unwrap(), 123);
    let v_overflow = Value::BigInteger(BigInt::from(i64::MAX));
    assert!(v_overflow.as_int32().is_err());

    // BigInt -> as_int64
    let v = Value::BigInteger(BigInt::from(123456i64));
    assert_eq!(v.as_int64().unwrap(), 123456i64);
    let v_overflow = Value::BigInteger(BigInt::from_str("123456789012345678901234567890").unwrap());
    assert!(v_overflow.as_int64().is_err());

    // BigInt -> as_float64
    let v = Value::BigInteger(BigInt::from(12345));
    assert!((v.as_float64().unwrap() - 12345.0).abs() < f64::EPSILON);
    let large_big_int_str = "1".repeat(400);
    let v_overflow = Value::BigInteger(BigInt::from_str(&large_big_int_str).unwrap());
    assert_eq!(v_overflow.as_float64().unwrap(), f64::INFINITY);

    // BigDecimal -> as_float64
    let v = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((v.as_float64().unwrap() - 123.456).abs() < f64::EPSILON);
    let v_overflow = Value::BigDecimal(BigDecimal::from_str("1.0e400").unwrap());
    assert_eq!(v_overflow.as_float64().unwrap(), f64::INFINITY);
}

#[test]
fn test_set_on_non_empty_for_coverage() {
    let mut v = Value::Int32(42);
    assert!(!v.is_empty());

    // Overwrite with a different type
    v.set_string("hello".to_string()).unwrap();
    assert_eq!(v.data_type(), DataType::String);
    assert!(!v.is_empty());
    assert_eq!(v.get_string().unwrap(), "hello");
    assert!(matches!(
        v.get_int32(),
        Err(ValueError::TypeMismatch { .. })
    ));

    // Overwrite with the same type
    v.set_string("world".to_string()).unwrap();
    assert_eq!(v.get_string().unwrap(), "world");
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

// ========================================================================
// Supplementary tests: increase coverage
// ========================================================================

#[test]
fn test_data_type_coverage_all_variants() {
    // Test data_type() method coverage for all data type variants
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Empty type (all possible DataType)
    assert_eq!(Value::Empty(DataType::Bool).data_type(), DataType::Bool);
    assert_eq!(Value::Empty(DataType::Char).data_type(), DataType::Char);
    assert_eq!(Value::Empty(DataType::Int8).data_type(), DataType::Int8);
    assert_eq!(Value::Empty(DataType::Int16).data_type(), DataType::Int16);
    assert_eq!(Value::Empty(DataType::Int32).data_type(), DataType::Int32);
    assert_eq!(Value::Empty(DataType::Int64).data_type(), DataType::Int64);
    assert_eq!(Value::Empty(DataType::Int128).data_type(), DataType::Int128);
    assert_eq!(Value::Empty(DataType::UInt8).data_type(), DataType::UInt8);
    assert_eq!(Value::Empty(DataType::UInt16).data_type(), DataType::UInt16);
    assert_eq!(Value::Empty(DataType::UInt32).data_type(), DataType::UInt32);
    assert_eq!(Value::Empty(DataType::UInt64).data_type(), DataType::UInt64);
    assert_eq!(
        Value::Empty(DataType::UInt128).data_type(),
        DataType::UInt128
    );
    assert_eq!(
        Value::Empty(DataType::Float32).data_type(),
        DataType::Float32
    );
    assert_eq!(
        Value::Empty(DataType::Float64).data_type(),
        DataType::Float64
    );
    assert_eq!(Value::Empty(DataType::String).data_type(), DataType::String);
    assert_eq!(Value::Empty(DataType::Date).data_type(), DataType::Date);
    assert_eq!(Value::Empty(DataType::Time).data_type(), DataType::Time);
    assert_eq!(
        Value::Empty(DataType::DateTime).data_type(),
        DataType::DateTime
    );
    assert_eq!(
        Value::Empty(DataType::Instant).data_type(),
        DataType::Instant
    );
    assert_eq!(
        Value::Empty(DataType::BigInteger).data_type(),
        DataType::BigInteger
    );
    assert_eq!(
        Value::Empty(DataType::BigDecimal).data_type(),
        DataType::BigDecimal
    );

    // All concrete value types
    assert_eq!(Value::Bool(true).data_type(), DataType::Bool);
    assert_eq!(Value::Char('A').data_type(), DataType::Char);
    assert_eq!(Value::Int8(1).data_type(), DataType::Int8);
    assert_eq!(Value::Int16(1).data_type(), DataType::Int16);
    assert_eq!(Value::Int32(1).data_type(), DataType::Int32);
    assert_eq!(Value::Int64(1).data_type(), DataType::Int64);
    assert_eq!(Value::Int128(1).data_type(), DataType::Int128);
    assert_eq!(Value::UInt8(1).data_type(), DataType::UInt8);
    assert_eq!(Value::UInt16(1).data_type(), DataType::UInt16);
    assert_eq!(Value::UInt32(1).data_type(), DataType::UInt32);
    assert_eq!(Value::UInt64(1).data_type(), DataType::UInt64);
    assert_eq!(Value::UInt128(1).data_type(), DataType::UInt128);
    assert_eq!(Value::Float32(1.0).data_type(), DataType::Float32);
    assert_eq!(Value::Float64(1.0).data_type(), DataType::Float64);
    assert_eq!(
        Value::String("test".to_string()).data_type(),
        DataType::String
    );
    assert_eq!(
        Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()).data_type(),
        DataType::Date
    );
    assert_eq!(
        Value::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).data_type(),
        DataType::Time
    );
    assert_eq!(
        Value::DateTime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        )
        .data_type(),
        DataType::DateTime
    );
    assert_eq!(Value::Instant(Utc::now()).data_type(), DataType::Instant);
    assert_eq!(
        Value::BigInteger(BigInt::from(123)).data_type(),
        DataType::BigInteger
    );
    assert_eq!(
        Value::BigDecimal(BigDecimal::from_str("123.45").unwrap()).data_type(),
        DataType::BigDecimal
    );
}

#[test]
fn test_is_empty_coverage_all_types() {
    // Test is_empty() returns false for all non-Empty types
    use chrono::{NaiveDate, NaiveTime, Utc};

    assert!(!Value::Bool(true).is_empty());
    assert!(!Value::Char('A').is_empty());
    assert!(!Value::Int8(1).is_empty());
    assert!(!Value::Int16(1).is_empty());
    assert!(!Value::Int32(1).is_empty());
    assert!(!Value::Int64(1).is_empty());
    assert!(!Value::Int128(1).is_empty());
    assert!(!Value::UInt8(1).is_empty());
    assert!(!Value::UInt16(1).is_empty());
    assert!(!Value::UInt32(1).is_empty());
    assert!(!Value::UInt64(1).is_empty());
    assert!(!Value::UInt128(1).is_empty());
    assert!(!Value::Float32(1.0).is_empty());
    assert!(!Value::Float64(1.0).is_empty());
    assert!(!Value::String("test".to_string()).is_empty());
    assert!(!Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()).is_empty());
    assert!(!Value::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).is_empty());
    assert!(!Value::DateTime(
        NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    )
    .is_empty());
    assert!(!Value::Instant(Utc::now()).is_empty());
    assert!(!Value::BigInteger(BigInt::from(123)).is_empty());
    assert!(!Value::BigDecimal(BigDecimal::from_str("123.45").unwrap()).is_empty());

    // Test all Empty types return true
    assert!(Value::Empty(DataType::Bool).is_empty());
    assert!(Value::Empty(DataType::Char).is_empty());
    assert!(Value::Empty(DataType::Int8).is_empty());
    assert!(Value::Empty(DataType::Int16).is_empty());
    assert!(Value::Empty(DataType::Int32).is_empty());
    assert!(Value::Empty(DataType::Int64).is_empty());
    assert!(Value::Empty(DataType::Int128).is_empty());
    assert!(Value::Empty(DataType::UInt8).is_empty());
    assert!(Value::Empty(DataType::UInt16).is_empty());
    assert!(Value::Empty(DataType::UInt32).is_empty());
    assert!(Value::Empty(DataType::UInt64).is_empty());
    assert!(Value::Empty(DataType::UInt128).is_empty());
    assert!(Value::Empty(DataType::Float32).is_empty());
    assert!(Value::Empty(DataType::Float64).is_empty());
    assert!(Value::Empty(DataType::String).is_empty());
    assert!(Value::Empty(DataType::Date).is_empty());
    assert!(Value::Empty(DataType::Time).is_empty());
    assert!(Value::Empty(DataType::DateTime).is_empty());
    assert!(Value::Empty(DataType::Instant).is_empty());
    assert!(Value::Empty(DataType::BigInteger).is_empty());
    assert!(Value::Empty(DataType::BigDecimal).is_empty());
}

#[test]
fn test_as_bool_string_conversion_error() {
    // Test string to boolean conversion failure returns ConversionError
    let value = Value::String("not_a_bool".to_string());
    match value.as_bool() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_bool"));
            assert!(msg.contains("to boolean"));
        }
        _ => panic!("Expected ConversionError"),
    }

    // Test empty string to boolean conversion
    let value = Value::String("".to_string());
    assert!(matches!(
        value.as_bool(),
        Err(ValueError::ConversionError(_))
    ));

    // Test various invalid boolean strings
    let invalid_bools = vec!["yes", "no", "1", "0", "True", "False", "TRUE", "FALSE"];
    for invalid in invalid_bools {
        let value = Value::String(invalid.to_string());
        assert!(matches!(
            value.as_bool(),
            Err(ValueError::ConversionError(_))
        ));
    }
}

#[test]
fn test_as_int32_conversion_errors() {
    // Test i64 out of range ConversionError
    let value = Value::Int64(i64::MAX);
    match value.as_int32() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i64"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i64 overflow"),
    }

    // Test i128 out of range ConversionError
    let value = Value::Int128(i128::MAX);
    match value.as_int32() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test u32 out of range ConversionError
    let value = Value::UInt32(u32::MAX);
    match value.as_int32() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u32"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u32 overflow"),
    }

    // Test string conversion failure ConversionError
    let value = Value::String("not_a_number".to_string());
    match value.as_int32() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_number"));
            assert!(msg.contains("i32"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Test BigInteger out of range ConversionError
    let value = Value::BigInteger(BigInt::from_str("999999999999999999999").unwrap());
    match value.as_int32() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigInteger"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for BigInteger overflow"),
    }
}

#[test]
fn test_as_int64_conversion_errors() {
    // Test i128 out of range ConversionError
    let value = Value::Int128(i128::MAX);
    match value.as_int64() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test u64 out of range ConversionError
    let value = Value::UInt64(u64::MAX);
    match value.as_int64() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 overflow"),
    }

    // Test string conversion failure ConversionError
    let value = Value::String("invalid_number".to_string());
    match value.as_int64() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("invalid_number"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Test BigInteger out of range ConversionError
    let value = Value::BigInteger(BigInt::from_str("999999999999999999999999999999").unwrap());
    match value.as_int64() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigInteger"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for BigInteger overflow"),
    }
}

#[test]
fn test_as_float64_conversion_errors() {
    // Test string conversion failure ConversionError
    let value = Value::String("not_a_float".to_string());
    match value.as_float64() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_float"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Test BigInteger conversion failure ConversionError (although it actually returns INFINITY)
    // Testing a scenario that may cause conversion failure
    // Note: to_f64() returns None or INFINITY for very large values
    // We need to find a case that returns None

    // Test BigDecimal conversion failure ConversionError
    // Similarly, to_f64() may return None
}

#[test]
fn test_as_int32_negative_i64_conversion() {
    // Test negative i64 out of i32 range
    let value = Value::Int64(i64::MIN);
    match value.as_int32() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i64 overflow"),
    }
}

#[test]
fn test_as_int32_negative_i128_conversion() {
    // Test negative i128 out of i32 range
    let value = Value::Int128(i128::MIN);
    match value.as_int32() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i128 overflow"),
    }
}

#[test]
fn test_as_int64_negative_i128_conversion() {
    // Test negative i128 out of i64 range
    let value = Value::Int128(i128::MIN);
    match value.as_int64() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i128 overflow"),
    }
}

#[test]
fn test_as_int32_small_uint64_success() {
    // Test UInt64 small values can convert successfully to i32
    let value = Value::UInt64(100);
    assert_eq!(value.as_int32().unwrap(), 100);

    // Test UInt64 large values conversion to i32 will fail
    let value = Value::UInt64(u64::MAX);
    assert!(value.as_int32().is_err());
}

#[test]
fn test_as_int64_small_uint128_conversion_failed() {
    // Test UInt128 small values can convert successfully to i64
    let value = Value::UInt128(100);
    assert_eq!(value.as_int64().unwrap(), 100);

    // Test UInt128 large values conversion to i64 will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.as_int64().is_err());
}

#[test]
fn test_as_float64_int128_conversion_failed() {
    // Test Int128 to f64 - now supports conversion
    let value = Value::Int128(100);
    assert_eq!(value.as_float64().unwrap(), 100.0);

    // Test large number conversion (may lose precision, but still succeeds)
    let value = Value::Int128(i128::MAX);
    assert!(value.as_float64().is_ok());
}

#[test]
fn test_as_float64_uint128_conversion_failed() {
    // Test UInt128 to f64 - now supports conversion
    let value = Value::UInt128(100);
    assert_eq!(value.as_float64().unwrap(), 100.0);

    // Test large number conversion (may lose precision, but still succeeds)
    let value = Value::UInt128(u128::MAX);
    assert!(value.as_float64().is_ok());
}

// ========================================================================
// Supplementary tests: comprehensive coverage of all data type branches for all as_xxxx methods
// ========================================================================

#[test]
fn test_as_bool_direct_bool_type() {
    // Test case where type itself is Bool
    let value_true = Value::Bool(true);
    assert!(value_true.as_bool().unwrap());

    let value_false = Value::Bool(false);
    assert!(!value_false.as_bool().unwrap());
}

#[test]
fn test_as_bool_string_parse_error() {
    // Test all cases where String type parsing bool fails
    let invalid_strings = vec![
        "yes", "no", "1", "0", "TRUE", "FALSE", "True", "False", "t", "f", "y", "n", "on", "off",
        "", "  ", "null", "None",
    ];

    for invalid_str in invalid_strings {
        let value = Value::String(invalid_str.to_string());
        assert!(
            value.as_bool().is_err(),
            "String '{}' should not be able to convert to bool",
            invalid_str
        );
    }

    // Test valid bool strings
    let value_true = Value::String("true".to_string());
    assert!(value_true.as_bool().unwrap());

    let value_false = Value::String("false".to_string());
    assert!(!value_false.as_bool().unwrap());
}

#[test]
fn test_as_bool_all_unsupported_types() {
    // Test all types that do not support conversion to bool
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Char type
    assert!(matches!(
        Value::Char('a').as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Float32 type
    assert!(matches!(
        Value::Float32(1.5).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Float64 type
    assert!(matches!(
        Value::Float64(2.5).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Date type
    assert!(matches!(
        Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Time type
    assert!(matches!(
        Value::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // DateTime type
    assert!(matches!(
        Value::DateTime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        )
        .as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Instant type
    assert!(matches!(
        Value::Instant(Utc::now()).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // BigInteger type
    assert!(matches!(
        Value::BigInteger(BigInt::from(123)).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // BigDecimal type
    assert!(matches!(
        Value::BigDecimal(BigDecimal::from_str("123.45").unwrap()).as_bool(),
        Err(ValueError::ConversionFailed { .. })
    ));
}

#[test]
fn test_as_int32_direct_int32_type() {
    // Test case where type itself is Int32
    let value = Value::Int32(42);
    assert_eq!(value.as_int32().unwrap(), 42);

    let value_negative = Value::Int32(-100);
    assert_eq!(value_negative.as_int32().unwrap(), -100);

    let value_max = Value::Int32(i32::MAX);
    assert_eq!(value_max.as_int32().unwrap(), i32::MAX);

    let value_min = Value::Int32(i32::MIN);
    assert_eq!(value_min.as_int32().unwrap(), i32::MIN);
}

#[test]
fn test_as_int32_int64_overflow_cases() {
    // Test various cases of Int64 overflow

    // Positive overflow
    let value_max = Value::Int64(i64::MAX);
    assert!(matches!(
        value_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int64(i64::MIN);
    assert!(matches!(
        value_min.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 max value
    let value_over_max = Value::Int64(i32::MAX as i64 + 1);
    assert!(matches!(
        value_over_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 min value
    let value_under_min = Value::Int64(i32::MIN as i64 - 1);
    assert!(matches!(
        value_under_min.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int64(1000);
    assert_eq!(value_in_range.as_int32().unwrap(), 1000);
}

#[test]
fn test_as_int32_int128_overflow_cases() {
    // Test various cases of Int128 overflow

    // Positive overflow
    let value_max = Value::Int128(i128::MAX);
    assert!(matches!(
        value_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int128(i128::MIN);
    assert!(matches!(
        value_min.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 max value
    let value_over_max = Value::Int128(i32::MAX as i128 + 1);
    assert!(matches!(
        value_over_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int128(500);
    assert_eq!(value_in_range.as_int32().unwrap(), 500);
}

#[test]
fn test_as_int32_uint32_overflow_cases() {
    // Test various cases of UInt32 overflow

    // u32::MAX exceeds i32 range
    let value_max = Value::UInt32(u32::MAX);
    assert!(matches!(
        value_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32::MAX
    let value_over_max = Value::UInt32(i32::MAX as u32 + 1);
    assert!(matches!(
        value_over_max.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::UInt32(100);
    assert_eq!(value_in_range.as_int32().unwrap(), 100);

    // i32::MAX should convert successfully
    let value_max_valid = Value::UInt32(i32::MAX as u32);
    assert_eq!(value_max_valid.as_int32().unwrap(), i32::MAX);
}

#[test]
fn test_as_int32_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("not_a_number".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Floating point string
    let value = Value::String("123.45".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Leading or trailing spaces
    let value = Value::String("  123  ".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Contains non-numeric characters
    let value = Value::String("123abc".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Number string out of range
    let value = Value::String("9999999999999999999".to_string());
    assert!(matches!(
        value.as_int32(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid number string should succeed
    let value = Value::String("12345".to_string());
    assert_eq!(value.as_int32().unwrap(), 12345);

    let value_negative = Value::String("-9876".to_string());
    assert_eq!(value_negative.as_int32().unwrap(), -9876);
}

#[test]
fn test_as_int64_direct_int64_type() {
    // Test case where type itself is Int64
    let value = Value::Int64(123456789);
    assert_eq!(value.as_int64().unwrap(), 123456789);

    let value_negative = Value::Int64(-987654321);
    assert_eq!(value_negative.as_int64().unwrap(), -987654321);

    let value_max = Value::Int64(i64::MAX);
    assert_eq!(value_max.as_int64().unwrap(), i64::MAX);

    let value_min = Value::Int64(i64::MIN);
    assert_eq!(value_min.as_int64().unwrap(), i64::MIN);
}

#[test]
fn test_as_int64_int128_overflow_cases() {
    // Test various cases of Int128 overflow

    // Positive overflow
    let value_max = Value::Int128(i128::MAX);
    assert!(matches!(
        value_max.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int128(i128::MIN);
    assert!(matches!(
        value_min.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64 max value
    let value_over_max = Value::Int128(i64::MAX as i128 + 1);
    assert!(matches!(
        value_over_max.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64 min value
    let value_under_min = Value::Int128(i64::MIN as i128 - 1);
    assert!(matches!(
        value_under_min.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int128(999999);
    assert_eq!(value_in_range.as_int64().unwrap(), 999999);
}

#[test]
fn test_as_int64_uint64_overflow_cases() {
    // Test various cases of UInt64 overflow

    // u64::MAX exceeds i64 range
    let value_max = Value::UInt64(u64::MAX);
    assert!(matches!(
        value_max.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64::MAX
    let value_over_max = Value::UInt64(i64::MAX as u64 + 1);
    assert!(matches!(
        value_over_max.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::UInt64(123456);
    assert_eq!(value_in_range.as_int64().unwrap(), 123456);

    // i64::MAX should convert successfully
    let value_max_valid = Value::UInt64(i64::MAX as u64);
    assert_eq!(value_max_valid.as_int64().unwrap(), i64::MAX);
}

#[test]
fn test_as_int64_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("invalid_number".to_string());
    assert!(matches!(
        value.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Floating point string
    let value = Value::String("456.789".to_string());
    assert!(matches!(
        value.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Number string out of range
    let value = Value::String("99999999999999999999999999999".to_string());
    assert!(matches!(
        value.as_int64(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid number string should succeed
    let value = Value::String("123456789".to_string());
    assert_eq!(value.as_int64().unwrap(), 123456789);

    let value_negative = Value::String("-987654321".to_string());
    assert_eq!(value_negative.as_int64().unwrap(), -987654321);
}

#[test]
fn test_as_float64_direct_float64_type() {
    // Test case where type itself is Float64
    let value = Value::Float64(3.5);
    assert_eq!(value.as_float64().unwrap(), 3.5);

    let value_negative = Value::Float64(-2.5);
    assert_eq!(value_negative.as_float64().unwrap(), -2.5);

    let value_zero = Value::Float64(0.0);
    assert_eq!(value_zero.as_float64().unwrap(), 0.0);

    // Test special values
    let value_inf = Value::Float64(f64::INFINITY);
    assert_eq!(value_inf.as_float64().unwrap(), f64::INFINITY);

    let value_neg_inf = Value::Float64(f64::NEG_INFINITY);
    assert_eq!(value_neg_inf.as_float64().unwrap(), f64::NEG_INFINITY);

    let value_nan = Value::Float64(f64::NAN);
    assert!(value_nan.as_float64().unwrap().is_nan());
}

#[test]
fn test_as_float64_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("not_a_float".to_string());
    assert!(matches!(
        value.as_float64(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.as_float64(),
        Err(ValueError::ConversionError(_))
    ));

    // Contains non-numeric characters
    let value = Value::String("12.34abc".to_string());
    assert!(matches!(
        value.as_float64(),
        Err(ValueError::ConversionError(_))
    ));

    // Multiple decimal points
    let value = Value::String("12.34.56".to_string());
    assert!(matches!(
        value.as_float64(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid floating point string should succeed
    let value = Value::String("3.5".to_string());
    assert_eq!(value.as_float64().unwrap(), 3.5);

    let value_negative = Value::String("-2.5".to_string());
    assert_eq!(value_negative.as_float64().unwrap(), -2.5);

    let value_scientific = Value::String("1.23e10".to_string());
    assert_eq!(value_scientific.as_float64().unwrap(), 1.23e10);
}

#[test]
fn test_as_float64_biginteger_conversion_error() {
    use std::str::FromStr;

    // Test BigInteger conversion failure cases
    // Note: Very large BigInteger may convert to f64::INFINITY instead of returning error

    // BigInteger within normal range should convert successfully
    let value = Value::BigInteger(BigInt::from(12345));
    assert_eq!(value.as_float64().unwrap(), 12345.0);

    // Very large BigInteger will convert to INFINITY
    let large_big_int = BigInt::from_str(&"9".repeat(400)).unwrap();
    let value = Value::BigInteger(large_big_int);
    let result = value.as_float64().unwrap();
    assert!(result.is_infinite() && result.is_sign_positive());

    // Test negative numbers
    let value_negative = Value::BigInteger(BigInt::from(-999999));
    assert_eq!(value_negative.as_float64().unwrap(), -999999.0);
}

#[test]
fn test_as_float64_bigdecimal_conversion_error() {
    use std::str::FromStr;

    // Test BigDecimal conversion failure cases

    // BigDecimal within normal range should convert successfully
    let value = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((value.as_float64().unwrap() - 123.456).abs() < 1e-10);

    // Very large BigDecimal will convert to INFINITY
    let large_big_decimal = BigDecimal::from_str("1.0e400").unwrap();
    let value = Value::BigDecimal(large_big_decimal);
    let result = value.as_float64().unwrap();
    assert!(result.is_infinite() && result.is_sign_positive());

    // Very small BigDecimal (negative) will convert to NEG_INFINITY
    let small_big_decimal = BigDecimal::from_str("-1.0e400").unwrap();
    let value = Value::BigDecimal(small_big_decimal);
    let result = value.as_float64().unwrap();
    assert!(result.is_infinite() && result.is_sign_negative());

    // Test high precision decimals
    let value = Value::BigDecimal(BigDecimal::from_str("0.123456789012345").unwrap());
    assert!((value.as_float64().unwrap() - 0.123456789012345).abs() < 1e-15);
}

#[test]
fn test_as_int32_all_unsigned_types() {
    // Ensure all unsigned integer types are tested

    // UInt8 - Always within range
    let value = Value::UInt8(255);
    assert_eq!(value.as_int32().unwrap(), 255);

    // UInt16 - Always within range
    let value = Value::UInt16(65535);
    assert_eq!(value.as_int32().unwrap(), 65535);

    // UInt32 - May overflow
    let value_ok = Value::UInt32(100);
    assert_eq!(value_ok.as_int32().unwrap(), 100);

    let value_overflow = Value::UInt32(u32::MAX);
    assert!(value_overflow.as_int32().is_err());

    // UInt64 - May overflow
    let value_ok = Value::UInt64(200);
    assert_eq!(value_ok.as_int32().unwrap(), 200);

    let value_overflow = Value::UInt64(u64::MAX);
    assert!(value_overflow.as_int32().is_err());

    // UInt128 - May overflow
    let value_ok = Value::UInt128(300);
    assert_eq!(value_ok.as_int32().unwrap(), 300);

    let value_overflow = Value::UInt128(u128::MAX);
    assert!(value_overflow.as_int32().is_err());
}

#[test]
fn test_as_int64_all_unsigned_types() {
    // Ensure all unsigned integer types are tested

    // UInt8 - Always within range
    let value = Value::UInt8(255);
    assert_eq!(value.as_int64().unwrap(), 255);

    // UInt16 - Always within range
    let value = Value::UInt16(65535);
    assert_eq!(value.as_int64().unwrap(), 65535);

    // UInt32 - Always within range
    let value = Value::UInt32(u32::MAX);
    assert_eq!(value.as_int64().unwrap(), u32::MAX as i64);

    // UInt64 - May overflow
    let value_ok = Value::UInt64(1000);
    assert_eq!(value_ok.as_int64().unwrap(), 1000);

    let value_overflow = Value::UInt64(u64::MAX);
    assert!(value_overflow.as_int64().is_err());

    // UInt128 - May overflow
    let value_ok = Value::UInt128(2000);
    assert_eq!(value_ok.as_int64().unwrap(), 2000);

    let value_overflow = Value::UInt128(u128::MAX);
    assert!(value_overflow.as_int64().is_err());
}

#[test]
fn test_as_float64_all_integer_types() {
    // Ensure all integer types to float64 conversion are tested

    // Signed integers
    assert_eq!(Value::Int8(127).as_float64().unwrap(), 127.0);
    assert_eq!(Value::Int8(-128).as_float64().unwrap(), -128.0);

    assert_eq!(Value::Int16(32767).as_float64().unwrap(), 32767.0);
    assert_eq!(Value::Int16(-32768).as_float64().unwrap(), -32768.0);

    assert_eq!(
        Value::Int32(i32::MAX).as_float64().unwrap(),
        i32::MAX as f64
    );
    assert_eq!(
        Value::Int32(i32::MIN).as_float64().unwrap(),
        i32::MIN as f64
    );

    assert_eq!(
        Value::Int64(i64::MAX).as_float64().unwrap(),
        i64::MAX as f64
    );
    assert_eq!(
        Value::Int64(i64::MIN).as_float64().unwrap(),
        i64::MIN as f64
    );

    assert_eq!(
        Value::Int128(i128::MAX).as_float64().unwrap(),
        i128::MAX as f64
    );
    assert_eq!(
        Value::Int128(i128::MIN).as_float64().unwrap(),
        i128::MIN as f64
    );

    // Unsigned integers
    assert_eq!(Value::UInt8(255).as_float64().unwrap(), 255.0);
    assert_eq!(Value::UInt16(65535).as_float64().unwrap(), 65535.0);
    assert_eq!(
        Value::UInt32(u32::MAX).as_float64().unwrap(),
        u32::MAX as f64
    );
    assert_eq!(
        Value::UInt64(u64::MAX).as_float64().unwrap(),
        u64::MAX as f64
    );
    assert_eq!(
        Value::UInt128(u128::MAX).as_float64().unwrap(),
        u128::MAX as f64
    );
}

#[test]
fn test_as_string_direct_string_type() {
    // Test case where type itself is String
    let value = Value::String("hello world".to_string());
    assert_eq!(value.as_string().unwrap(), "hello world");

    let value_empty = Value::String("".to_string());
    assert_eq!(value_empty.as_string().unwrap(), "");

    let value_unicode = Value::String("你好世界🌍".to_string());
    assert_eq!(value_unicode.as_string().unwrap(), "你好世界🌍");
}

#[test]
fn test_conversion_with_edge_values() {
    // Test boundary value conversions

    // Int32 boundary values
    assert_eq!(Value::Int32(i32::MAX).as_int64().unwrap(), i32::MAX as i64);
    assert_eq!(Value::Int32(i32::MIN).as_int64().unwrap(), i32::MIN as i64);
    assert_eq!(
        Value::Int32(i32::MAX).as_float64().unwrap(),
        i32::MAX as f64
    );
    assert_eq!(
        Value::Int32(i32::MIN).as_float64().unwrap(),
        i32::MIN as f64
    );

    // Int64 boundary values
    assert_eq!(
        Value::Int64(i64::MAX).as_float64().unwrap(),
        i64::MAX as f64
    );
    assert_eq!(
        Value::Int64(i64::MIN).as_float64().unwrap(),
        i64::MIN as f64
    );

    // UInt32 boundary values
    assert_eq!(Value::UInt32(u32::MAX).as_int64().unwrap(), u32::MAX as i64);
    assert_eq!(
        Value::UInt32(u32::MAX).as_float64().unwrap(),
        u32::MAX as f64
    );

    // Float32 boundary values
    assert_eq!(
        Value::Float32(f32::MAX).as_float64().unwrap(),
        f32::MAX as f64
    );
    assert_eq!(
        Value::Float32(f32::MIN).as_float64().unwrap(),
        f32::MIN as f64
    );
}

/// Test BigDecimal to i32 conversion out of range cases
#[test]
fn test_as_int32_bigdecimal_out_of_range() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    // Create a BigDecimal out of i32 range
    let huge_decimal = BigDecimal::from_str("999999999999999999.123").unwrap();
    let value = Value::BigDecimal(huge_decimal);

    // Attempting conversion should fail
    let result = value.as_int32();
    assert!(result.is_err());
}

/// Test non-numeric type to i32 conversion failure
#[test]
fn test_as_int32_non_numeric_type_conversion_failed() {
    use chrono::NaiveDate;

    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);

    // Date type cannot convert to i32
    let result = value.as_int32();
    assert!(result.is_err());

    // Verify error type
    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Date);
            assert_eq!(to, DataType::Int32);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Test other non-numeric types
    use chrono::NaiveTime;
    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);
    assert!(value.as_int32().is_err());

    use chrono::DateTime;
    let datetime = DateTime::from_timestamp(1_000_000_000, 0)
        .unwrap()
        .naive_utc();
    let value = Value::DateTime(datetime);
    assert!(value.as_int32().is_err());
}

/// Test non-numeric type to i64 conversion failure
#[test]
fn test_as_int64_non_numeric_type_conversion_failed() {
    use chrono::NaiveTime;

    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);

    // Time type cannot convert to i64
    let result = value.as_int64();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Time);
            assert_eq!(to, DataType::Int64);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Test Date type
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);
    assert!(value.as_int64().is_err());
}

/// Test BigInteger and BigDecimal to i64 conversion
#[test]
fn test_as_int64_big_types_edge_cases() {
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;
    use std::str::FromStr;

    // BigInteger out of i64 range
    let huge_bigint = BigInt::from_str("99999999999999999999").unwrap();
    let value = Value::BigInteger(huge_bigint);
    let result = value.as_int64();
    assert!(result.is_err());

    // BigDecimal with decimal part should be able to convert
    let decimal = BigDecimal::from_str("123.456").unwrap();
    let value = Value::BigDecimal(decimal);
    let result = value.as_int64();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
}

/// Test non-numeric type to f64 conversion failure
#[test]
fn test_as_float64_non_numeric_type_conversion_failed() {
    use chrono::{DateTime, Utc};

    // DateTime type cannot convert to f64
    let datetime = DateTime::from_timestamp(1_000_000_000, 0)
        .unwrap()
        .naive_utc();
    let value = Value::DateTime(datetime);

    let result = value.as_float64();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::DateTime);
            assert_eq!(to, DataType::Float64);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Instant type also cannot convert
    let instant = Utc::now();
    let value = Value::Instant(instant);
    assert!(value.as_float64().is_err());

    // Date type
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);
    assert!(value.as_float64().is_err());

    // Time type
    use chrono::NaiveTime;
    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);
    assert!(value.as_float64().is_err());
}

/// Test Float32 type as_bool conversion failure
#[test]
fn test_float32_as_bool_conversion() {
    let f32_zero = Value::Float32(0.0);
    let f32_nonzero = Value::Float32(3.5);

    // Float32 does not support conversion to bool (only integer types support)
    assert!(f32_zero.as_bool().is_err());
    assert!(f32_nonzero.as_bool().is_err());

    // Verify error type
    match f32_zero.as_bool() {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Float32);
            assert_eq!(to, DataType::Bool);
        }
        _ => panic!("Expected ConversionFailed error"),
    }
}

/// Test Char type numeric conversion
#[test]
fn test_char_numeric_conversions() {
    let char_val = Value::Char('A');

    // 'A' ASCII value is 65
    assert_eq!(char_val.as_int32().unwrap(), 65);
    assert_eq!(char_val.as_int64().unwrap(), 65);
    assert_eq!(char_val.as_float64().unwrap(), 65.0);
}

/// Test Float32 and Float64 to i64 conversion
#[test]
fn test_float_to_int64_conversions() {
    let f32_val = Value::Float32(42.7);
    assert_eq!(f32_val.as_int64().unwrap(), 42);

    let f64_val = Value::Float64(123.9);
    assert_eq!(f64_val.as_int64().unwrap(), 123);
}

/// Test UInt128 to i64 out of range
#[test]
fn test_uint128_to_int64_overflow() {
    use std::str::FromStr;

    // Create a u128 out of i64 range
    let huge_val = u128::from_str("99999999999999999999").unwrap();
    let value = Value::UInt128(huge_val);

    let result = value.as_int64();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128 value out of i64 range"));
        }
        _ => panic!("Expected ConversionError"),
    }
}

// ========================================================================
// Supplementary tests: cover user-specified missing test scenarios
// ========================================================================

/// Test Bool type conversion in as_int64 function
#[test]
fn test_as_int64_bool_conversion() {
    // Bool(true) should convert to 1
    let value_true = Value::Bool(true);
    assert_eq!(value_true.as_int64().unwrap(), 1i64);

    // Bool(false) should convert to 0
    let value_false = Value::Bool(false);
    assert_eq!(value_false.as_int64().unwrap(), 0i64);
}

/// Test cases where Int128 exceeds i64 range in as_int64 function
#[test]
fn test_as_int64_int128_overflow() {
    // Test positive overflow
    let value_max = Value::Int128(i128::MAX);
    let result = value_max.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test negative overflow
    let value_min = Value::Int128(i128::MIN);
    let result = value_min.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 underflow"),
    }
}

/// Test cases where UInt64 exceeds i64 range in as_int64 function
#[test]
fn test_as_int64_uint64_overflow() {
    // u64::MAX exceeds i64 range
    let value = Value::UInt64(u64::MAX);
    let result = value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 overflow"),
    }

    // u64 just exceeding i64::MAX
    let value = Value::UInt64(i64::MAX as u64 + 1);
    let result = value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 just over i64::MAX"),
    }
}

/// Test cases where UInt128 exceeds i64 range in as_int64 function
#[test]
fn test_as_int64_uint128_overflow() {
    // u128::MAX exceeds i64 range
    let value = Value::UInt128(u128::MAX);
    let result = value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u128 overflow"),
    }

    // u128 out of i64 range
    let large_value = Value::UInt128(i64::MAX as u128 + 1000);
    let result = large_value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for large u128"),
    }
}

/// Test cases where BigDecimal cannot convert to i64 in as_int64 function
#[test]
fn test_as_int64_bigdecimal_conversion_failed() {
    use std::str::FromStr;

    // Create a BigDecimal out of i64 range
    let huge_decimal = BigDecimal::from_str("999999999999999999999.123").unwrap();
    let value = Value::BigDecimal(huge_decimal);

    let result = value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigDecimal"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for BigDecimal conversion"),
    }

    // Test very small negative BigDecimal
    let tiny_decimal = BigDecimal::from_str("-999999999999999999999.123").unwrap();
    let value = Value::BigDecimal(tiny_decimal);
    let result = value.as_int64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigDecimal"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for negative BigDecimal conversion"),
    }
}

/// Test cases where String cannot parse to f64 in as_float64 function
#[test]
fn test_as_float64_string_parse_failed() {
    // Not a number at all string
    let value = Value::String("not_a_number".to_string());
    let result = value.as_float64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_number"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Empty string
    let value = Value::String("".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Contains non-numeric characters
    let value = Value::String("12.34xyz".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Multiple decimal points
    let value = Value::String("1.2.3".to_string());
    let result = value.as_float64();
    assert!(result.is_err());
}

/// Test cases where BigInteger cannot convert to f64 in as_float64 function
/// Note: Actually BigInteger to_f64() returns Some(f64::INFINITY) for very large numbers
/// Here we test boundary cases that return None (if any exist)
#[test]
fn test_as_float64_biginteger_conversion_edge_cases() {
    use std::str::FromStr;

    // BigInteger within normal range should convert successfully
    let normal_value = Value::BigInteger(BigInt::from(12345));
    assert_eq!(normal_value.as_float64().unwrap(), 12345.0);

    // Very large BigInteger will convert to INFINITY (instead of returning error)
    let huge_bigint = BigInt::from_str(&"9".repeat(400)).unwrap();
    let value = Value::BigInteger(huge_bigint);
    let result = value.as_float64();
    assert!(result.is_ok());
    let float_result = result.unwrap();
    assert!(float_result.is_infinite() && float_result.is_sign_positive());

    // Negative very large BigInteger will convert to NEG_INFINITY
    let neg_huge_bigint = BigInt::from_str(&format!("-{}", "9".repeat(400))).unwrap();
    let value = Value::BigInteger(neg_huge_bigint);
    let result = value.as_float64();
    assert!(result.is_ok());
    let float_result = result.unwrap();
    assert!(float_result.is_infinite() && float_result.is_sign_negative());

    // Zero value BigInteger
    let zero_bigint = BigInt::from(0);
    let value = Value::BigInteger(zero_bigint);
    assert_eq!(value.as_float64().unwrap(), 0.0);
}

/// Test cases where BigDecimal cannot convert to f64 in as_float64 function
/// Note: Similar to BigInteger, BigDecimal to_f64() returns INFINITY for very large numbers
#[test]
fn test_as_float64_bigdecimal_conversion_edge_cases() {
    use std::str::FromStr;

    // BigDecimal within normal range should convert successfully
    let normal_value = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((normal_value.as_float64().unwrap() - 123.456).abs() < 1e-10);

    // Very large BigDecimal will convert to INFINITY
    let huge_decimal = BigDecimal::from_str("1e400").unwrap();
    let value = Value::BigDecimal(huge_decimal);
    let result = value.as_float64();
    assert!(result.is_ok());
    let float_result = result.unwrap();
    assert!(float_result.is_infinite() && float_result.is_sign_positive());

    // Negative very large BigDecimal will convert to NEG_INFINITY
    let neg_huge_decimal = BigDecimal::from_str("-1e400").unwrap();
    let value = Value::BigDecimal(neg_huge_decimal);
    let result = value.as_float64();
    assert!(result.is_ok());
    let float_result = result.unwrap();
    assert!(float_result.is_infinite() && float_result.is_sign_negative());

    // Very small positive BigDecimal (close to zero)
    let tiny_decimal = BigDecimal::from_str("1e-400").unwrap();
    let value = Value::BigDecimal(tiny_decimal);
    let result = value.as_float64();
    assert!(result.is_ok());
    // Very small numbers may convert to 0.0
    let float_result = result.unwrap();
    assert!(float_result >= 0.0);

    // Zero value BigDecimal
    let zero_decimal = BigDecimal::from_str("0.0").unwrap();
    let value = Value::BigDecimal(zero_decimal);
    assert_eq!(value.as_float64().unwrap(), 0.0);
}

/// Specifically test all branches of Bool type in as_float64 function
#[test]
fn test_as_float64_bool_type_all_branches() {
    // Test Bool(true) conversion to f64
    let value_true = Value::Bool(true);
    let result = value_true.as_float64();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1.0);

    // Test Bool(false) conversion to f64
    let value_false = Value::Bool(false);
    let result = value_false.as_float64();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0.0);

    // Verify both branches are tested
    assert_ne!(
        value_true.as_float64().unwrap(),
        value_false.as_float64().unwrap()
    );
}

/// Specifically test various cases where String type cannot parse to f64 in as_float64 function
#[test]
fn test_as_float64_string_parse_all_error_cases() {
    // Test not a number at all string
    let value = Value::String("abc".to_string());
    let result = value.as_float64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("abc"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError"),
    }

    // Test empty string
    let value = Value::String("".to_string());
    let result = value.as_float64();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(_)) => {
            // Expected
        }
        _ => panic!("Expected ConversionError for empty string"),
    }

    // Test string containing letters
    let value = Value::String("12.34abc".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test multiple decimal points
    let value = Value::String("1.2.3".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test string with only symbols
    let value = Value::String("+".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    let value = Value::String("-".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test string containing spaces (Leading or trailing spaces)
    let value = Value::String("  123.45  ".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test special characters
    let value = Value::String("@#$%".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test Chinese characters
    let value = Value::String("一二三".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Test mixed characters
    let value = Value::String("123abc456".to_string());
    let result = value.as_float64();
    assert!(result.is_err());

    // Comparison: Valid floating point string should succeed
    let valid_value = Value::String("123.45".to_string());
    assert!(valid_value.as_float64().is_ok());
    assert_eq!(valid_value.as_float64().unwrap(), 123.45);
}
