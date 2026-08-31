// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for scalar value and unset invariants.

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;

#[test]
fn unset_preserves_declared_type_without_becoming_a_concrete_value() {
    let value = Value::new_unset(DataType::Int32);

    assert!(value.is_unset());
    assert_eq!(value.data_type(), DataType::Int32);
    assert!(value.get::<i32>().is_err());
}

#[test]
fn strict_getter_does_not_convert_a_concrete_value() {
    let value = Value::Int32(7);

    assert_eq!(value.get::<i32>().unwrap(), 7);
    assert!(value.get::<i64>().is_err());
}

#[test]
fn empty_collection_is_distinct_from_unset_collection() {
    let empty = MultiValues::Int32(Vec::new());
    let unset = MultiValues::new_unset(DataType::Int32);

    assert_eq!(empty.len(), 0);
    assert!(unset.is_unset());
    assert_ne!(empty, unset);
}
