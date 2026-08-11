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
use std::fmt::Debug;
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
use qubit_budget::JsonResource;
#[cfg(feature = "json")]
use qubit_budget::JsonValueLimits;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use url::Url;

#[cfg(feature = "json")]
use crate::json_budget_test_support_tests::JsonValueLimitsExt;

/// Returns the standard-library hash for equality-contract assertions.
///
/// # Parameters
///
/// * `value` - Value whose hash is calculated.
///
/// # Returns
///
/// The hash produced by [`DefaultHasher`].
fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Requires lawful equality and equal hashes for a representative pair.
///
/// # Parameters
///
/// * `left` - Left value in the equality comparison.
/// * `right` - Right value in the equality comparison.
fn assert_equal_hash<T>(left: &T, right: &T)
where
    T: Debug + Eq + Hash,
{
    assert_eq!(left, right);
    assert_eq!(hash(left), hash(right));
}

/// Verifies canonical float identity across multi-value elements.
#[test]
fn test_multi_values_float_identity_is_reflexive_and_hash_consistent() {
    assert_equal_hash(
        &MultiValues::Float32(vec![-0.0, f32::from_bits(0x7fc0_0001)]),
        &MultiValues::Float32(vec![0.0, f32::from_bits(0x7fff_ffff)]),
    );
    assert_equal_hash(
        &MultiValues::Float64(vec![
            -0.0,
            f64::from_bits(0x7ff8_0000_0000_0001),
        ]),
        &MultiValues::Float64(vec![0.0, f64::from_bits(0x7fff_ffff_ffff_ffff)]),
    );

    let mut cache = HashMap::new();
    cache.insert(MultiValues::Float64(vec![f64::NAN]), "first");
    cache.insert(
        MultiValues::Float64(vec![f64::from_bits(0x7fff_ffff_ffff_ffff)]),
        "replacement",
    );
    assert_eq!(cache.len(), 1);
    assert_eq!(cache[&MultiValues::Float64(vec![f64::NAN])], "replacement");
}

/// Verifies unset metadata, variant tags, and outer order remain significant.
#[test]
fn test_multi_values_unset_variant_and_order_remain_part_of_identity() {
    assert_ne!(
        MultiValues::Unset(DataType::Int32),
        MultiValues::Int32(Vec::new()),
    );
    assert_ne!(
        MultiValues::Unset(DataType::Int32),
        MultiValues::Unset(DataType::Int64),
    );
    assert_ne!(
        MultiValues::Int32(vec![1, 2]),
        MultiValues::Int32(vec![2, 1]),
    );
    assert_ne!(MultiValues::Int32(vec![1]), MultiValues::Int64(vec![1]),);
}

/// Verifies structurally unordered payloads receive order-independent hashes.
#[test]
fn test_multi_values_unordered_payloads_hash_structurally() {
    let left_map = HashMap::from([
        ("b".to_owned(), "2".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ]);
    let right_map = HashMap::from([
        ("a".to_owned(), "1".to_owned()),
        ("b".to_owned(), "2".to_owned()),
    ]);
    assert_equal_hash(
        &MultiValues::StringMap(vec![left_map]),
        &MultiValues::StringMap(vec![right_map]),
    );
    assert_equal_hash(
        &MultiValues::Json(vec![
            serde_json::json!({"b": {"y": 2, "x": 1}, "a": 0}),
        ]),
        &MultiValues::Json(vec![
            serde_json::json!({"a": 0, "b": {"x": 1, "y": 2}}),
        ]),
    );
}

/// Verifies equal decimal encodings share identity without scale-sized work.
#[test]
fn test_multi_values_big_decimal_identity_is_canonical() {
    assert_equal_hash(
        &MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(10), 1)]),
        &MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(1), 0)]),
    );
    let extreme = MultiValues::BigDecimal(vec![BigDecimal::new(
        BigInt::from(1),
        i64::MIN,
    )]);
    let _ = hash(&extreme);
}

/// Exercises equality and hashing for every multi-value variant.
#[test]
fn test_multi_values_identity_covers_every_variant() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 17)
        .expect("the test fixture date must be valid");
    let time = NaiveTime::from_hms_nano_opt(12, 34, 56, 789)
        .expect("the test fixture time must be valid");
    let datetime = date.and_time(time);
    let values = vec![
        MultiValues::Unset(DataType::Bool),
        MultiValues::Bool(vec![true]),
        MultiValues::Char(vec!['x']),
        MultiValues::Int8(vec![-1]),
        MultiValues::Int16(vec![-2]),
        MultiValues::Int32(vec![-3]),
        MultiValues::Int64(vec![-4]),
        MultiValues::Int128(vec![-5]),
        MultiValues::UInt8(vec![1]),
        MultiValues::UInt16(vec![2]),
        MultiValues::UInt32(vec![3]),
        MultiValues::UInt64(vec![4]),
        MultiValues::UInt128(vec![5]),
        MultiValues::Float32(vec![f32::NAN]),
        MultiValues::Float64(vec![f64::NAN]),
        MultiValues::BigInteger(vec![BigInt::from(6)]),
        MultiValues::BigDecimal(vec![BigDecimal::from(7)]),
        MultiValues::String(vec!["text".to_owned()]),
        MultiValues::Date(vec![date]),
        MultiValues::Time(vec![time]),
        MultiValues::DateTime(vec![datetime]),
        MultiValues::Instant(vec![DateTime::<Utc>::from_naive_utc_and_offset(
            datetime, Utc,
        )]),
        MultiValues::Duration(vec![Duration::new(8, 9)]),
        MultiValues::Url(vec![
            Url::parse("https://example.com/path")
                .expect("the test fixture URL must be valid"),
        ]),
        MultiValues::StringMap(vec![HashMap::from([(
            "key".to_owned(),
            "value".to_owned(),
        )])]),
        MultiValues::Json(vec![serde_json::json!({"items": [null, true, 42]})]),
    ];

    for value in &values {
        assert_eq!(value, value);
        let _ = hash(value);
    }
    let keys: HashSet<_> = values.into_iter().collect();
    assert_eq!(keys.len(), 26);
}

/// Verifies budgeted JSON hashing reports an exhausted node budget.
#[cfg(feature = "json")]
#[test]
fn test_multi_values_hash_with_json_budget_accumulates_json_node_budget() {
    let values = MultiValues::Json(vec![
        serde_json::json!(null),
        serde_json::json!(null),
    ]);
    let mut budget = JsonValueLimits::default().with_max_nodes(1).budget();
    let mut state = DefaultHasher::new();

    let error = values
        .hash_with_json_budget(&mut state, &mut budget)
        .expect_err(
            "the second JSON value must exhaust the shared node budget",
        );

    assert!(matches!(
        error,
        BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        }
    ));

    let follow_up = budget.charge_node().expect_err(
        "the failed hash must retain the first JSON value's charge",
    );
    assert!(matches!(
        follow_up,
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
fn test_multi_values_hash_with_json_budget_matches_standard_hash_for_special_non_json_values()
 {
    let float = MultiValues::Float32(vec![-0.0]);
    let string_map = MultiValues::StringMap(vec![HashMap::from([
        ("second".to_owned(), "2".to_owned()),
        ("first".to_owned(), "1".to_owned()),
    ])]);
    let decimal =
        MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(10), 1)]);

    for values in [&float, &string_map, &decimal] {
        let expected = hash(values);
        let mut budget = JsonValueLimits::default().budget();
        let mut state = DefaultHasher::new();

        values
            .hash_with_json_budget(&mut state, &mut budget)
            .expect("non-JSON values must not consume the JSON budget");

        assert_eq!(state.finish(), expected);
    }
}
