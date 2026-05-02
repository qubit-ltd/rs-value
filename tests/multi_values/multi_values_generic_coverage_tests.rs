/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # MultiValues Generic Coverage Tests
//!
//! Covers the four generic entry points `set`/`add`/`get`/`get_first`, verifying all supported types and three parameter categories。

use chrono::{
    DateTime,
    NaiveDate,
    NaiveTime,
    Utc,
};
use qubit_datatype::DataType;
use qubit_value::{
    IntoValueDefault,
    MultiValues,
};

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
    mv.set(vec![1u8, 2, 3]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3]);

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

    let mut mv = MultiValues::Empty(DataType::UInt8);
    let b = vec![9u8, 8, 7, 6];
    mv.set(&b[..]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), b);
}

#[test]
fn test_generic_new_convenient_inputs_for_coverage() {
    let values = MultiValues::new(vec![1i32, 2]);
    assert_eq!(values.get_int32s().unwrap(), &[1, 2]);

    let slice = [3i32, 4];
    let values = MultiValues::new(&slice[..]);
    assert_eq!(values.get_int32s().unwrap(), &[3, 4]);

    let vec_ref = vec![5i32, 6];
    let values = MultiValues::new(&vec_ref);
    assert_eq!(values.get_int32s().unwrap(), &[5, 6]);

    let values = MultiValues::new([7i32, 8]);
    assert_eq!(values.get_int32s().unwrap(), &[7, 8]);

    let array_ref = [9i32, 10];
    let array_ref_arg: &[i32; 2] = &array_ref;
    let values = MultiValues::new(array_ref_arg);
    assert_eq!(values.get_int32s().unwrap(), &[9, 10]);

    let values = MultiValues::new(vec!["a", "b"]);
    assert_eq!(values.get_strings().unwrap(), &["a", "b"]);

    let str_slice = ["c", "d"];
    let values = MultiValues::new(&str_slice[..]);
    assert_eq!(values.get_strings().unwrap(), &["c", "d"]);

    let str_vec_ref = vec!["e", "f"];
    let values = MultiValues::new(&str_vec_ref);
    assert_eq!(values.get_strings().unwrap(), &["e", "f"]);

    let values = MultiValues::new(["g", "h"]);
    assert_eq!(values.get_strings().unwrap(), &["g", "h"]);

    let str_array_ref = ["i", "j"];
    let str_array_ref_arg: &[&str; 2] = &str_array_ref;
    let values = MultiValues::new(str_array_ref_arg);
    assert_eq!(values.get_strings().unwrap(), &["i", "j"]);
}

#[test]
fn test_generic_set_convenient_inputs_for_coverage() {
    let mut values = MultiValues::Empty(DataType::Int32);
    let vec_ref = vec![1i32, 2];
    values.set(&vec_ref).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2]);

    values.set([3i32, 4]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[3, 4]);

    let array_ref = [5i32, 6];
    values.set(&array_ref).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[5, 6]);

    let mut values = MultiValues::Empty(DataType::String);
    let owned_vec_ref = vec!["a".to_string(), "b".to_string()];
    values.set(&owned_vec_ref).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["a", "b"]);

    values.set(["c".to_string(), "d".to_string()]).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["c", "d"]);

    let owned_array_ref = ["e".to_string(), "f".to_string()];
    values.set(&owned_array_ref).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["e", "f"]);

    let str_vec_ref = vec!["g", "h"];
    values.set(&str_vec_ref).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["g", "h"]);

    values.set(["i", "j"]).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["i", "j"]);

    let str_array_ref = ["k", "l"];
    values.set(&str_array_ref).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["k", "l"]);
}

#[test]
fn test_generic_add_convenient_inputs_for_coverage() {
    let mut values = MultiValues::Int32(vec![1]);
    let vec_ref = vec![2i32, 3];
    values.add(&vec_ref).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3]);

    values.add([4i32, 5]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4, 5]);

    let array_ref = [6i32, 7];
    values.add(&array_ref).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4, 5, 6, 7]);

    let mut values = MultiValues::String(vec!["a".to_string()]);
    let owned_vec_ref = vec!["b".to_string(), "c".to_string()];
    values.add(&owned_vec_ref).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["a", "b", "c"]);

    values.add(["d".to_string(), "e".to_string()]).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["a", "b", "c", "d", "e"]);

    let owned_array_ref = ["f".to_string(), "g".to_string()];
    values.add(&owned_array_ref).unwrap();
    assert_eq!(
        values.get_strings().unwrap(),
        &["a", "b", "c", "d", "e", "f", "g"]
    );

    let str_vec_ref = vec!["h", "i"];
    values.add(&str_vec_ref).unwrap();
    assert_eq!(
        values.get_strings().unwrap(),
        &["a", "b", "c", "d", "e", "f", "g", "h", "i"]
    );

    values.add(["j", "k"]).unwrap();
    assert_eq!(
        values.get_strings().unwrap(),
        &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]
    );

    let str_array_ref = ["l", "m"];
    values.add(&str_array_ref).unwrap();
    assert_eq!(
        values.get_strings().unwrap(),
        &[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m"
        ]
    );
}

#[test]
fn test_into_value_default_inputs_for_coverage() {
    fn into_default<T, D>(default: D) -> T
    where
        D: IntoValueDefault<T>,
    {
        default.into_value_default()
    }

    let value: i64 = into_default(42i64);
    assert_eq!(value, 42);

    let value: String = into_default("fallback");
    assert_eq!(value, "fallback");

    let owned = "owned".to_string();
    let value: String = into_default(&owned);
    assert_eq!(value, "owned");

    let slice = [1i32, 2];
    let value: Vec<i32> = into_default(&slice[..]);
    assert_eq!(value, vec![1, 2]);

    let vec_ref = vec![3i32, 4];
    let value: Vec<i32> = into_default(&vec_ref);
    assert_eq!(value, vec![3, 4]);

    let value: Vec<i32> = into_default([5i32, 6]);
    assert_eq!(value, vec![5, 6]);

    let array_ref = [7i32, 8];
    let array_ref_arg: &[i32; 2] = &array_ref;
    let value = <&[i32; 2] as IntoValueDefault<Vec<i32>>>::into_value_default(array_ref_arg);
    assert_eq!(value, vec![7, 8]);

    let value: Vec<String> = into_default(vec!["a", "b"]);
    assert_eq!(value, vec!["a".to_string(), "b".to_string()]);

    let str_slice = ["c", "d"];
    let value: Vec<String> = into_default(&str_slice[..]);
    assert_eq!(value, vec!["c".to_string(), "d".to_string()]);

    let str_vec_ref = vec!["e", "f"];
    let value: Vec<String> = into_default(&str_vec_ref);
    assert_eq!(value, vec!["e".to_string(), "f".to_string()]);

    let value: Vec<String> = into_default(["g", "h"]);
    assert_eq!(value, vec!["g".to_string(), "h".to_string()]);

    let str_array_ref = ["i", "j"];
    let str_array_ref_arg: &[&str; 2] = &str_array_ref;
    let value =
        <&[&str; 2] as IntoValueDefault<Vec<String>>>::into_value_default(str_array_ref_arg);
    assert_eq!(value, vec!["i".to_string(), "j".to_string()]);
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

    let mut mv = MultiValues::Empty(DataType::UInt8);
    mv.set(42u8).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![42u8]);
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

    let mut mv = MultiValues::UInt8(vec![5u8]);
    mv.add(6u8).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![5u8, 6u8]);
}

#[test]
fn test_generic_add_vec_all_types() {
    let mut mv = MultiValues::UInt16(vec![1u16]);
    mv.add(vec![2u16, 3]).unwrap();
    assert_eq!(mv.get_uint16s().unwrap(), &[1, 2, 3]);

    let mut mv = MultiValues::UInt8(vec![1u8]);
    mv.add(vec![2u8, 3]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3]);
}

#[test]
fn test_generic_add_slice_all_types() {
    let mut mv = MultiValues::Float32(vec![1.0f32]);
    let more = [2.0f32, 3.0f32];
    mv.add(&more[..]).unwrap();
    assert_eq!(mv.get_float32s().unwrap(), &[1.0, 2.0, 3.0]);

    let mut mv = MultiValues::UInt8(vec![1u8]);
    let more = [2u8, 3u8, 4u8];
    mv.add(&more[..]).unwrap();
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2, 3, 4]);
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

    let mv = MultiValues::UInt8(vec![1u8, 2u8]);
    assert_eq!(mv.get::<u8>().unwrap(), vec![1u8, 2u8]);
}

#[test]
fn test_generic_get_first_all_types() {
    let mv = MultiValues::UInt32(vec![10u32, 20u32]);
    let first: u32 = mv.get_first().unwrap();
    assert_eq!(first, 10);

    let mv = MultiValues::String(vec!["x".to_string(), "y".to_string()]);
    let first: String = mv.get_first().unwrap();
    assert_eq!(first, "x");

    let mv = MultiValues::UInt8(vec![11u8, 22u8]);
    assert_eq!(mv.get_first::<u8>().unwrap(), 11u8);
}
