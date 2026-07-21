//! Tests strict scalar reads.

use qubit_value::Value;

/// Verifies strict reads return the value when the requested type matches.
#[test]
fn test_strict_value_read_returns_matching_value() {
    let value = Value::Int32(7);
    assert_eq!(
        value.get::<i32>().expect("matching type must be readable"),
        7
    );
}
