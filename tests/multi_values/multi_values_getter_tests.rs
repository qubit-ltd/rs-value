/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    ValueError,
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
