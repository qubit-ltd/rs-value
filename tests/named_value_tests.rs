/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Named Single Value Unit Tests
//!
//! Tests various functionalities of the named single value container。
//!

use chrono::DateTime as UtcDateTime;
use chrono::{
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use qubit_datatype::DataType;
use qubit_value::{
    NamedValue,
    Value,
};

#[test]
fn test_named_value_new() {
    let nv = NamedValue::new("port", Value::Int32(8080));
    assert_eq!(nv.name(), "port");
    assert_eq!(nv.get_int32().unwrap(), 8080);
}

#[test]
fn test_named_value_name_getter() {
    let nv = NamedValue::new("config", Value::Bool(true));
    assert_eq!(nv.name(), "config");
}

#[test]
fn test_named_value_set_name() {
    let mut nv = NamedValue::new("config", Value::Bool(true));
    nv.set_name("new_config");
    assert_eq!(nv.name(), "new_config");
}

#[test]
fn test_named_value_into_parts() {
    let nv = NamedValue::new("port", Value::Int32(8080));
    let (name, value) = nv.into_parts();
    assert_eq!(name, "port");
    assert_eq!(value, Value::Int32(8080));
}

#[test]
fn test_named_value_deref_mut_assignment() {
    let mut nv = NamedValue::new("counter", Value::Int32(0));
    *nv = Value::Int32(42);
    assert_eq!(nv.get_int32().unwrap(), 42);
}

// ------------------- Individual get_xxx() method coverage -------------------

#[test]
fn test_named_value_get_bool() {
    let nv = NamedValue::new("b", Value::Bool(true));
    assert!(nv.get_bool().unwrap());
}

#[test]
fn test_named_value_get_char() {
    let nv = NamedValue::new("c", Value::Char('A'));
    assert_eq!(nv.get_char().unwrap(), 'A');
}

#[test]
fn test_named_value_get_int8() {
    let nv = NamedValue::new("i8", Value::Int8(-8));
    assert_eq!(nv.get_int8().unwrap(), -8);
}

#[test]
fn test_named_value_get_int16() {
    let nv = NamedValue::new("i16", Value::Int16(-16));
    assert_eq!(nv.get_int16().unwrap(), -16);
}

#[test]
fn test_named_value_get_int32() {
    let nv = NamedValue::new("i32", Value::Int32(-32));
    assert_eq!(nv.get_int32().unwrap(), -32);
}

#[test]
fn test_named_value_get_int64() {
    let nv = NamedValue::new("i64", Value::Int64(-64));
    assert_eq!(nv.get_int64().unwrap(), -64);
}

#[test]
fn test_named_value_get_int128() {
    let nv = NamedValue::new("i128", Value::Int128(-128));
    assert_eq!(nv.get_int128().unwrap(), -128);
}

#[test]
fn test_named_value_get_uint8() {
    let nv = NamedValue::new("u8", Value::UInt8(8));
    assert_eq!(nv.get_uint8().unwrap(), 8);
}

#[test]
fn test_named_value_get_uint16() {
    let nv = NamedValue::new("u16", Value::UInt16(16));
    assert_eq!(nv.get_uint16().unwrap(), 16);
}

#[test]
fn test_named_value_get_uint32() {
    let nv = NamedValue::new("u32", Value::UInt32(32));
    assert_eq!(nv.get_uint32().unwrap(), 32);
}

#[test]
fn test_named_value_get_uint64() {
    let nv = NamedValue::new("u64", Value::UInt64(64));
    assert_eq!(nv.get_uint64().unwrap(), 64);
}

#[test]
fn test_named_value_get_uint128() {
    let nv = NamedValue::new("u128", Value::UInt128(128));
    assert_eq!(nv.get_uint128().unwrap(), 128);
}

#[test]
fn test_named_value_get_float32() {
    let nv = NamedValue::new("f32", Value::Float32(1.5));
    assert_eq!(nv.get_float32().unwrap(), 1.5);
}

#[test]
fn test_named_value_get_float64() {
    let nv = NamedValue::new("f64", Value::Float64(2.5));
    assert_eq!(nv.get_float64().unwrap(), 2.5);
}

#[test]
fn test_named_value_get_string() {
    let nv = NamedValue::new("s", Value::String("hello".to_string()));
    assert_eq!(nv.get_string().unwrap(), "hello");
}

#[test]
fn test_named_value_get_date() {
    let nv = NamedValue::new(
        "d",
        Value::Date(NaiveDate::from_ymd_opt(2020, 5, 17).unwrap()),
    );
    assert_eq!(
        nv.get_date().unwrap(),
        NaiveDate::from_ymd_opt(2020, 5, 17).unwrap()
    );
}

#[test]
fn test_named_value_get_time() {
    let nv = NamedValue::new(
        "t",
        Value::Time(NaiveTime::from_hms_milli_opt(13, 14, 15, 123).unwrap()),
    );
    assert_eq!(
        nv.get_time().unwrap(),
        NaiveTime::from_hms_milli_opt(13, 14, 15, 123).unwrap()
    );
}

#[test]
fn test_named_value_get_datetime() {
    let nv = NamedValue::new(
        "dt",
        Value::DateTime(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2021, 6, 7).unwrap(),
            NaiveTime::from_hms_opt(8, 9, 10).unwrap(),
        )),
    );
    assert_eq!(
        nv.get_datetime().unwrap(),
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2021, 6, 7).unwrap(),
            NaiveTime::from_hms_opt(8, 9, 10).unwrap(),
        )
    );
}

#[test]
fn test_named_value_get_instant() {
    let inst: UtcDateTime<Utc> = Utc::now();
    let nv = NamedValue::new("inst", Value::Instant(inst));
    assert_eq!(nv.get_instant().unwrap(), inst);
}

// ------------------- Generic set()/get<T>() coverage for each type -------------------

#[test]
fn test_named_value_set_get_bool() {
    let mut nv = NamedValue::new("b", Value::Bool(false));
    nv.set(true).unwrap();
    let b: bool = nv.get().unwrap();
    assert!(b);
}

#[test]
fn test_named_value_set_get_char() {
    let mut nv = NamedValue::new("c", Value::Char('x'));
    nv.set('A').unwrap();
    let c: char = nv.get().unwrap();
    assert_eq!(c, 'A');
}

#[test]
fn test_named_value_set_get_i8() {
    let mut nv = NamedValue::new("i8", Value::Int8(0));
    nv.set(-8i8).unwrap();
    let v: i8 = nv.get().unwrap();
    assert_eq!(v, -8);
}

#[test]
fn test_named_value_set_get_i16() {
    let mut nv = NamedValue::new("i16", Value::Int16(0));
    nv.set(-16i16).unwrap();
    let v: i16 = nv.get().unwrap();
    assert_eq!(v, -16);
}

#[test]
fn test_named_value_set_get_i32() {
    let mut nv = NamedValue::new("i32", Value::Int32(0));
    nv.set(-32i32).unwrap();
    let v: i32 = nv.get().unwrap();
    assert_eq!(v, -32);
}

#[test]
fn test_named_value_set_get_i64() {
    let mut nv = NamedValue::new("i64", Value::Int64(0));
    nv.set(-64i64).unwrap();
    let v: i64 = nv.get().unwrap();
    assert_eq!(v, -64);
}

#[test]
fn test_named_value_set_get_i128() {
    let mut nv = NamedValue::new("i128", Value::Int128(0));
    nv.set(-128i128).unwrap();
    let v: i128 = nv.get().unwrap();
    assert_eq!(v, -128);
}

#[test]
fn test_named_value_set_get_u8() {
    let mut nv = NamedValue::new("u8", Value::UInt8(0));
    nv.set(8u8).unwrap();
    let v: u8 = nv.get().unwrap();
    assert_eq!(v, 8);
}

#[test]
fn test_named_value_set_get_u16() {
    let mut nv = NamedValue::new("u16", Value::UInt16(0));
    nv.set(16u16).unwrap();
    let v: u16 = nv.get().unwrap();
    assert_eq!(v, 16);
}

#[test]
fn test_named_value_set_get_u32() {
    let mut nv = NamedValue::new("u32", Value::UInt32(0));
    nv.set(32u32).unwrap();
    let v: u32 = nv.get().unwrap();
    assert_eq!(v, 32);
}

#[test]
fn test_named_value_set_get_u64() {
    let mut nv = NamedValue::new("u64", Value::UInt64(0));
    nv.set(64u64).unwrap();
    let v: u64 = nv.get().unwrap();
    assert_eq!(v, 64);
}

#[test]
fn test_named_value_set_get_u128() {
    let mut nv = NamedValue::new("u128", Value::UInt128(0));
    nv.set(128u128).unwrap();
    let v: u128 = nv.get().unwrap();
    assert_eq!(v, 128);
}

#[test]
fn test_named_value_set_get_f32() {
    let mut nv = NamedValue::new("f32", Value::Float32(0.0));
    nv.set(1.5f32).unwrap();
    let v: f32 = nv.get().unwrap();
    assert_eq!(v, 1.5);
}

#[test]
fn test_named_value_set_get_f64() {
    let mut nv = NamedValue::new("f64", Value::Float64(0.0));
    nv.set(2.5f64).unwrap();
    let v: f64 = nv.get().unwrap();
    assert_eq!(v, 2.5);
}

#[test]
fn test_named_value_set_get_string() {
    let mut nv = NamedValue::new("s", Value::String(String::new()));
    nv.set("hello".to_string()).unwrap();
    let s: String = nv.get().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn test_named_value_set_get_date() {
    let mut nv = NamedValue::new(
        "d",
        Value::Date(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
    );
    let date = NaiveDate::from_ymd_opt(2020, 5, 17).unwrap();
    nv.set(date).unwrap();
    let got: NaiveDate = nv.get().unwrap();
    assert_eq!(got, date);
}

#[test]
fn test_named_value_set_get_time() {
    let mut nv = NamedValue::new("t", Value::Time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
    let time = NaiveTime::from_hms_milli_opt(13, 14, 15, 123).unwrap();
    nv.set(time).unwrap();
    let got: NaiveTime = nv.get().unwrap();
    assert_eq!(got, time);
}

#[test]
fn test_named_value_set_get_datetime() {
    let mut nv = NamedValue::new(
        "dt",
        Value::DateTime(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        )),
    );
    let dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2021, 6, 7).unwrap(),
        NaiveTime::from_hms_opt(8, 9, 10).unwrap(),
    );
    nv.set(dt).unwrap();
    let got: NaiveDateTime = nv.get().unwrap();
    assert_eq!(got, dt);
}

#[test]
fn test_named_value_set_get_instant() {
    let mut nv = NamedValue::new("inst", Value::Instant(Utc::now()));
    let inst: UtcDateTime<Utc> = Utc::now();
    nv.set(inst).unwrap();
    let got: UtcDateTime<Utc> = nv.get().unwrap();
    assert_eq!(got, inst);
}

// ------------------- Other general behaviors -------------------

#[test]
fn test_named_value_is_empty() {
    let nv = NamedValue::new("e", Value::Empty(DataType::Int32));
    assert!(nv.is_empty());
}

#[test]
fn test_named_value_clear() {
    let mut nv = NamedValue::new("e", Value::Int32(7));
    nv.clear();
    assert!(nv.is_empty());
    assert_eq!(nv.data_type(), DataType::Int32);
}

#[test]
fn test_named_value_set_type() {
    let mut nv = NamedValue::new("e", Value::Int32(7));
    nv.set_type(DataType::String);
    assert!(nv.is_empty());
    assert_eq!(nv.data_type(), DataType::String);
}

#[test]
fn test_named_value_data_type() {
    let nv = NamedValue::new("i32", Value::Int32(1));
    assert_eq!(nv.data_type(), DataType::Int32);
}
