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
fn test_multi_values_constructor_builds_typed_variants() {
    let integers = MultiValues::new(vec![1i16, 2, 3]);
    assert_eq!(integers.data_type(), DataType::Int16);
    assert_eq!(integers.get::<i16>().unwrap(), vec![1, 2, 3]);

    let flags = MultiValues::new(vec![true, false]);
    assert_eq!(flags.data_type(), DataType::Bool);
    assert_eq!(flags.get_bools().unwrap(), &[true, false]);
}
