//! Big-integer decimal visitor behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_noncanonical_string() {
    use qubit_value::ValueWireV1;

    assert!(
        serde_json::from_value::<ValueWireV1>(
            serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "042"}}})
        )
        .is_err()
    );
}
