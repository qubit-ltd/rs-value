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
    Value,
    ValueError,
};

#[test]
fn test_value_getter_is_strict_and_default_is_empty_only() {
    let value = Value::Int32(5);
    assert_eq!(value.get::<i32>().unwrap(), 5);
    assert!(matches!(
        value.get::<String>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));

    let empty = Value::Empty(DataType::String);
    assert_eq!(empty.get_or::<String>("missing").unwrap(), "missing");
}
