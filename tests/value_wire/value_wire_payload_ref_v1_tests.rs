//! Borrowed V1 payload behavior.

#[test]
fn test_borrowed_payload_omits_envelope() {
    use qubit_value::{Value, ValueWirePayloadRefV1};

    assert_eq!(
        serde_json::to_value(ValueWirePayloadRefV1::try_from(&Value::Int32(1)).unwrap()).unwrap(),
        serde_json::json!({"scalar": {"int32": 1}})
    );
}
