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
use qubit_value::{
    MultiValues,
    ValueError,
};

#[test]
fn test_multi_values_first_getter_reads_first_or_default() {
    let values = MultiValues::Int32(vec![8, 13]);
    assert_eq!(values.get_first::<i32>().unwrap(), 8);
    assert_eq!(values.get_first_or::<i32>(99).unwrap(), 8);

    let empty = MultiValues::Empty(DataType::Int32);
    assert_eq!(empty.get_first_or::<i32>(99).unwrap(), 99);
    assert!(matches!(empty.get_first::<i32>(), Err(ValueError::NoValue)));
}
