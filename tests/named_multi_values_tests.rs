/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Named Multi Values Unit Tests
//!
//! Tests various functionalities of the named multi values container。
//!
//! # Author
//!
//! Haixing Hu

use chrono::DateTime as UtcDateTime;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use qubit_common::lang::DataType;
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value};

#[test]
fn test_named_multi_value_creation() {
    let nmv = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081, 8082]));
    assert_eq!(nmv.name(), "ports");
    assert_eq!(nmv.count(), 3);
}

#[test]
fn test_named_multi_value_accessors() {
    let mut nmv = NamedMultiValues::new("servers", MultiValues::String(vec!["s1".to_string()]));
    assert_eq!(nmv.name(), "servers");
    assert_eq!(nmv.count(), 1);

    nmv.set_name("new_servers");
    assert_eq!(nmv.name(), "new_servers");

    *nmv = MultiValues::String(vec!["s2".to_string(), "s3".to_string()]);
    assert_eq!(nmv.count(), 2);
}

#[test]
fn test_named_multi_value_mut() {
    let mut nmv = NamedMultiValues::new("numbers", MultiValues::Int32(vec![1, 2]));
    nmv.add_int32(3).unwrap();
    assert_eq!(nmv.count(), 3);
    assert_eq!(nmv.get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_named_value_to_named_multi_value() {
    let nv = NamedValue::new("single", Value::Int32(99));
    let nmv: NamedMultiValues = nv.into();
    assert_eq!(nmv.name(), "single");
    assert_eq!(nmv.count(), 1);
    assert_eq!(nmv.get_first_int32().unwrap(), 99);
}

#[test]
fn test_named_multi_value_struct_access() {
    let nmv = NamedMultiValues::new(
        "items",
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
    );
    assert_eq!(nmv.name(), "items");
    assert_eq!(nmv.count(), 2);
}

// ===================== Basic properties and common methods =====================

#[test]
fn test_nmv_count_and_is_empty_and_clear() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1, 2, 3]));
    assert_eq!(nmv.count(), 3);
    assert!(!nmv.is_empty());
    nmv.clear();
    assert_eq!(nmv.count(), 0);
    assert!(nmv.is_empty());
}

#[test]
fn test_nmv_data_type_and_set_type() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    assert_eq!(nmv.data_type(), DataType::Int32);
    nmv.set_type(DataType::String);
    assert!(nmv.is_empty());
    assert_eq!(nmv.data_type(), DataType::String);
}

// ===================== Generic get<T>() coverage =====================

#[test]
fn test_nmv_get_i32_list() {
    let nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1, 2, 3]));
    let v: Vec<i32> = nmv.get().unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn test_nmv_get_string_list() {
    let nmv = NamedMultiValues::new(
        "s",
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
    );
    let v: Vec<String> = nmv.get().unwrap();
    assert_eq!(v, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_nmv_get_dates() {
    let nmv = NamedMultiValues::new(
        "d",
        MultiValues::Date(vec![NaiveDate::from_ymd_opt(2020, 1, 2).unwrap()]),
    );
    let v: Vec<NaiveDate> = nmv.get().unwrap();
    assert_eq!(v, vec![NaiveDate::from_ymd_opt(2020, 1, 2).unwrap()]);
}

#[test]
fn test_nmv_get_times() {
    let nmv = NamedMultiValues::new(
        "t",
        MultiValues::Time(vec![NaiveTime::from_hms_milli_opt(1, 2, 3, 4).unwrap()]),
    );
    let v: Vec<NaiveTime> = nmv.get().unwrap();
    assert_eq!(v, vec![NaiveTime::from_hms_milli_opt(1, 2, 3, 4).unwrap()]);
}

#[test]
fn test_nmv_get_datetimes() {
    let nmv = NamedMultiValues::new(
        "dt",
        MultiValues::DateTime(vec![NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
            NaiveTime::from_hms_opt(3, 4, 5).unwrap(),
        )]),
    );
    let v: Vec<NaiveDateTime> = nmv.get().unwrap();
    assert_eq!(
        v,
        vec![NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
            NaiveTime::from_hms_opt(3, 4, 5).unwrap(),
        )]
    );
}

#[test]
fn test_nmv_get_instants() {
    let now: UtcDateTime<Utc> = Utc::now();
    let nmv = NamedMultiValues::new("inst", MultiValues::Instant(vec![now]));
    let v: Vec<UtcDateTime<Utc>> = nmv.get().unwrap();
    assert_eq!(v, vec![now]);
}

// ===================== Generic get_first<T>() coverage =====================

#[test]
fn test_nmv_get_first_i32() {
    let nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![7, 8]));
    let first: i32 = nmv.get_first().unwrap();
    assert_eq!(first, 7);
}

#[test]
fn test_nmv_get_first_string() {
    let nmv = NamedMultiValues::new(
        "s",
        MultiValues::String(vec!["x".to_string(), "y".to_string()]),
    );
    let first: String = nmv.get_first().unwrap();
    assert_eq!(first, "x");
}

// ===================== Generic set<T,S>() coverage (Vec<T> / &[T] / single T) =====================

#[test]
fn test_nmv_set_vec_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Empty(DataType::Int32));
    nmv.set(vec![1, 2, 3]).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_set_slice_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Empty(DataType::Int32));
    let s = &[4, 5, 6][..];
    nmv.set(s).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[4, 5, 6]);
}

#[test]
fn test_nmv_set_single_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Empty(DataType::Int32));
    nmv.set(7).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[7]);
}

#[test]
fn test_nmv_set_vec_string() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::Empty(DataType::String));
    nmv.set(vec!["a".to_string(), "b".to_string()]).unwrap();
    assert_eq!(nmv.get_strings().unwrap(), &["a", "b"]);
}

// ===================== Generic add<T,S>() coverage (T / Vec<T> / &[T]) =====================

#[test]
fn test_nmv_add_i32_single() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    nmv.add(2).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[1, 2]);
}

#[test]
fn test_nmv_add_i32_vec() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    nmv.add(vec![2, 3]).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_add_i32_slice() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    let s = &[2, 3][..];
    nmv.add(s).unwrap();
    assert_eq!(nmv.get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_add_string_single() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::String(vec!["a".to_string()]));
    nmv.add("b".to_string()).unwrap();
    assert_eq!(nmv.get_strings().unwrap(), &["a", "b"]);
}

#[test]
fn test_nmv_add_string_vec() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::String(vec!["a".to_string()]));
    nmv.add(vec!["b".to_string(), "c".to_string()]).unwrap();
    assert_eq!(nmv.get_strings().unwrap(), &["a", "b", "c"]);
}

#[test]
fn test_named_multi_values_to_named_value_non_empty() {
    let nmv = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081]));
    let named = nmv.to_named_value();
    assert_eq!(named.name(), "ports");
    assert_eq!(named.get_int32().unwrap(), 8080);
}

#[test]
fn test_named_multi_values_to_named_value_empty_preserves_type() {
    let nmv = NamedMultiValues::new("threshold", MultiValues::Empty(DataType::Float64));
    let named = nmv.to_named_value();
    assert_eq!(named.name(), "threshold");
    assert_eq!(named.data_type(), DataType::Float64);
    assert!(matches!(
        named.get_float64(),
        Err(qubit_value::ValueError::NoValue)
    ));
}
