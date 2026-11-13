// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;

#[test]
fn test_value_preserves_type_when_unset() {
    let mut value = Value::String("abc".to_string());
    value.unset();

    assert!(value.is_unset(), "unset must remove the concrete string value");
    assert_eq!(
        value.data_type(),
        DataType::String,
        "unset must preserve the declared string type"
    );
    assert!(
        matches!(value.get_string(), Err(ValueError::Missing(_))),
        "reading an unset string must report a missing value"
    );
}
