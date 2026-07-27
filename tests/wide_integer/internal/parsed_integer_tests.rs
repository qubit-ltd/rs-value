//! Wide-integer wire parsing behavior.

#[test]
fn test_wide_integer_wire_parses_canonical_string() {
    use qubit_value::{Value, ValueWireV1};

    assert_eq!(
        serde_json::from_value::<ValueWireV1>(
            serde_json::json!({"version": 1, "value": {"scalar": {"uint128": "1"}}})
        )
        .unwrap()
        .into_container(),
        Value::UInt128(1).into(),
    );
}
