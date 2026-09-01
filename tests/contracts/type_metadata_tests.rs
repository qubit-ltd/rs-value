// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests that the complete runtime type set remains available.

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;

#[test]
fn all_core_types_have_scalar_and_collection_constructors() {
    let scalars = [
        Value::Bool(true).data_type(),
        Value::Char('x').data_type(),
        Value::Int8(1).data_type(),
        Value::Int16(1).data_type(),
        Value::Int32(1).data_type(),
        Value::Int64(1).data_type(),
        Value::Int128(1).data_type(),
        Value::UInt8(1).data_type(),
        Value::UInt16(1).data_type(),
        Value::UInt32(1).data_type(),
        Value::UInt64(1).data_type(),
        Value::UInt128(1).data_type(),
        Value::Float32(1.0).data_type(),
        Value::Float64(1.0).data_type(),
        Value::String(String::from("x")).data_type(),
        Value::Duration(std::time::Duration::from_secs(1)).data_type(),
        Value::StringMap(std::collections::HashMap::new()).data_type(),
    ];
    let collections = [
        MultiValues::Bool(vec![true]).data_type(),
        MultiValues::Char(vec!['x']).data_type(),
        MultiValues::Int32(vec![1]).data_type(),
        MultiValues::Float64(vec![1.0]).data_type(),
        MultiValues::String(vec![String::from("x")]).data_type(),
        MultiValues::Duration(vec![std::time::Duration::from_secs(1)]).data_type(),
        MultiValues::StringMap(vec![std::collections::HashMap::new()]).data_type(),
    ];

    assert!(scalars.contains(&DataType::Bool));
    assert!(scalars.contains(&DataType::StringMap));
    assert!(collections.contains(&DataType::Bool));
    assert!(collections.contains(&DataType::StringMap));
}
