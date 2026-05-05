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
fn test_multi_values_multi_adder_slice_appends_without_consuming_source() {
    let source = [1u16, 2, 3];
    let mut values = MultiValues::Empty(DataType::UInt16);
    values.add(&source[..]).unwrap();
    values.add(&source[1..]).unwrap();

    assert_eq!(values.get_uint16s().unwrap(), &[1, 2, 3, 2, 3]);
    assert_eq!(source, [1, 2, 3]);
}
