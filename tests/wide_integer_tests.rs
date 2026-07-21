//! Tests wide integer JSON behavior.

use qubit_value::Value;

/// Verifies 128-bit integers retain their decimal JSON representation.
#[test]
fn test_wide_integer_serializes_as_decimal_text() {
    assert_eq!(
        serde_json::to_value(Value::Int128(i128::MAX)).unwrap(),
        serde_json::json!({
            "version": 1,
            "value": {"scalar": {"int128": i128::MAX.to_string()}},
        })
    );
}
