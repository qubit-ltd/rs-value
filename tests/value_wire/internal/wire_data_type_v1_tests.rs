//! V1 type-tag behavior.

#[test]
fn test_wire_type_tag_is_decoded() {
    use qubit_value::{Value, ValueWireV1};

    let wire: ValueWireV1 = serde_json::from_value(
        serde_json::json!({"version": 1, "value": {"scalar": {"int32": 1}}}),
    )
    .unwrap();
    assert_eq!(wire.into_container(), Value::Int32(1).into());
}
