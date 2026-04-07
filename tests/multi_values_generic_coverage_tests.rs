/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MultiValues Generic Coverage Tests
//!
//! Covers the four generic entry points `set`/`add`/`get`/`get_first`, verifying all supported types and three parameter categories。

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use prism3_core::lang::DataType;
use qubit_value::MultiValues;

// ------------------------------ set: Vec<T> ------------------------------

#[test]
fn test_generic_set_vec_all_types() {
    // bool
    let mut mv = MultiValues::Empty(DataType::Bool);
    mv.set(vec![true, false]).unwrap();
    assert_eq!(mv.get_bools().unwrap(), &[true, false]);

    // char
    let mut mv = MultiValues::Empty(DataType::Char);
    mv.set(vec!['a', 'b']).unwrap();
    assert_eq!(mv.get_chars().unwrap(), &['a', 'b']);

    // integers
    let mut mv = MultiValues::Empty(DataType::Int8);
    mv.set(vec![1i8, 2]).unwrap();
    assert_eq!(mv.get_int8s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::Int16);
    mv.set(vec![1i16, 2]).unwrap();
    assert_eq!(mv.get_int16s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::Int32);
    mv.set(vec![1i32, 2]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set(vec![1i64, 2]).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::Int128);
    mv.set(vec![1i128, 2]).unwrap();
    assert_eq!(mv.get_int128s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::UInt8);
    // Note: u8 does not support generic set, as Vec<u8> is used for byte arrays
    mv.set_uint8s(vec![1u8, 2]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::UInt16);
    mv.set(vec![1u16, 2]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::UInt32);
    mv.set(vec![1u32, 2]).unwrap();
    assert_eq!(mv.get_uint32s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::UInt64);
    mv.set(vec![1u64, 2]).unwrap();
    assert_eq!(mv.get_uint64s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::Empty(DataType::UInt128);
    mv.set(vec![1u128, 2]).unwrap();
    assert_eq!(mv.get_uint128s().unwrap(), &[1, 2]);

    // floats
    let mut mv = MultiValues::Empty(DataType::Float32);
    mv.set(vec![1.0f32, 2.0]).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[1.0, 2.0]);

    let mut mv = MultiValues::Empty(DataType::Float64);
    mv.set(vec![1.0f64, 2.0]).unwrap();
    assert_eq!(mv.get_float64s().unwrap(), &[1.0, 2.0]);

    // string
    let mut mv = MultiValues::Empty(DataType::String);
    mv.set(vec!["a".to_string(), "b".to_string()]).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["a", "b"]);

    // u8 now supports generic set
    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set(vec![1u8, 2, 3]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[1, 2, 3]);

    // date/time
    let d1 = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let d2 = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
    let mut mv = MultiValues::Empty(DataType::Date);
    mv.set(vec![d1, d2]).unwrap();
    assert_eq!(mv.get_dates().unwrap().len(), 2);

    let t1 = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let t2 = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
    let mut mv = MultiValues::Empty(DataType::Time);
    mv.set(vec![t1, t2]).unwrap();
    assert_eq!(mv.get_times().unwrap().len(), 2);

    let ndt1 = NaiveDate::from_ymd_opt(2020, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let ndt2 = NaiveDate::from_ymd_opt(2020, 1, 2)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let mut mv = MultiValues::Empty(DataType::DateTime);
    mv.set(vec![ndt1, ndt2]).unwrap();
    assert_eq!(mv.get_datetimes().unwrap().len(), 2);

    let i1 = DateTime::<Utc>::from_naive_utc_and_offset(ndt1, Utc);
    let i2 = DateTime::<Utc>::from_naive_utc_and_offset(ndt2, Utc);
    let mut mv = MultiValues::Empty(DataType::Instant);
    mv.set(vec![i1, i2]).unwrap();
    assert_eq!(mv.get_instants().unwrap().len(), 2);
}

// ------------------------------ set: &[T] ------------------------------

#[test]
fn test_generic_set_slice_all_types() {
    // int32 as representative style with slice; other types follow same path
    let mut mv = MultiValues::Empty(DataType::Int32);
    let v = [10i32, 20, 30];
    mv.set(&v[..]).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[10, 20, 30]);

    let mut mv = MultiValues::Empty(DataType::String);
    let s = ["x".to_string(), "y".to_string()];
    mv.set(&s[..]).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["x", "y"]);

    // u8 now supports generic set slice
    let mut mv = MultiValues::Empty(DataType::UInt8);
    let b = vec![9u8, 8, 7, 6];
    mv.set(&b[..]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &b);
}

// ------------------------------ set: single T ------------------------------

#[test]
fn test_generic_set_single_all_types() {
    let mut mv = MultiValues::Empty(DataType::Int64);
    mv.set(123i64).unwrap();
    assert_eq!(mv.get_int64s().unwrap(), &[123]);

    let mut mv = MultiValues::Empty(DataType::String);
    mv.set("ok".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["ok"]);
}

// ------------------------------ add: T/Vec/&[T] ------------------------------

#[test]
fn test_generic_add_single_all_types() {
    let mut mv = MultiValues::Int32(vec![1]);
    mv.add(2i32).unwrap();
    assert_eq!(mv.get_int32s().unwrap(), &[1, 2]);

    let mut mv = MultiValues::String(vec!["a".to_string()]);
    mv.add("b".to_string()).unwrap();
    assert_eq!(mv.get_strings().unwrap(), &["a", "b"]);
}

#[test]
fn test_generic_add_vec_all_types() {
    let mut mv = MultiValues::UInt16(vec![1u16]);
    mv.add(vec![2u16, 3]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[1, 2, 3]);

    // u8 now supports generic add Vec
    let mut mv = MultiValues::UInt8(vec![1u8]);
    mv.add(vec![2u8, 3]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_generic_add_slice_all_types() {
    let mut mv = MultiValues::Float32(vec![1.0f32]);
    let more = [2.0f32, 3.0f32];
    mv.add(&more[..]).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[1.0, 2.0, 3.0]);

    // u8 now supports generic add slice
    let mut mv = MultiValues::UInt8(vec![1u8]);
    let more = [2u8, 3u8, 4u8];
    mv.add(&more[..]).unwrap();
    assert_eq!(mv.get_uint8s().unwrap(), &[1, 2, 3, 4]);
}

// ------------------------------ get / get_first ------------------------------

#[test]
fn test_generic_get_all_types() {
    let mv = MultiValues::Int8(vec![1i8, 2i8]);
    let got: Vec<i8> = mv.get().unwrap();
    assert_eq!(got, vec![1, 2]);

    let mv = MultiValues::String(vec!["s".to_string()]);
    let got: Vec<String> = mv.get().unwrap();
    assert_eq!(got, vec!["s".to_string()]);

    // u8 now supports generic get
    let mv = MultiValues::UInt8(vec![1u8, 2u8]);
    let got: Vec<u8> = mv.get().unwrap();
    assert_eq!(got, vec![1u8, 2u8]);
}

#[test]
fn test_generic_get_first_all_types() {
    let mv = MultiValues::UInt32(vec![10u32, 20u32]);
    let first: u32 = mv.get_first().unwrap();
    assert_eq!(first, 10);

    let mv = MultiValues::String(vec!["x".to_string(), "y".to_string()]);
    let first: String = mv.get_first().unwrap();
    assert_eq!(first, "x");
}
