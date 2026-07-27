//! Strict JSON tuple-variant behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_tuple_variant() {
    use qubit_value::Value;
    use serde::Serialize;

    #[derive(Serialize)]
    enum Fixture {
        Item(i32, bool),
    }

    let value =
        Value::from_serializable(&Fixture::Item(42, true)).expect("variant should serialize");
    assert_eq!(
        value.to_json_value().expect("project JSON"),
        serde_json::json!({"Item": [42, true]})
    );
}
