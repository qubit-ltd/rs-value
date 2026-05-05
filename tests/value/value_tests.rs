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
fn test_value_preserves_type_when_cleared() {
    let mut value = Value::String("abc".to_string());
    value.clear();

    assert!(value.is_empty());
    assert_eq!(value.data_type(), DataType::String);
    assert!(matches!(value.get_string(), Err(ValueError::NoValue)));
}
