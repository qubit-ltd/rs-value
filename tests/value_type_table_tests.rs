//! Tests runtime value type lookup.

use qubit_datatype::DataType;
use qubit_value::Value;

/// Verifies values report their concrete runtime data type.
#[test]
fn test_value_type_table_reports_concrete_type() {
    assert_eq!(Value::Int32(7).data_type(), DataType::Int32);
}
