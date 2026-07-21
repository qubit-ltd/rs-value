//! Tests parsed wide integer validation.

use qubit_value::Value;

/// Verifies malformed wide integer text is rejected.
#[test]
fn test_parsed_integer_rejects_malformed_text() {
    let result = serde_json::from_value::<Value>(
        serde_json::json!({"int128": "not-an-integer"}),
    );
    assert!(result.is_err());
}
