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

#[test]
fn test_multi_values_single_setter_replaces_existing_list() {
    let mut values = MultiValues::Int32(vec![1, 2, 3]);
    values.set(9).unwrap();

    assert_eq!(values.data_type(), DataType::Int32);
    assert_eq!(values.get_int32s().unwrap(), &[9]);
}
