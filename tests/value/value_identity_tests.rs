// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Utc;
use num_bigint::BigInt;
#[cfg(feature = "json")]
use qubit_budget::BudgetError;
#[cfg(feature = "json")]
use qubit_budget::JsonLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonResource;
use qubit_datatype::DataType;
use qubit_value::Value;
use url::Url;

/// Returns the standard-library hash of `value` for equality-contract tests.
fn hash(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Verifies canonical signed-zero and NaN identity for both float widths.
#[test]
fn test_value_float_identity_is_reflexive_and_hash_consistent() {
    assert_eq!(Value::Float32(-0.0), Value::Float32(0.0));
    assert_eq!(hash(&Value::Float32(-0.0)), hash(&Value::Float32(0.0)));
    assert_eq!(Value::Float64(-0.0), Value::Float64(0.0));
    assert_eq!(hash(&Value::Float64(-0.0)), hash(&Value::Float64(0.0)));

    let f32_left = Value::Float32(f32::from_bits(0x7fc0_0001));
    let f32_right = Value::Float32(f32::from_bits(0x7fff_ffff));
    assert_eq!(f32_left, f32_left);
    assert_eq!(f32_left, f32_right);
    assert_eq!(hash(&f32_left), hash(&f32_right));

    let f64_left = Value::Float64(f64::from_bits(0x7ff8_0000_0000_0001));
    let f64_right = Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff));
    assert_eq!(f64_left, f64_left);
    assert_eq!(f64_left, f64_right);
    assert_eq!(hash(&f64_left), hash(&f64_right));
    assert_ne!(Value::Float32(f32::NAN), Value::Float64(f64::NAN));
}

/// Verifies map and JSON object identity is independent of insertion order.
#[cfg(feature = "json")]
#[test]
fn test_value_map_and_json_identity_is_order_independent() {
    let left_map = HashMap::from([
        ("second".to_owned(), "2".to_owned()),
        ("first".to_owned(), "1".to_owned()),
    ]);
    let right_map = HashMap::from([
        ("first".to_owned(), "1".to_owned()),
        ("second".to_owned(), "2".to_owned()),
    ]);
    let left = Value::StringMap(left_map);
    let right = Value::StringMap(right_map);
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));

    let left = Value::Json(
        serde_json::from_str(r#"{"b":{"y":2,"x":1},"a":0}"#).unwrap(),
    );
    let right = Value::Json(
        serde_json::from_str(r#"{"a":0,"b":{"x":1,"y":2}}"#).unwrap(),
    );
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));
    assert_ne!(
        Value::Json(serde_json::json!([1, 2])),
        Value::Json(serde_json::json!([2, 1]))
    );
}

/// Verifies variants remain representation-distinct and usable as hash keys.
#[test]
fn test_value_variant_identity_supports_hash_collections() {
    assert_ne!(Value::Int32(1), Value::Int64(1));
    assert_ne!(Value::Int32(1), Value::Float64(1.0));

    let values = HashSet::from([
        Value::Int32(1),
        Value::Int64(1),
        Value::Float64(f64::NAN),
        Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff)),
    ]);
    assert_eq!(values.len(), 3);
}

/// Exercises native equality and hashing for every payload variant.
#[test]
fn test_value_identity_covers_every_variant() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 16).unwrap();
    let time = NaiveTime::from_hms_nano_opt(12, 34, 56, 789).unwrap();
    let datetime = date.and_time(time);
    let values = vec![
        Value::new_unset(DataType::Bool),
        Value::Bool(true),
        Value::Char('x'),
        Value::Int8(-1),
        Value::Int16(-2),
        Value::Int32(-3),
        Value::Int64(-4),
        Value::Int128(-5),
        Value::UInt8(1),
        Value::UInt16(2),
        Value::UInt32(3),
        Value::UInt64(4),
        Value::UInt128(5),
        Value::Float32(1.25),
        Value::Float64(2.5),
        Value::BigInteger(BigInt::from(6)),
        Value::BigDecimal(BigDecimal::from(7)),
        Value::String("text".to_owned()),
        Value::Date(date),
        Value::Time(time),
        Value::DateTime(datetime),
        Value::Instant(DateTime::<Utc>::from_naive_utc_and_offset(
            datetime, Utc,
        )),
        Value::Duration(Duration::new(8, 9)),
        Value::new(Url::parse("https://example.com/path").unwrap()),
        Value::StringMap(HashMap::from([(
            "key".to_owned(),
            "value".to_owned(),
        )])),
        Value::Json(serde_json::json!({
            "items": [null, true, 42, "text", [], {}]
        })),
    ];

    for value in &values {
        assert_eq!(value, value);
        let _ = hash(value);
    }
    assert_ne!(
        Value::new_unset(DataType::Bool),
        Value::new_unset(DataType::String)
    );
}

/// Verifies equal decimal encodings use the same canonical hash.
#[test]
fn test_value_big_decimal_identity_normalizes_coefficient_and_scale() {
    let encodings = [
        BigDecimal::new(BigInt::from(1), 0),
        BigDecimal::new(BigInt::from(10), 1),
        BigDecimal::new(BigInt::from(10_000), 4),
    ];

    for value in &encodings {
        assert_eq!(value, &encodings[0]);
        assert_eq!(
            hash(&Value::BigDecimal(value.clone())),
            hash(&Value::BigDecimal(encodings[0].clone())),
        );
    }

    let values: HashSet<_> =
        encodings.into_iter().map(Value::BigDecimal).collect();
    assert_eq!(values.len(), 1);
}

/// Verifies zero and extreme scales never trigger scale-sized hashing work.
#[test]
fn test_value_big_decimal_hash_handles_extreme_scales() {
    let zero_min =
        Value::BigDecimal(BigDecimal::new(BigInt::from(0), i64::MIN));
    let zero_max =
        Value::BigDecimal(BigDecimal::new(BigInt::from(0), i64::MAX));
    assert_eq!(zero_min, zero_max);
    assert_eq!(hash(&zero_min), hash(&zero_max));

    let positive =
        Value::BigDecimal(BigDecimal::new(BigInt::from(1), i64::MIN));
    let negative =
        Value::BigDecimal(BigDecimal::new(BigInt::from(-1), i64::MIN));
    assert_eq!(positive, positive);
    assert_eq!(negative, negative);
    let _ = hash(&positive);
    let _ = hash(&negative);
}

/// Verifies budgeted JSON hashing reports an exhausted node budget.
#[cfg(feature = "json")]
#[test]
fn test_value_hash_with_json_budget_rejects_json_exceeding_node_budget() {
    let value = Value::Json(serde_json::json!([null]));
    let mut budget = JsonLimits::new().with_max_nodes(1).budget();
    let mut state = DefaultHasher::new();

    let error = value
        .hash_with_json_budget(&mut state, &mut budget)
        .expect_err("the JSON node budget must reject the nested value");

    assert!(matches!(
        error,
        BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        }
    ));
}

/// Verifies budgeted hashing preserves special non-JSON identity hashes.
#[cfg(feature = "json")]
#[test]
fn test_value_hash_with_json_budget_matches_standard_hash_for_special_non_json_values()
 {
    let float = Value::Float32(-0.0);
    let string_map = Value::StringMap(HashMap::from([
        ("second".to_owned(), "2".to_owned()),
        ("first".to_owned(), "1".to_owned()),
    ]));
    let decimal = Value::BigDecimal(BigDecimal::new(BigInt::from(10), 1));

    for value in [&float, &string_map, &decimal] {
        let expected = hash(value);
        let mut budget = JsonLimits::new().budget();
        let mut state = DefaultHasher::new();

        value
            .hash_with_json_budget(&mut state, &mut budget)
            .expect("non-JSON values must not consume the JSON budget");

        assert_eq!(state.finish(), expected);
    }
}
