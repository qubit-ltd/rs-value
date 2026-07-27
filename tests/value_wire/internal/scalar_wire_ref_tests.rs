//! Borrowed scalar V1 wire behavior.

#[test]
fn test_borrowed_scalar_wire_serializes() {
    use qubit_value::{Value, ValueWireRefV1};

    assert!(serde_json::to_value(ValueWireRefV1::try_from(&Value::Int32(1)).unwrap()).is_ok());
}
