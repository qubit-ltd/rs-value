//! Borrowed V1 shape behavior.

#[test]
fn test_borrowed_wire_shape_preserves_collection() {
    use qubit_value::{MultiValues, ValueWireRefV1};

    assert_eq!(
        serde_json::to_value(ValueWireRefV1::try_from(&MultiValues::Int32(vec![1])).unwrap())
            .unwrap()["value"]["collection"]["int32"],
        serde_json::json!([1])
    );
}
