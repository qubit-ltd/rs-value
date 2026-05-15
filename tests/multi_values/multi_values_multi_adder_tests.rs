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
fn test_multi_values_multi_adder_initializes_empty_and_extends_existing() {
    let mut values = MultiValues::Empty(DataType::Int32);
    values.add(vec![1, 2]).unwrap();
    values.add(vec![3, 4]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);
}
