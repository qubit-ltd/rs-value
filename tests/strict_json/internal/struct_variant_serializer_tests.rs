//! Tests strict JSON struct-variant serialization through the public API.

use qubit_value::Value;

/// Verifies structured JSON values retain named fields.
#[test]
fn test_struct_variant_serializer_preserves_named_fields() {
    let value = Value::Json(serde_json::json!({"kind": {"field": 7}}));
    assert_eq!(
        value.to_json_value().unwrap(),
        serde_json::json!({"kind": {"field": 7}})
    );
}
