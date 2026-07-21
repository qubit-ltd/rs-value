//! Tests strict JSON sequence serialization through the public API.

use qubit_value::Value;

/// Verifies JSON sequences preserve element order.
#[test]
fn test_sequence_serializer_preserves_element_order() {
    let value = Value::Json(serde_json::json!([1, 2, 3]));
    assert_eq!(value.to_json_value().unwrap(), serde_json::json!([1, 2, 3]));
}
