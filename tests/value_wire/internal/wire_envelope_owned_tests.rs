//! Tests owned wire envelopes.

use qubit_value::{
    Value,
    ValueWireV1,
};

/// Verifies owned wire envelopes include the format version.
#[test]
fn test_wire_envelope_owned_writes_version() {
    let wire = ValueWireV1::from(Value::Int32(7));
    assert_eq!(
        serde_json::to_value(wire).unwrap()["version"],
        serde_json::json!(1)
    );
}
