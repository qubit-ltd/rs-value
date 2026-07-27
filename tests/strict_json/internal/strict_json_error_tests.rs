//! Strict JSON error behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_reports_serialization_error() {
    use qubit_value::Value;

    assert!(Value::from_serializable(&u128::MAX).is_err());
}
