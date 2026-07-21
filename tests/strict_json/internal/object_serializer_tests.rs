//! Tests strict JSON object serialization through the public API.

use qubit_value::Value;

/// Verifies JSON objects preserve their fields through value serialization.
#[test]
fn test_object_serializer_preserves_fields() {
    let value = Value::Json(serde_json::json!({"field": 7}));
    assert_eq!(
        value.to_json_value().unwrap(),
        serde_json::json!({"field": 7})
    );
}
