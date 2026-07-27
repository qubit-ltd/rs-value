//! Strict JSON serialization behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_rejects_non_finite_float() {
    use qubit_value::Value;

    assert!(Value::from_serializable(&f64::NAN).is_err());
}
