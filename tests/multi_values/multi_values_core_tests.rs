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
fn test_multi_values_core_tracks_count_and_type_changes() {
    let mut values = MultiValues::Int32(vec![1, 2, 3]);
    assert_eq!(values.count(), 3);
    assert_eq!(values.data_type(), DataType::Int32);

    values.clear();
    assert!(values.is_empty());
    assert_eq!(values.data_type(), DataType::Int32);

    values.set_type(DataType::String);
    assert!(values.is_empty());
    assert_eq!(values.data_type(), DataType::String);
    assert!(matches!(
        values.get_first::<String>(),
        Err(ValueError::NoValue)
    ));
}
