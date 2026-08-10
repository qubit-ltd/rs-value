// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Named Multi Values Unit Tests
//!
//! Tests various functionalities of the named multi values container。

use chrono::DateTime as UtcDateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireLimitKind;
use qubit_value::WireLimits;

/// Rejects schema fields outside the stable named-collection wrapper contract.
#[test]
fn test_named_multi_values_rejects_unknown_fields() {
    let input = r#"{"name":"ports","value":{"version":1,"value":{"collection":{"int32":[42]}}},"extra":true}"#;

    assert!(serde_json::from_str::<NamedMultiValues>(input).is_err());
}

/// Serializes named collections through the same V1 envelope as direct values.
#[test]
fn test_named_multi_values_serializes_with_v1_wire_contract() {
    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![42]));

    assert_eq!(
        serde_json::to_value(named).expect("named collection should serialize"),
        serde_json::json!({
            "name": "ports",
            "value": {"version": 1, "value": {"collection": {"int32": [42]}}},
        }),
    );
}

/// Applies one shared budget to the wrapper and every collection element.
#[test]
fn test_named_multi_values_bounded_decode_reuses_collection_budget() {
    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![42, 43]));
    let input = serde_json::to_vec(&named).expect("named collection should serialize");

    let error = NamedMultiValues::decode_json_slice_with_limits(
        &input,
        WireLimits::new(input.len()).with_max_nodes(3),
    )
    .expect_err("wrapper, collection, and two items should consume four nodes");

    assert!(matches!(
        error,
        ValueWireDecodeError::LimitExceeded {
            kind: ValueWireLimitKind::Nodes,
            value: 4,
            maximum: 3,
        }
    ));
}

#[test]
fn test_named_multi_values_identity_is_reflexive_with_nan() {
    let values = NamedMultiValues::new("samples", MultiValues::Float32(vec![f32::NAN]));
    assert_eq!(values, values);
}

#[test]
fn test_named_multi_value_creation() {
    let mut nmv = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081, 8082]));
    assert_eq!(nmv.name(), "ports");
    assert_eq!(nmv.values(), &MultiValues::Int32(vec![8080, 8081, 8082]));

    nmv.values_mut().add(8083).unwrap();
    assert_eq!(nmv.values().len(), 4);

    nmv.set_values(MultiValues::Bool(vec![true]));
    assert_eq!(nmv.values(), &MultiValues::Bool(vec![true]));
}

#[test]
fn test_named_multi_value_into_parts() {
    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081]));
    let (name, values) = named.into_parts();

    assert_eq!(name, "ports");
    assert_eq!(values, MultiValues::Int32(vec![8080, 8081]));
}

#[test]
fn test_named_multi_value_accessors() {
    let mut nmv = NamedMultiValues::new("servers", MultiValues::String(vec!["s1".to_string()]));
    assert_eq!(nmv.name(), "servers");
    assert_eq!(nmv.values().len(), 1);

    nmv.set_name("new_servers");
    assert_eq!(nmv.name(), "new_servers");

    nmv.set_values(MultiValues::String(vec![
        "s2".to_string(),
        "s3".to_string(),
    ]));
    assert_eq!(nmv.values().len(), 2);
}

#[test]
fn test_named_multi_value_mut() {
    let mut nmv = NamedMultiValues::new("numbers", MultiValues::Int32(vec![1, 2]));
    nmv.values_mut().add(3).unwrap();
    assert_eq!(nmv.values().len(), 3);
    assert_eq!(nmv.values().get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_named_value_to_named_multi_value() {
    let nv = NamedValue::new("single", Value::Int32(99));
    let nmv: NamedMultiValues = nv.into();
    assert_eq!(nmv.name(), "single");
    assert_eq!(nmv.values().len(), 1);
    assert_eq!(nmv.values().get_first_int32().unwrap(), 99);
}

#[test]
fn test_named_multi_value_consuming_conversion_reuses_owned_parts() {
    let named = NamedMultiValues::new("port", MultiValues::Int32(vec![8080, 8081]));
    let value = named.into_first_named_value();

    assert_eq!(value.name(), "port");
    assert_eq!(value.value().get_int32().unwrap(), 8080);
}

#[test]
fn test_named_multi_value_struct_access() {
    let nmv = NamedMultiValues::new(
        "items",
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
    );
    assert_eq!(nmv.name(), "items");
    assert_eq!(nmv.values().len(), 2);
}

// ===================== Basic properties and common methods
// =====================

#[test]
fn test_nmv_len_and_is_empty_and_clear() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1, 2, 3]));
    assert_eq!(nmv.values().len(), 3);
    assert_ne!(nmv.values().len(), 0);
    nmv.values_mut().clear();
    assert_eq!(nmv.values().len(), 0);
    assert_eq!(nmv.values().len(), 0);
}

#[test]
fn test_nmv_data_type_and_set_type() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    assert_eq!(nmv.values().data_type(), DataType::Int32);
    nmv.values_mut().set_type(DataType::String);
    assert_eq!(nmv.values().len(), 0);
    assert_eq!(nmv.values().data_type(), DataType::String);
}

// ===================== Generic get<T>() coverage =====================

#[test]
fn test_nmv_get_i32_list() {
    let nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1, 2, 3]));
    let v: Vec<i32> = nmv.values().get().unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn test_nmv_get_string_list() {
    let nmv = NamedMultiValues::new(
        "s",
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
    );
    let v: Vec<String> = nmv.values().get().unwrap();
    assert_eq!(v, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_nmv_get_dates() {
    let nmv = NamedMultiValues::new(
        "d",
        MultiValues::Date(vec![NaiveDate::from_ymd_opt(2020, 1, 2).unwrap()]),
    );
    let v: Vec<NaiveDate> = nmv.values().get().unwrap();
    assert_eq!(v, vec![NaiveDate::from_ymd_opt(2020, 1, 2).unwrap()]);
}

#[test]
fn test_nmv_get_times() {
    let nmv = NamedMultiValues::new(
        "t",
        MultiValues::Time(vec![NaiveTime::from_hms_milli_opt(1, 2, 3, 4).unwrap()]),
    );
    let v: Vec<NaiveTime> = nmv.values().get().unwrap();
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
    let v: Vec<NaiveDateTime> = nmv.values().get().unwrap();
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
    let now: UtcDateTime<Utc> =
        UtcDateTime::from_timestamp(1_700_000_000, 0).expect("fixed test instant must be valid");
    let nmv = NamedMultiValues::new("inst", MultiValues::Instant(vec![now]));
    let v: Vec<UtcDateTime<Utc>> = nmv.values().get().unwrap();
    assert_eq!(v, vec![now]);
}

// ===================== Generic get_first<T>() coverage =====================

#[test]
fn test_nmv_get_first_i32() {
    let nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![7, 8]));
    let first: i32 = nmv.values().get_first().unwrap();
    assert_eq!(first, 7);
}

#[test]
fn test_nmv_get_first_string() {
    let nmv = NamedMultiValues::new(
        "s",
        MultiValues::String(vec!["x".to_string(), "y".to_string()]),
    );
    let first: String = nmv.values().get_first().unwrap();
    assert_eq!(first, "x");
}

// ===================== Generic set<T,S>() coverage (Vec<T> / &[T] / single T)
// =====================

#[test]
fn test_nmv_set_vec_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Unset(DataType::Int32));
    nmv.values_mut().set(vec![1, 2, 3]);
    assert_eq!(nmv.values().get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_set_slice_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Unset(DataType::Int32));
    let s = &[4, 5, 6][..];
    nmv.values_mut().set(s);
    assert_eq!(nmv.values().get_int32s().unwrap(), &[4, 5, 6]);
}

#[test]
fn test_nmv_set_single_i32() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Unset(DataType::Int32));
    nmv.values_mut().set(7);
    assert_eq!(nmv.values().get_int32s().unwrap(), &[7]);
}

#[test]
fn test_nmv_set_vec_string() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::Unset(DataType::String));
    nmv.values_mut().set(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(nmv.values().get_strings().unwrap(), &["a", "b"]);
}

// ===================== Generic add<T,S>() coverage (T / Vec<T> / &[T])
// =====================

#[test]
fn test_nmv_add_i32_single() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    nmv.values_mut().add(2).unwrap();
    assert_eq!(nmv.values().get_int32s().unwrap(), &[1, 2]);
}

#[test]
fn test_nmv_add_i32_vec() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    nmv.values_mut().add(vec![2, 3]).unwrap();
    assert_eq!(nmv.values().get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_add_i32_slice() {
    let mut nmv = NamedMultiValues::new("n", MultiValues::Int32(vec![1]));
    let s = &[2, 3][..];
    nmv.values_mut().add(s).unwrap();
    assert_eq!(nmv.values().get_int32s().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_nmv_add_string_single() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::String(vec!["a".to_string()]));
    nmv.values_mut().add("b".to_string()).unwrap();
    assert_eq!(nmv.values().get_strings().unwrap(), &["a", "b"]);
}

#[test]
fn test_nmv_add_string_vec() {
    let mut nmv = NamedMultiValues::new("s", MultiValues::String(vec!["a".to_string()]));
    nmv.values_mut()
        .add(vec!["b".to_string(), "c".to_string()])
        .unwrap();
    assert_eq!(nmv.values().get_strings().unwrap(), &["a", "b", "c"]);
}

#[test]
fn test_named_multi_values_first_named_value_non_empty() {
    let nmv = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081]));
    let named = nmv.first_named_value();
    assert_eq!(named.name(), "ports");
    assert_eq!(named.value().get_int32().unwrap(), 8080);
}

#[test]
fn test_named_multi_values_first_named_value_empty_preserves_type() {
    let nmv = NamedMultiValues::new("threshold", MultiValues::Unset(DataType::Float64));
    let named = nmv.first_named_value();
    assert_eq!(named.name(), "threshold");
    assert_eq!(named.value().data_type(), DataType::Float64);
    assert!(matches!(
        named.value().get_float64(),
        Err(ValueError::Missing(_))
    ));
}

#[test]
fn test_named_multi_values_empty_get_mismatched_type_returns_error() {
    let nmv = NamedMultiValues::new("ports", MultiValues::Unset(DataType::Int32));
    assert!(matches!(
        nmv.values().get_strings(),
        Err(ValueError::TypeMismatch { .. })
    ));
}
