//! Tests strict JSON tuple-variant serialization through the public API.

use qubit_value::Value;

/// Verifies structured JSON arrays retain tuple-like values.
#[test]
fn test_tuple_variant_serializer_preserves_values() {
    let value = Value::Json(serde_json::json!({"kind": [7, "value"]}));
    assert_eq!(
        value.to_json_value().unwrap(),
        serde_json::json!({"kind": [7, "value"]})
    );
}
