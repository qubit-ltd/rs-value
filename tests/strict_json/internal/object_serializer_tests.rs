//! Strict JSON object behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_struct_object() {
    use qubit_value::Value;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Object {
        answer: i32,
    }

    let value = Value::from_serializable(&Object { answer: 42 }).expect("struct should serialize");
    assert_eq!(
        value.to_json_value().expect("project JSON"),
        serde_json::json!({"answer": 42})
    );
}
