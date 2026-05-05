/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_value::multi_values::MultiValuesAdder;
use qubit_value::{
    MultiValues,
    ValueError,
};

#[test]
fn test_multi_values_adder_appends_matching_single_value() {
    let mut values = MultiValues::Bool(vec![true]);
    values.add_value(false).unwrap();
    assert_eq!(values.get_bools().unwrap(), &[true, false]);

    let mut strings = MultiValues::String(vec!["x".to_string()]);
    assert!(matches!(
        strings.add_value(1i32),
        Err(ValueError::TypeMismatch { .. })
    ));
}
