//! Strict JSON map-key behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_string_map_keys() {
    use std::collections::BTreeMap;

    use qubit_value::Value;

    let value = Value::from_serializable(&BTreeMap::from([("answer", 42)]))
        .expect("string keys should serialize");
    assert_eq!(
        value.to_json_value().expect("project JSON"),
        serde_json::json!({"answer": 42})
    );
}
