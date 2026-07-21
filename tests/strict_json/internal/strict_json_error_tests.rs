//! Tests strict JSON error reporting through public deserialization.

use qubit_value::Value;

/// Verifies invalid strict wire payloads report an error.
#[test]
fn test_strict_json_error_rejects_unknown_wire_tag() {
    let result =
        serde_json::from_value::<Value>(serde_json::json!({"unknown": 7}));
    assert!(result.is_err());
}
