// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::{MultiValues, ValueError};

#[test]
fn test_multi_values_getters_return_slices_without_copying() {
    let values = MultiValues::String(vec!["red".to_string(), "blue".to_string()]);
    let strings = values.get_strings().unwrap();
    assert_eq!(strings, &["red", "blue"]);
    assert_eq!(strings.len(), values.len());
}

#[test]
fn test_multi_values_getters_distinguish_unset_from_concrete_empty() {
    let unset = MultiValues::Unset(DataType::Int32);
    assert!(matches!(unset.get::<i32>(), Err(ValueError::NoValue(_))));
    assert!(matches!(unset.get_int32s(), Err(ValueError::NoValue(_))));

    let empty = MultiValues::Int32(Vec::new());
    assert_eq!(empty.get::<i32>(), Ok(Vec::new()));
    assert_eq!(empty.get_int32s(), Ok(&[][..]));
}
