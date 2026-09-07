// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Public lower-bound admission before bounded wire serialization.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::TimeZone;
use chrono::Utc;
use num_bigint::BigInt;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireEncodePreflight;
use serde_json::json;
use url::Url;

/// Every supported scalar shape can pass conservative preparation with room.
#[test]
fn test_preflight_accepts_supported_scalar_representations() {
    let date = NaiveDate::from_ymd_opt(2026, 9, 7).unwrap();
    let time = NaiveTime::from_hms_opt(12, 34, 56).unwrap();
    let values = [
        Value::Unset(DataType::String),
        Value::Bool(true),
        Value::Char('界'),
        Value::String("visible".into()),
        Value::Int8(i8::MIN),
        Value::Int16(i16::MIN),
        Value::Int32(i32::MIN),
        Value::Int64(i64::MIN),
        Value::Int128(i128::MIN),
        Value::UInt8(u8::MAX),
        Value::UInt16(u16::MAX),
        Value::UInt32(u32::MAX),
        Value::UInt64(u64::MAX),
        Value::UInt128(u128::MAX),
        Value::Float32(1.5),
        Value::Float64(-1.5),
        Value::BigInteger(BigInt::from(0)),
        Value::BigInteger(BigInt::from(-123)),
        Value::BigDecimal(BigDecimal::from_str("-123.45").unwrap()),
        Value::Duration(Duration::new(1, 2)),
        Value::Date(date),
        Value::Time(time),
        Value::DateTime(date.and_time(time)),
        Value::Instant(Utc.with_ymd_and_hms(2026, 9, 7, 12, 34, 56).unwrap()),
        Value::Url(Url::parse("https://example.invalid/path").unwrap()),
        Value::StringMap(HashMap::from([("key".into(), "value".into())])),
        Value::Json(json!({"all": [null, true, 42, "text", {"nested": []}]})),
    ];
    for value in values {
        ValueWireEncodePreflight::new(JsonEncodeLimits::new())
            .check_container(&ValueContainer::Scalar(value))
            .expect("supported scalar must fit unlimited preparation");
    }
}

/// Homogeneous collections retain their distinct admission paths.
#[test]
fn test_preflight_accepts_collection_shapes() {
    for values in [
        MultiValues::Bool(vec![true, false]),
        MultiValues::Char(vec!['界', 'a']),
        MultiValues::String(vec!["one".into(), "two".into()]),
        MultiValues::Int64(vec![1, -2]),
        MultiValues::StringMap(vec![HashMap::from([("key".into(), "value".into())])]),
        MultiValues::Json(vec![json!([1, null]), json!({"key": false})]),
    ] {
        ValueWireEncodePreflight::new(JsonEncodeLimits::new())
            .check_container(&ValueContainer::Collection(values))
            .expect("supported collection must fit unlimited preparation");
    }
}

/// Lower-bound cumulative rejection preserves resource and observation kind.
#[test]
fn test_preflight_reports_cumulative_lower_bounds_and_wire_error_conversion() {
    for (limits, resource, maximum) in [
        (JsonEncodeLimits::builder().max_nodes(1).build(), JsonResource::Nodes, 1),
        (
            JsonEncodeLimits::builder().max_payload_bytes(1).build(),
            JsonResource::PayloadBytes,
            1,
        ),
        (
            JsonEncodeLimits::builder().max_output_bytes(3).build(),
            JsonResource::OutputBytes,
            3,
        ),
    ] {
        let mut checker = ValueWireEncodePreflight::new(limits);
        checker
            .check_value(&Value::String("a".into()))
            .expect("first item fits exactly");
        let error = checker
            .check_value(&Value::Bool(true))
            .expect_err("shared ledger must reject second item");
        assert!(
            matches!(&error, MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: actual, observed: Observation::AtLeast(_), maximum: bound,
        }) if *actual == resource && *bound == maximum)
        );
        assert!(matches!(
            ValueWireEncodeError::from(error),
            ValueWireEncodeError::Budget(_)
        ));
    }
}

/// Per-value byte and structural limits reject before expensive preparation.
#[test]
fn test_preflight_rejects_point_limits_for_strings_numbers_and_containers() {
    for (limits, value) in [
        (
            JsonEncodeLimits::builder().max_string_bytes(2).build(),
            Value::Char('界'),
        ),
        (
            JsonEncodeLimits::builder().max_number_bytes(1).build(),
            Value::Int32(-12),
        ),
        (
            JsonEncodeLimits::builder().max_key_bytes(2).build(),
            Value::StringMap(HashMap::from([("long".into(), "x".into())])),
        ),
        (
            JsonEncodeLimits::builder().max_map_entries(0).build(),
            Value::Duration(Duration::ZERO),
        ),
        (
            JsonEncodeLimits::builder().max_depth(1).build(),
            Value::Json(json!(["child"])),
        ),
        (
            JsonEncodeLimits::builder().max_sequence_items(0).build(),
            Value::Json(json!([true])),
        ),
    ] {
        assert!(ValueWireEncodePreflight::new(limits).check_value(&value).is_err());
    }
    assert!(
        ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_sequence_items(1).build())
            .check_values(&MultiValues::Bool(vec![true, false]))
            .is_err()
    );
}

/// Native and u64 construction preserve configured limits independently.
#[test]
fn test_preflight_constructors_preserve_value_and_u64_profiles() {
    let value_limits = JsonValueLimits::builder().max_string_bytes(2).build();
    assert!(
        ValueWireEncodePreflight::new_value_limits(value_limits)
            .check_value(&Value::String("abc".into()))
            .is_err()
    );
    let profile = JsonEncodeLimits::<JsonResource, u64>::builder()
        .max_output_bytes(128)
        .max_depth(8)
        .max_nodes(16)
        .max_sequence_items(8)
        .max_map_entries(8)
        .max_key_bytes(8)
        .max_string_bytes(2)
        .max_number_bytes(8)
        .max_payload_bytes(64)
        .build();
    let mut checker = ValueWireEncodePreflight::new_u64_limits(profile);
    checker
        .check_value(&Value::String("ab".into()))
        .expect("exact string limit");
    assert!(checker.check_value(&Value::String("abc".into())).is_err());
    ValueWireEncodePreflight::new_u64_limits(JsonEncodeLimits::new())
        .check_value(&Value::String("unlimited".into()))
        .expect("absent limits stay absent");
}
