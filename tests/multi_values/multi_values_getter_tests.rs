// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    ValueError,
    ValueMissing,
};

#[test]
fn test_multi_values_getter_is_strict() {
    let values = MultiValues::Int32(vec![1, 2]);
    assert_eq!(values.get::<i32>().unwrap(), vec![1, 2]);
    assert!(matches!(
        values.get::<String>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
}

#[test]
fn test_multi_values_first_read_reports_precise_missing_state() {
    assert_eq!(
        MultiValues::Unset(DataType::Int32).get_first::<i32>(),
        Err(ValueError::Missing(ValueMissing::UnsetCollection {
            data_type: DataType::Int32,
        })),
    );
    assert_eq!(
        MultiValues::Int32(Vec::new()).get_first::<i32>(),
        Err(ValueError::Missing(ValueMissing::EmptyCollection {
            data_type: DataType::Int32,
        })),
    );
}
