//! Strict JSON sequence behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_sequence() {
    use qubit_value::Value;

    let value = Value::from_serializable(&[1_i32, 2]).expect("sequence should serialize");
    assert_eq!(
        value.to_json_value().expect("project JSON"),
        serde_json::json!([1, 2])
    );
}
