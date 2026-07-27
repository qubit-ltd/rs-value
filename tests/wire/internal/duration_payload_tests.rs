//! Duration V1 payload behavior.

#[test]
fn test_duration_wire_rejects_invalid_nanos() {
    use qubit_value::ValueWireV1;

    assert!(serde_json::from_value::<ValueWireV1>(serde_json::json!({"version": 1, "value": {"scalar": {"duration": {"secs": 1, "nanos": 1_000_000_000}}}})).is_err());
}
