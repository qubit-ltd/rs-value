//! Wide-integer wire display behavior.

#[test]
fn test_wide_integer_wire_displays_uint128() {
    use qubit_value::{Value, ValueWireV1};

    assert_eq!(
        serde_json::to_value(ValueWireV1::try_from(Value::UInt128(1)).unwrap()).unwrap()["value"]["scalar"]
            ["uint128"],
        "1"
    );
}
