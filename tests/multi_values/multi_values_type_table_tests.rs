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
fn test_multi_values_type_table_maps_representative_variants() {
    assert_eq!(MultiValues::Bool(vec![true]).data_type(), DataType::Bool);
    assert_eq!(MultiValues::Char(vec!['x']).data_type(), DataType::Char);
    assert_eq!(
        MultiValues::Float64(vec![1.25]).data_type(),
        DataType::Float64
    );
    assert_eq!(
        MultiValues::Empty(DataType::Json).data_type(),
        DataType::Json
    );
}
