//! Tests owned collection wire conversion.

use qubit_value::{
    MultiValues,
    ValueWireV1,
};

/// Verifies owned collections serialize through the versioned wire format.
#[test]
fn test_collection_wire_owned_serializes_collection() {
    let wire = ValueWireV1::from(MultiValues::Int32(vec![7]));
    assert_eq!(
        serde_json::to_value(wire).unwrap()["value"]["collection"]["int32"],
        serde_json::json!([7])
    );
}
