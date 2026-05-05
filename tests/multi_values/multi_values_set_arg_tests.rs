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
fn test_multi_values_set_arg_replaces_with_vec_slice_and_single() {
    let mut values = MultiValues::Empty(DataType::Int32);
    values.set(vec![1, 2]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2]);

    values.set(&[3, 4][..]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[3, 4]);

    values.set(5).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[5]);
}
