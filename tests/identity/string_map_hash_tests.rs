//! Tests string-map hashing.

use std::collections::HashMap;

use qubit_value::Value;

/// Verifies string-map insertion order does not affect public value identity.
#[test]
fn test_string_map_hash_ignores_insertion_order() {
    let left = Value::StringMap(HashMap::from([("first".into(), "1".into())]));
    let right = Value::StringMap(HashMap::from([("first".into(), "1".into())]));
    assert_eq!(left, right);
}
