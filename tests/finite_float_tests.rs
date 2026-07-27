//! Public finite-float wire behavior.

use qubit_value::{Value, ValueWireEncodeError, ValueWireV1};

#[test]
fn test_finite_float_wire_contract() {
    assert!(matches!(
        ValueWireV1::try_from(Value::Float64(f64::NAN)),
        Err(ValueWireEncodeError::NonFiniteFloat { .. })
    ));
}
