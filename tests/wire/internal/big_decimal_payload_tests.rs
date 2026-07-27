//! Big-decimal V1 payload behavior.

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_wire_rejects_excessive_scale() {
    use qubit_value::ValueWireV1;

    assert!(serde_json::from_value::<ValueWireV1>(serde_json::json!({"version": 1, "value": {"scalar": {"bigdecimal": {"coefficient": "1", "scale": 150001}}}})).is_err());
}
