//! Tests owned wire shapes.

use qubit_value::{
    Value,
    ValueWireV1,
};

/// Verifies owned scalar wire values use the scalar shape.
#[test]
fn test_wire_shape_owned_writes_scalar_shape() {
    let wire = ValueWireV1::from(Value::Int32(7));
    assert!(
        serde_json::to_value(wire).unwrap()["value"]
            .get("scalar")
            .is_some()
    );
}
