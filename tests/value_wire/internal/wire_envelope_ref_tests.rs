//! Borrowed V1 envelope behavior.

#[test]
fn test_borrowed_wire_envelope_has_version_one() {
    use qubit_value::{Value, ValueWireRefV1};

    assert_eq!(
        serde_json::to_value(ValueWireRefV1::try_from(&Value::Int32(1)).unwrap()).unwrap()["version"],
        1
    );
}
