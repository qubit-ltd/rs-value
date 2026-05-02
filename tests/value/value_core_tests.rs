/*****************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Core Unit Tests
//!
//! Tests for core and structural `Value` operations.
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
fn test_value_set_retypes_existing_value() {
    let mut v = Value::Int32(42);
    v.set("hello".to_string()).unwrap();
    assert_eq!(v.data_type(), DataType::String);
    assert_eq!(v.get_string().unwrap(), "hello");

    v.set(true).unwrap();
    assert_eq!(v.data_type(), DataType::Bool);
    assert!(v.get_bool().unwrap());
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
fn test_value_datetime_types() {
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

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
fn test_value_set_type_same_type() {
    // Test setting same type does not clear value
    let mut value = Value::Int32(42);
    value.set_type(DataType::Int32);
    assert_eq!(value.get_int32().unwrap(), 42);
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
fn test_data_type_coverage_all_variants() {
    // Test data_type() method coverage for all data type variants
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

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
    use chrono::{
        NaiveDate,
        NaiveTime,
        Utc,
    };

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
    assert!(
        !Value::DateTime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        )
        .is_empty()
    );
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
