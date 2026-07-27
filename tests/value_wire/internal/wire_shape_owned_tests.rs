//! Owned V1 shape behavior.

#[test]
fn test_owned_wire_shape_rejects_unknown_tag() {
    use qubit_value::ValueWireV1;

    assert!(
        serde_json::from_value::<ValueWireV1>(
            serde_json::json!({"version": 1, "value": {"scalar": {"unknown": 1}}})
        )
        .is_err()
    );
}
