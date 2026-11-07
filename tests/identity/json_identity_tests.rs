// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests JSON identity normalization.

#[cfg(feature = "json")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "json")]
use std::env::current_exe;
#[cfg(feature = "json")]
use std::env::var_os;
#[cfg(feature = "json")]
use std::hash::Hash;
#[cfg(feature = "json")]
use std::hash::Hasher;
#[cfg(feature = "json")]
use std::panic::AssertUnwindSafe;
#[cfg(feature = "json")]
use std::panic::catch_unwind;
#[cfg(feature = "json")]
use std::process::Command;

#[cfg(feature = "json")]
use qubit_budget::BudgetError;
#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::Observation;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
#[cfg(feature = "json")]
use qubit_budget::json::JsonValueLimits;
use qubit_value::Value;
use serde_json::Map;
use serde_json::Value as JsonValue;
use serde_json::json;

#[cfg(feature = "json")]
#[derive(Default)]
struct RecordingHasher(Vec<u8>);

#[cfg(feature = "json")]
impl Hasher for RecordingHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

#[cfg(feature = "json")]
#[derive(Default)]
struct EventHasher(Vec<Vec<u8>>);

#[cfg(feature = "json")]
impl Hasher for EventHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.push(bytes.to_vec());
    }
}

/// Verifies root JSON leaves retain the Value prefix, JSON tag, and payload
/// writes.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_root_leaf_write_sequence() {
    let fixtures = vec![
        (JsonValue::Null, 0_u8, None),
        (JsonValue::Bool(true), 1_u8, Some(JsonValue::Bool(true))),
        (json!(42), 2_u8, Some(json!(42))),
        (
            JsonValue::Number(serde_json::Number::from_f64(42.5).expect("finite float")),
            2_u8,
            Some(JsonValue::Number(
                serde_json::Number::from_f64(42.5).expect("finite float"),
            )),
        ),
        (
            JsonValue::Number(serde_json::Number::from_f64(-0.0).expect("finite signed zero")),
            2_u8,
            Some(JsonValue::Number(
                serde_json::Number::from_f64(-0.0).expect("finite signed zero"),
            )),
        ),
        (json!("leaf"), 3_u8, Some(json!("leaf"))),
    ];

    let mut null_events = EventHasher::default();
    Value::Json(JsonValue::Null).hash(&mut null_events);
    let mut prefix = null_events.0;
    assert_eq!(prefix.pop(), Some(vec![0]), "Null's JSON tag is the final write");

    for (value, tag, payload) in fixtures {
        let mut actual = EventHasher::default();
        Value::Json(value).hash(&mut actual);
        let mut expected = prefix.clone();
        tag.hash(&mut EventHasherAppender(&mut expected));
        if let Some(payload) = payload {
            match payload {
                JsonValue::Bool(value) => value.hash(&mut EventHasherAppender(&mut expected)),
                JsonValue::Number(value) => value.hash(&mut EventHasherAppender(&mut expected)),
                JsonValue::String(value) => value.hash(&mut EventHasherAppender(&mut expected)),
                _ => unreachable!(),
            }
        }
        assert_eq!(actual.0, expected, "unexpected write sequence for JSON tag {tag}");
    }
}

#[cfg(feature = "json")]
struct EventHasherAppender<'a>(&'a mut Vec<Vec<u8>>);

#[cfg(feature = "json")]
impl Hasher for EventHasherAppender<'_> {
    fn finish(&self) -> u64 {
        0
    }
    fn write(&mut self, bytes: &[u8]) {
        self.0.push(bytes.to_vec());
    }
}

#[cfg(feature = "json")]
struct PanickingHasher;

#[cfg(feature = "json")]
impl Hasher for PanickingHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("test hasher panic");
    }
}

#[cfg(feature = "json")]
const DEEP_JSON_IDENTITY_CHILD_ENV: &str = "QUBIT_VALUE_DEEP_JSON_IDENTITY_CHILD";
#[cfg(feature = "json")]
const DEEP_JSON_HASH_CHILD_ENV: &str = "QUBIT_VALUE_DEEP_JSON_HASH_CHILD";
#[cfg(feature = "json")]
const DEEP_JSON_DEPTH: usize = 10_000;

/// Verifies JSON object key order does not affect public value identity.
#[test]
fn test_json_identity_ignores_object_key_order() {
    let left = Value::Json(json!({"first": 1, "second": 2}));
    let right = Value::Json(json!({"second": 2, "first": 1}));
    assert_eq!(left, right);
}

/// Verifies hashing ignores the insertion order of JSON object entries.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_ignores_object_insertion_order() {
    let mut left = Map::new();
    left.insert("first".into(), json!([1, 2]));
    left.insert("second".into(), json!({"nested": true}));
    let mut right = Map::new();
    right.insert("second".into(), json!({"nested": true}));
    right.insert("first".into(), json!([1, 2]));

    assert_eq!(
        calculate_json_hash(&JsonValue::Object(left)),
        calculate_json_hash(&JsonValue::Object(right)),
    );
}

/// Verifies hashing remains sensitive to JSON array element order.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_distinguishes_array_order() {
    assert_ne!(
        calculate_json_hash(&json!([1, 2, 3])),
        calculate_json_hash(&json!([3, 2, 1])),
    );
}

/// Verifies deeply nested JSON hashing is evaluated without recursion.
#[cfg(feature = "json")]
#[test]
fn test_deep_json_identity_hash_does_not_recurse() {
    let output = Command::new(current_exe().expect("locate test binary"))
        .arg("--exact")
        .arg("identity::json_identity_tests::test_deep_json_identity_hash_in_isolated_process")
        .arg("--ignored")
        .arg("--nocapture")
        .env(DEEP_JSON_HASH_CHILD_ENV, "1")
        .output()
        .expect("run deep JSON hash child test");

    assert!(
        output.status.success(),
        "deep JSON hash child failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("QUBIT_VALUE_DEEP_HASH_DONE"),
        "deep JSON hash child did not complete\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Exercises JSON hashing in a process isolated from stack overflow.
#[cfg(feature = "json")]
#[test]
#[ignore = "run only through test_deep_json_identity_hash_does_not_recurse"]
fn test_deep_json_identity_hash_in_isolated_process() {
    assert!(
        var_os(DEEP_JSON_HASH_CHILD_ENV).is_some(),
        "this helper test must run through its parent test"
    );

    let value = Value::Json(build_deep_json_value(1));
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    let _ = state.finish();

    let json = JsonValue::try_from(value).expect("hash fixture retains its JSON type");
    dismantle_json_value(json);
    eprintln!("QUBIT_VALUE_DEEP_HASH_DONE");
}

/// Verifies the root is counted as depth one by budgeted hashing.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_root_inclusive_depth() {
    let error = hash_json_with_limits(&json!([null]), JsonValueLimits::builder().max_depth(1).build());
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::Depth,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}

/// Verifies budgeted hashing charges every visited JSON node.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_charges_nodes() {
    let error = hash_json_with_limits(
        &json!([null]),
        JsonValueLimits::<JsonResource, usize>::builder()
            .max_nodes(1_usize)
            .build(),
    );
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        })
    ));
}

/// Verifies a wide array still honors a node budget.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_rejects_wide_array_by_node_budget() {
    let value = JsonValue::Array((0..10_000).map(|_| JsonValue::Null).collect());
    let error = hash_json_with_limits(
        &value,
        JsonValueLimits::<JsonResource, usize>::builder()
            .max_nodes(1_usize)
            .build(),
    );
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            remaining: 0,
            requested: 1,
            ..
        })
    ));
}

/// Verifies a wide object still honors a node budget.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_rejects_wide_object_by_node_budget() {
    let value = JsonValue::Object(
        (0..10_000)
            .map(|index| (format!("key-{index}"), JsonValue::Null))
            .collect(),
    );
    let error = hash_json_with_limits(
        &value,
        JsonValueLimits::<JsonResource, usize>::builder()
            .max_nodes(1_usize)
            .build(),
    );
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            remaining: 0,
            requested: 1,
            ..
        })
    ));
}

/// Verifies budgeted hashing checks each array's item count.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_sequence_items() {
    let error = hash_json_with_limits(
        &json!([null, null]),
        JsonValueLimits::builder().max_sequence_items(1).build(),
    );
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::SequenceItems,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}

/// Verifies budgeted hashing checks each object's entry count.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_map_entries() {
    let error = hash_json_with_limits(
        &json!({"first": null, "second": null}),
        JsonValueLimits::builder().max_map_entries(1).build(),
    );
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::MapEntries,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}

/// Verifies budgeted hashing checks object key byte lengths.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_key_bytes() {
    let error = hash_json_with_limits(&json!({"é": null}), JsonValueLimits::builder().max_key_bytes(1).build());
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::KeyBytes,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}

/// Verifies budgeted hashing checks JSON string byte lengths.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_string_bytes() {
    let error = hash_json_with_limits(&json!("é"), JsonValueLimits::builder().max_string_bytes(1).build());
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}

/// Verifies budgeted hashing checks JSON number text byte lengths.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_checks_number_bytes() {
    let error = hash_json_with_limits(&json!(1234), JsonValueLimits::builder().max_number_bytes(3).build());
    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::NumberBytes,
            observed: Observation::Exact(4),
            maximum: 3,
        })
    ));
}

/// Verifies a rejected JSON value does not modify its hasher or budget.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_error_is_atomic() {
    let value = json!([null]);
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder().max_nodes(1).budget();
    let mut state = RecordingHasher::default();

    assert!(
        Value::Json(value.clone())
            .hash_with_json_budget(&mut state, &mut budget)
            .is_err()
    );
    assert!(state.0.is_empty());
    assert_eq!(budget.used_nodes(), Some(0));
}

/// Verifies a hasher panic leaves the direct JSON transaction staged and
/// reusable for a later successful value.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_panic_rolls_back_and_reuses_budget() {
    let value = json!(null);
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder().max_nodes(1).budget();

    let result = catch_unwind(AssertUnwindSafe(|| {
        Value::Json(value.clone()).hash_with_json_budget(&mut PanickingHasher, &mut budget)
    }));
    assert!(result.is_err());
    assert_eq!(budget.used_nodes(), Some(0));

    let mut state = RecordingHasher::default();
    Value::Json(value)
        .hash_with_json_budget(&mut state, &mut budget)
        .expect("the rolled-back budget must accept a later value");
    assert!(!state.0.is_empty());
    assert_eq!(budget.used_nodes(), Some(1));
}

/// Verifies an unconfigured budget preserves the ordinary hash result.
#[cfg(feature = "json")]
#[test]
fn test_hash_json_with_budget_matches_unbounded_hash() {
    let value = json!({
        "array": [1, {"nested": "text"}],
        "flag": false
    });
    let expected = calculate_json_hash(&value);
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder().budget();
    let mut state = DefaultHasher::new();
    Value::Json(value)
        .hash_with_json_budget(&mut state, &mut budget)
        .expect("an unconfigured JSON budget must accept the value");
    assert_eq!(state.finish(), expected);
}

/// Verifies deeply nested JSON equality is evaluated without recursion.
#[cfg(feature = "json")]
#[test]
fn test_deep_json_identity_equality_does_not_recurse() {
    let output = Command::new(current_exe().expect("locate test binary"))
        .arg("--exact")
        .arg("identity::json_identity_tests::test_deep_json_identity_equality_in_isolated_process")
        .arg("--ignored")
        .arg("--nocapture")
        .env(DEEP_JSON_IDENTITY_CHILD_ENV, "1")
        .output()
        .expect("run deep JSON identity child test");

    assert!(
        output.status.success(),
        "deep JSON identity child failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("QUBIT_VALUE_DEEP_EQUALITY_DONE"),
        "deep JSON equality child did not complete\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Exercises JSON equality in a process isolated from stack overflow.
#[cfg(feature = "json")]
#[test]
#[ignore = "run only through test_deep_json_identity_equality_does_not_recurse"]
fn test_deep_json_identity_equality_in_isolated_process() {
    assert!(
        var_os(DEEP_JSON_IDENTITY_CHILD_ENV).is_some(),
        "this helper test must run through its parent test"
    );

    let left = Value::Json(build_deep_json_value(1));
    let equal = Value::Json(build_deep_json_value(1));
    let unequal = Value::Json(build_deep_json_value(2));

    assert!(left == equal, "matching deeply nested JSON values must be equal");
    assert!(
        left != unequal,
        "leaf differences in deeply nested JSON values must not be equal"
    );

    for value in [left, equal, unequal] {
        let json = JsonValue::try_from(value).expect("equality fixture retains its JSON type");
        dismantle_json_value(json);
    }
    eprintln!("QUBIT_VALUE_DEEP_EQUALITY_DONE");
}

/// Hashes a JSON value with the public value identity contract.
#[cfg(feature = "json")]
fn calculate_json_hash(value: &JsonValue) -> u64 {
    let mut state = DefaultHasher::new();
    Value::Json(value.clone()).hash(&mut state);
    state.finish()
}

/// Hashes a JSON value with one limit set and returns the expected failure.
#[cfg(feature = "json")]
fn hash_json_with_limits(value: &JsonValue, limits: JsonValueLimits) -> MeasuredBudgetError<JsonResource, usize> {
    let mut budget = limits.budget();
    let mut state = DefaultHasher::new();
    match Value::Json(value.clone())
        .hash_with_json_budget(&mut state, &mut budget)
        .expect_err("the configured JSON limit must reject the value")
    {
        MeasuredBudgetError::Budget(error) => MeasuredBudgetError::Budget(error),
        MeasuredBudgetError::Quantity { .. } => {
            panic!("u64 JSON budget must represent native test measurements")
        }
    }
}

/// Builds an array chain whose innermost value is the supplied integer.
#[cfg(feature = "json")]
fn build_deep_json_value(leaf: i64) -> JsonValue {
    let mut value = JsonValue::Number(leaf.into());
    for _ in 0..DEEP_JSON_DEPTH {
        value = JsonValue::Array(vec![value]);
    }
    value
}

/// Iteratively consumes a JSON fixture so its destructor cannot recurse.
#[cfg(feature = "json")]
fn dismantle_json_value(value: JsonValue) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            JsonValue::Array(values) => pending.extend(values),
            JsonValue::Object(values) => pending.extend(values.into_values()),
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
        }
    }
}

/// Equality preserves early and late mismatches in wide nested containers.
#[test]
fn test_json_equality_wide_containers() {
    let array = JsonValue::Array((0..8192).map(JsonValue::from).collect());
    let object = JsonValue::Object(
        (0..8192)
            .map(|index| (index.to_string(), JsonValue::from(index)))
            .collect(),
    );
    for json in [array, object] {
        let left = Value::Json(json.clone());
        assert_eq!(left, Value::Json(json.clone()));
        let mut changed = json;
        match &mut changed {
            JsonValue::Array(items) => items[8191] = JsonValue::Null,
            JsonValue::Object(items) => {
                items.insert("8191".into(), JsonValue::Null);
            }
            _ => unreachable!("wide container fixture"),
        }
        assert_ne!(left, Value::Json(changed));
    }
}
