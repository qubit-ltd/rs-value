//! Tests strict collection reads.

use qubit_value::MultiValues;

/// Verifies strict collection reads return matching values.
#[test]
fn test_strict_value_list_read_returns_matching_values() {
    let values = MultiValues::Int32(vec![7]);
    assert_eq!(
        values.get::<i32>().expect("matching type must be readable"),
        &[7]
    );
}
