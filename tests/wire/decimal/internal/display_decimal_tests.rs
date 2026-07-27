//! Big-integer decimal display behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_serializes_negative_value() {
    use num_bigint::BigInt;
    use qubit_value::{Value, ValueWireV1};

    assert_eq!(
        serde_json::to_value(ValueWireV1::try_from(Value::BigInteger(BigInt::from(-42))).unwrap())
            .unwrap()["value"]["scalar"]["biginteger"],
        "-42"
    );
}
