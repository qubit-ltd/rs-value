/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_value::MultiValues;

#[test]
fn test_multi_values_add_arg_appends_single_vec_and_slice() {
    let mut values = MultiValues::Int32(vec![1]);
    values.add(2).unwrap();
    values.add(vec![3, 4]).unwrap();
    values.add(&[5, 6][..]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4, 5, 6]);
}
