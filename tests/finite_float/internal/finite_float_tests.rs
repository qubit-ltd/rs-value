//! Finite-float adapter behavior.

#[test]
fn test_float_wire_accepts_finite_value() {
    use qubit_value::{Value, ValueWireV1};

    assert!(serde_json::to_value(ValueWireV1::try_from(Value::Float64(1.25)).unwrap()).is_ok());
}
