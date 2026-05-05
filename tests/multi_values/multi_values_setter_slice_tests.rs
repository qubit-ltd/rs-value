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
fn test_multi_values_setter_slice_clones_slice_values() {
    let mut values = MultiValues::Empty(DataType::Int64);
    let source = [10i64, 20, 30];
    values.set(&source[..]).unwrap();

    assert_eq!(values.get_int64s().unwrap(), &[10, 20, 30]);
    assert_eq!(source, [10, 20, 30]);
}
