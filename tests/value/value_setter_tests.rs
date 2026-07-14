// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::Value;

#[test]
fn test_value_setter_replaces_value_and_runtime_type() {
    let mut value = Value::Int32(10);
    let (): () = value.set("ten");

    assert_eq!(value.data_type(), DataType::String);
    assert_eq!(value.get_string().unwrap(), "ten");

    value.set("eleven".to_string());
    assert_eq!(value.get::<String>().unwrap(), "eleven");
}
