// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Core Unit Tests
//!
//! Tests for core and structural `Value` operations.

use std::str::FromStr;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireV1;

#[test]
fn test_value_creation() {
    let v = Value::Int32(42);
    assert_eq!(v.data_type(), DataType::Int32);
    assert!(!v.is_unset());
    assert_eq!(v.get_int32().unwrap(), 42);
}
#[test]
fn test_value_empty() {
    let v = Value::Unset(DataType::String);
    assert_eq!(v.data_type(), DataType::String);
    assert!(v.is_unset());
    assert!(matches!(v.get_string(), Err(ValueError::Missing(_))));
}
#[test]
fn test_value_unset() {
    let mut v = Value::Int32(42);
    v.unset();
    assert!(v.is_unset());
    assert_eq!(v.data_type(), DataType::Int32);
}
#[test]
fn test_value_set_type() {
    let mut v = Value::Int32(42);
    v.set_type(DataType::String);
    assert!(v.is_unset());
    assert_eq!(v.data_type(), DataType::String);
}
#[test]
fn test_value_set_retypes_existing_value() {
    let mut v = Value::Int32(42);
    v.set("hello".to_string());
    assert_eq!(v.data_type(), DataType::String);
    assert_eq!(v.get_string().unwrap(), "hello");

    v.set(true);
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
fn test_value_is_numeric_is_state_aware() {
    assert!(Value::Int32(42).is_numeric());
    assert!(Value::Float64(0.0).is_numeric());
    assert!(Value::BigInteger(BigInt::from(1)).is_numeric());
    assert!(!Value::Unset(DataType::Int32).is_numeric());
    assert!(!Value::String("42".to_string()).is_numeric());
    assert!(!Value::Bool(true).is_numeric());
}

#[test]
fn test_value_wire_rejects_non_finite_floats() {
    let finite = Value::Float64(1.25);
    let wire = ValueWireV1::try_from(finite.clone())
        .expect("finite value should fit the V1 wire contract");
    let json = serde_json::to_string(&wire).expect("serialize V1 wire");
    let decoded: ValueWireV1 = serde_json::from_str(&json).expect("deserialize V1 wire");
    assert_eq!(decoded.into_container(), finite.into());

    for value in [
        Value::Float32(f32::NAN),
        Value::Float64(f64::INFINITY),
        Value::Float64(f64::NEG_INFINITY),
    ] {
        assert!(matches!(
            ValueWireV1::try_from(value),
            Err(ValueWireEncodeError::NonFiniteFloat { .. })
        ));
    }
}
#[test]
fn test_value_new_unset_preserves_declared_type() {
    let v = Value::new_unset(DataType::String);
    assert_eq!(v.data_type(), DataType::String);
    assert!(v.is_unset());
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
    let mut value = Value::Unset(DataType::String);
    value.set("hello");
    assert_eq!(value.get_string().unwrap(), "hello");

    // Test creating Value from &str
    let value = Value::new("world");
    assert_eq!(value.get_string().unwrap(), "world");
}
#[test]
fn test_value_datetime_types() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;

    // Test Date
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let mut value = Value::Unset(DataType::Date);
    value.set(date);
    assert_eq!(value.get_date().unwrap(), date);
    assert_eq!(value.data_type(), DataType::Date);

    // Test Time
    let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    let mut value = Value::Unset(DataType::Time);
    value.set(time);
    assert_eq!(value.get_time().unwrap(), time);
    assert_eq!(value.data_type(), DataType::Time);

    // Test DateTime
    let datetime = NaiveDate::from_ymd_opt(2024, 1, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let mut value = Value::Unset(DataType::DateTime);
    value.set(datetime);
    assert_eq!(value.get_datetime().unwrap(), datetime);
    assert_eq!(value.data_type(), DataType::DateTime);

    // Test Instant
    let instant = DateTime::<Utc>::from_timestamp(1_700_000_000, 0)
        .expect("fixed test instant must be valid");
    let mut value = Value::Unset(DataType::Instant);
    value.set(instant);
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
    assert!(!v.is_unset());

    // Overwrite with a different type
    v.set("hello".to_string());
    assert_eq!(v.data_type(), DataType::String);
    assert!(!v.is_unset());
    assert_eq!(v.get_string().unwrap(), "hello");
    assert!(matches!(
        v.get_int32(),
        Err(ValueError::TypeMismatch { .. })
    ));

    // Overwrite with the same type
    v.set("world".to_string());
    assert_eq!(v.get_string().unwrap(), "world");
}
#[test]
fn test_data_type_coverage_all_variants() {
    // Test data_type() method coverage for all data type variants
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;

    // Empty type (all possible DataType)
    assert_eq!(Value::Unset(DataType::Bool).data_type(), DataType::Bool);
    assert_eq!(Value::Unset(DataType::Char).data_type(), DataType::Char);
    assert_eq!(Value::Unset(DataType::Int8).data_type(), DataType::Int8);
    assert_eq!(Value::Unset(DataType::Int16).data_type(), DataType::Int16);
    assert_eq!(Value::Unset(DataType::Int32).data_type(), DataType::Int32);
    assert_eq!(Value::Unset(DataType::Int64).data_type(), DataType::Int64);
    assert_eq!(Value::Unset(DataType::Int128).data_type(), DataType::Int128);
    assert_eq!(Value::Unset(DataType::UInt8).data_type(), DataType::UInt8);
    assert_eq!(Value::Unset(DataType::UInt16).data_type(), DataType::UInt16);
    assert_eq!(Value::Unset(DataType::UInt32).data_type(), DataType::UInt32);
    assert_eq!(Value::Unset(DataType::UInt64).data_type(), DataType::UInt64);
    assert_eq!(
        Value::Unset(DataType::UInt128).data_type(),
        DataType::UInt128
    );
    assert_eq!(
        Value::Unset(DataType::Float32).data_type(),
        DataType::Float32
    );
    assert_eq!(
        Value::Unset(DataType::Float64).data_type(),
        DataType::Float64
    );
    assert_eq!(Value::Unset(DataType::String).data_type(), DataType::String);
    assert_eq!(Value::Unset(DataType::Date).data_type(), DataType::Date);
    assert_eq!(Value::Unset(DataType::Time).data_type(), DataType::Time);
    assert_eq!(
        Value::Unset(DataType::DateTime).data_type(),
        DataType::DateTime
    );
    assert_eq!(
        Value::Unset(DataType::Instant).data_type(),
        DataType::Instant
    );
    assert_eq!(
        Value::Unset(DataType::BigInteger).data_type(),
        DataType::BigInteger
    );
    assert_eq!(
        Value::Unset(DataType::BigDecimal).data_type(),
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
    assert_eq!(
        Value::Instant(
            DateTime::<Utc>::from_timestamp(1_700_000_000, 0)
                .expect("fixed test instant must be valid"),
        )
        .data_type(),
        DataType::Instant
    );
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
fn test_is_unset_distinguishes_empty_inner_values() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;

    assert!(!Value::Bool(true).is_unset());
    assert!(!Value::Char('A').is_unset());
    assert!(!Value::Int8(1).is_unset());
    assert!(!Value::Int16(1).is_unset());
    assert!(!Value::Int32(1).is_unset());
    assert!(!Value::Int64(1).is_unset());
    assert!(!Value::Int128(1).is_unset());
    assert!(!Value::UInt8(1).is_unset());
    assert!(!Value::UInt16(1).is_unset());
    assert!(!Value::UInt32(1).is_unset());
    assert!(!Value::UInt64(1).is_unset());
    assert!(!Value::UInt128(1).is_unset());
    assert!(!Value::Float32(1.0).is_unset());
    assert!(!Value::Float64(1.0).is_unset());
    assert!(!Value::String(String::new()).is_unset());
    assert!(!Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()).is_unset());
    assert!(!Value::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).is_unset());
    assert!(
        !Value::DateTime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        )
        .is_unset()
    );
    assert!(
        !Value::Instant(
            DateTime::<Utc>::from_timestamp(1_700_000_000, 0)
                .expect("fixed test instant must be valid"),
        )
        .is_unset()
    );
    assert!(!Value::BigInteger(BigInt::from(123)).is_unset());
    assert!(!Value::BigDecimal(BigDecimal::from_str("123.45").unwrap()).is_unset());

    for &data_type in DataType::ALL {
        assert!(Value::Unset(data_type).is_unset());
    }

    let mut value = Value::String(String::new());
    value.unset();
    assert!(value.is_unset());
    assert_eq!(value.data_type(), DataType::String);
}
