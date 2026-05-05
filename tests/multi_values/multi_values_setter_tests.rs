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
use qubit_value::MultiValues;
use qubit_value::multi_values::MultiValuesSetter;

#[test]
fn test_multi_values_setter_sets_owned_vectors() {
    let mut values = MultiValues::Empty(DataType::String);
    values
        .set_values(vec!["alpha".to_string(), "beta".to_string()])
        .unwrap();

    assert_eq!(values.data_type(), DataType::String);
    assert_eq!(values.get_strings().unwrap(), &["alpha", "beta"]);
}
