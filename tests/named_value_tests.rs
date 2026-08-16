// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Named Single Value Unit Tests
//!
//! Tests various functionalities of the named single value container。

use chrono::DateTime as UtcDateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_datatype::DataType;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueWireDecodeError;

/// Rejects schema fields outside the stable named-value wrapper contract.
#[test]
fn test_named_value_rejects_unknown_fields() {
    let input = r#"{"name":"port","value":{"version":1,"value":{"scalar":{"int32":42}}},"extra":true}"#;

    assert!(serde_json::from_str::<NamedValue>(input).is_err());
}

/// Serializes named values through the same V1 envelope as direct values.
#[test]
fn test_named_value_serializes_with_v1_wire_contract() {
    let named = NamedValue::new("port", Value::Int32(42));

    assert_eq!(
        serde_json::to_value(named).expect("named value should serialize"),
        serde_json::json!({
            "name": "port",
            "value": {"version": 1, "value": {"scalar": {"int32": 42}}},
        }),
    );
}

/// Applies one shared budget to the wrapper name and nested scalar value.
#[test]
fn test_named_value_bounded_decode_reuses_value_budget() {
    let named = NamedValue::new("port", Value::Int32(42));
    let input =
        serde_json::to_vec(&named).expect("named value should serialize");

    let name_error = NamedValue::decode_json_slice_with_limits(
        &input,
        JsonDecodeLimits::builder()
            .max_input_bytes(input.len())
            .max_string_bytes(3)
            .build(),
    )
    .expect_err("the name should exceed the string limit");
    assert!(matches!(
        name_error,
        ValueWireDecodeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(4),
            maximum: 3,
        })
    ));

    let node_error = NamedValue::decode_json_slice_with_limits(
        &input,
        JsonDecodeLimits::builder()
            .max_input_bytes(input.len())
            .max_nodes(1)
            .build(),
    )
    .expect_err("the wrapper and scalar should consume two nodes");
    assert!(matches!(
        node_error,
        ValueWireDecodeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            requested: 1,
            ..
        })
    ));
}

#[test]
fn test_named_value_default_encoding_round_trips() {
    let named = NamedValue::new("port", Value::Int32(42));
    let encoded = named
        .to_json_vec()
        .expect("default limits should encode named value");

    assert_eq!(
        NamedValue::decode_json_slice(&encoded)
            .expect("default limits should decode named value"),
        named
    );
}

#[test]
fn test_named_value_identity_includes_name() {
    assert_ne!(
        NamedValue::new("left", Value::Float64(f64::NAN)),
        NamedValue::new("right", Value::Float64(f64::NAN)),
    );
    assert_eq!(
        NamedValue::new("same", Value::Float64(f64::NAN)),
        NamedValue::new("same", Value::Float64(f64::NAN)),
    );
}

#[test]
fn test_named_value_new() {
    let mut nv = NamedValue::new("port", Value::Int32(8080));
    assert_eq!(nv.name(), "port");
    assert_eq!(nv.value(), &Value::Int32(8080));

    nv.value_mut().set(8081);
    assert_eq!(nv.value(), &Value::Int32(8081));

    nv.set_value(Value::String("ready".to_owned()));
    assert_eq!(nv.value(), &Value::String("ready".to_owned()));
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
fn test_named_value_set_value() {
    let mut nv = NamedValue::new("counter", Value::Int32(0));
    nv.set_value(Value::Int32(42));
    assert_eq!(nv.value().get_int32().unwrap(), 42);
}

// ------------------- Individual get_xxx() method coverage -------------------

#[test]
fn test_named_value_get_bool() {
    let nv = NamedValue::new("b", Value::Bool(true));
    assert!(nv.value().get_bool().unwrap());
}

#[test]
fn test_named_value_get_char() {
    let nv = NamedValue::new("c", Value::Char('A'));
    assert_eq!(nv.value().get_char().unwrap(), 'A');
}

#[test]
fn test_named_value_get_int8() {
    let nv = NamedValue::new("i8", Value::Int8(-8));
    assert_eq!(nv.value().get_int8().unwrap(), -8);
}

#[test]
fn test_named_value_get_int16() {
    let nv = NamedValue::new("i16", Value::Int16(-16));
    assert_eq!(nv.value().get_int16().unwrap(), -16);
}

#[test]
fn test_named_value_get_int32() {
    let nv = NamedValue::new("i32", Value::Int32(-32));
    assert_eq!(nv.value().get_int32().unwrap(), -32);
}

#[test]
fn test_named_value_get_int64() {
    let nv = NamedValue::new("i64", Value::Int64(-64));
    assert_eq!(nv.value().get_int64().unwrap(), -64);
}

#[test]
fn test_named_value_get_int128() {
    let nv = NamedValue::new("i128", Value::Int128(-128));
    assert_eq!(nv.value().get_int128().unwrap(), -128);
}

#[test]
fn test_named_value_get_uint8() {
    let nv = NamedValue::new("u8", Value::UInt8(8));
    assert_eq!(nv.value().get_uint8().unwrap(), 8);
}

#[test]
fn test_named_value_get_uint16() {
    let nv = NamedValue::new("u16", Value::UInt16(16));
    assert_eq!(nv.value().get_uint16().unwrap(), 16);
}

#[test]
fn test_named_value_get_uint32() {
    let nv = NamedValue::new("u32", Value::UInt32(32));
    assert_eq!(nv.value().get_uint32().unwrap(), 32);
}

#[test]
fn test_named_value_get_uint64() {
    let nv = NamedValue::new("u64", Value::UInt64(64));
    assert_eq!(nv.value().get_uint64().unwrap(), 64);
}

#[test]
fn test_named_value_get_uint128() {
    let nv = NamedValue::new("u128", Value::UInt128(128));
    assert_eq!(nv.value().get_uint128().unwrap(), 128);
}

#[test]
fn test_named_value_get_float32() {
    let nv = NamedValue::new("f32", Value::Float32(1.5));
    assert_eq!(nv.value().get_float32().unwrap(), 1.5);
}

#[test]
fn test_named_value_get_float64() {
    let nv = NamedValue::new("f64", Value::Float64(2.5));
    assert_eq!(nv.value().get_float64().unwrap(), 2.5);
}

#[test]
fn test_named_value_get_string() {
    let nv = NamedValue::new("s", Value::String("hello".to_string()));
    assert_eq!(nv.value().get_string().unwrap(), "hello");
}

#[test]
fn test_named_value_get_date() {
    let nv = NamedValue::new(
        "d",
        Value::Date(NaiveDate::from_ymd_opt(2020, 5, 17).unwrap()),
    );
    assert_eq!(
        nv.value().get_date().unwrap(),
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
        nv.value().get_time().unwrap(),
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
        nv.value().get_datetime().unwrap(),
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2021, 6, 7).unwrap(),
            NaiveTime::from_hms_opt(8, 9, 10).unwrap(),
        )
    );
}

#[test]
fn test_named_value_get_instant() {
    let inst: UtcDateTime<Utc> = UtcDateTime::from_timestamp(1_700_000_000, 0)
        .expect("fixed test instant must be valid");
    let nv = NamedValue::new("inst", Value::Instant(inst));
    assert_eq!(nv.value().get_instant().unwrap(), inst);
}

// ------------------- Generic set()/get<T>() coverage for each type
// -------------------

#[test]
fn test_named_value_set_get_bool() {
    let mut nv = NamedValue::new("b", Value::Bool(false));
    nv.value_mut().set(true);
    let b: bool = nv.value().get().unwrap();
    assert!(b);
}

#[test]
fn test_named_value_set_get_char() {
    let mut nv = NamedValue::new("c", Value::Char('x'));
    nv.value_mut().set('A');
    let c: char = nv.value().get().unwrap();
    assert_eq!(c, 'A');
}

#[test]
fn test_named_value_set_get_i8() {
    let mut nv = NamedValue::new("i8", Value::Int8(0));
    nv.value_mut().set(-8i8);
    let v: i8 = nv.value().get().unwrap();
    assert_eq!(v, -8);
}

#[test]
fn test_named_value_set_get_i16() {
    let mut nv = NamedValue::new("i16", Value::Int16(0));
    nv.value_mut().set(-16i16);
    let v: i16 = nv.value().get().unwrap();
    assert_eq!(v, -16);
}

#[test]
fn test_named_value_set_get_i32() {
    let mut nv = NamedValue::new("i32", Value::Int32(0));
    nv.value_mut().set(-32i32);
    let v: i32 = nv.value().get().unwrap();
    assert_eq!(v, -32);
}

#[test]
fn test_named_value_set_get_i64() {
    let mut nv = NamedValue::new("i64", Value::Int64(0));
    nv.value_mut().set(-64i64);
    let v: i64 = nv.value().get().unwrap();
    assert_eq!(v, -64);
}

#[test]
fn test_named_value_set_get_i128() {
    let mut nv = NamedValue::new("i128", Value::Int128(0));
    nv.value_mut().set(-128i128);
    let v: i128 = nv.value().get().unwrap();
    assert_eq!(v, -128);
}

#[test]
fn test_named_value_set_get_u8() {
    let mut nv = NamedValue::new("u8", Value::UInt8(0));
    nv.value_mut().set(8u8);
    let v: u8 = nv.value().get().unwrap();
    assert_eq!(v, 8);
}

#[test]
fn test_named_value_set_get_u16() {
    let mut nv = NamedValue::new("u16", Value::UInt16(0));
    nv.value_mut().set(16u16);
    let v: u16 = nv.value().get().unwrap();
    assert_eq!(v, 16);
}

#[test]
fn test_named_value_set_get_u32() {
    let mut nv = NamedValue::new("u32", Value::UInt32(0));
    nv.value_mut().set(32u32);
    let v: u32 = nv.value().get().unwrap();
    assert_eq!(v, 32);
}

#[test]
fn test_named_value_set_get_u64() {
    let mut nv = NamedValue::new("u64", Value::UInt64(0));
    nv.value_mut().set(64u64);
    let v: u64 = nv.value().get().unwrap();
    assert_eq!(v, 64);
}

#[test]
fn test_named_value_set_get_u128() {
    let mut nv = NamedValue::new("u128", Value::UInt128(0));
    nv.value_mut().set(128u128);
    let v: u128 = nv.value().get().unwrap();
    assert_eq!(v, 128);
}

#[test]
fn test_named_value_set_get_f32() {
    let mut nv = NamedValue::new("f32", Value::Float32(0.0));
    nv.value_mut().set(1.5f32);
    let v: f32 = nv.value().get().unwrap();
    assert_eq!(v, 1.5);
}

#[test]
fn test_named_value_set_get_f64() {
    let mut nv = NamedValue::new("f64", Value::Float64(0.0));
    nv.value_mut().set(2.5f64);
    let v: f64 = nv.value().get().unwrap();
    assert_eq!(v, 2.5);
}

#[test]
fn test_named_value_set_get_string() {
    let mut nv = NamedValue::new("s", Value::String(String::new()));
    nv.value_mut().set("hello".to_string());
    let s: String = nv.value().get().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn test_named_value_set_get_date() {
    let mut nv = NamedValue::new(
        "d",
        Value::Date(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
    );
    let date = NaiveDate::from_ymd_opt(2020, 5, 17).unwrap();
    nv.value_mut().set(date);
    let got: NaiveDate = nv.value().get().unwrap();
    assert_eq!(got, date);
}

#[test]
fn test_named_value_set_get_time() {
    let mut nv = NamedValue::new(
        "t",
        Value::Time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
    );
    let time = NaiveTime::from_hms_milli_opt(13, 14, 15, 123).unwrap();
    nv.value_mut().set(time);
    let got: NaiveTime = nv.value().get().unwrap();
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
    nv.value_mut().set(dt);
    let got: NaiveDateTime = nv.value().get().unwrap();
    assert_eq!(got, dt);
}

#[test]
fn test_named_value_set_get_instant() {
    let inst: UtcDateTime<Utc> = UtcDateTime::from_timestamp(1_700_000_000, 0)
        .expect("fixed test instant must be valid");
    let mut nv = NamedValue::new("inst", Value::Instant(inst));
    nv.value_mut().set(inst);
    let got: UtcDateTime<Utc> = nv.value().get().unwrap();
    assert_eq!(got, inst);
}

// ------------------- Other general behaviors -------------------

#[test]
fn test_named_value_is_empty() {
    let nv = NamedValue::new("e", Value::Unset(DataType::Int32));
    assert!(nv.value().is_unset());
}

#[test]
fn test_named_value_unset() {
    let mut nv = NamedValue::new("e", Value::Int32(7));
    nv.value_mut().unset();
    assert!(nv.value().is_unset());
    assert_eq!(nv.value().data_type(), DataType::Int32);
}

#[test]
fn test_named_value_set_type() {
    let mut nv = NamedValue::new("e", Value::Int32(7));
    nv.value_mut().set_type(DataType::String);
    assert!(nv.value().is_unset());
    assert_eq!(nv.value().data_type(), DataType::String);
}

#[test]
fn test_named_value_data_type() {
    let nv = NamedValue::new("i32", Value::Int32(1));
    assert_eq!(nv.value().data_type(), DataType::Int32);
}
