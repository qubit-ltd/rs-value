//! Tests owned scalar wire conversion.

use qubit_value::{
    Value,
    ValueWireV1,
};

/// Verifies owned scalars serialize through the versioned wire format.
#[test]
fn test_scalar_wire_owned_serializes_scalar() {
    let wire = ValueWireV1::from(Value::Int32(7));
    assert_eq!(
        serde_json::to_value(wire).unwrap()["value"]["scalar"]["int32"],
        serde_json::json!(7)
    );
}
