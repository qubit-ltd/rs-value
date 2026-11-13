// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::MultiValuesRef;
use qubit_value::Value;
use qubit_value::ValueError;

#[test]
fn test_multi_values_merge_clones_matching_non_empty_payload() {
    let mut values = MultiValues::Int32(vec![1, 2]);
    let other = MultiValues::Int32(vec![3, 4]);

    values.merge(&other).expect("matching collections should merge");

    assert_eq!(values.get_int32s().expect("merged values"), &[1, 2, 3, 4]);
    assert_eq!(other.get_int32s().expect("source values remain intact"), &[3, 4]);
}

#[test]
fn test_multi_values_add_empty_input_preserves_collection_state() {
    let mut values = MultiValues::Int32(vec![1]);

    values
        .add(Vec::<i32>::new())
        .expect("an empty matching input is a no-op");

    assert_eq!(values, MultiValues::Int32(vec![1]));

    let mut unset = MultiValues::Unset(DataType::Int32);
    unset
        .add(Vec::<i32>::new())
        .expect("an empty input can initialize no values");
    assert_eq!(unset, MultiValues::Unset(DataType::Int32));
}

#[test]
fn test_multi_values_add_empty_input_still_checks_type() {
    let mut values = MultiValues::Int32(vec![1]);

    assert!(matches!(
        values.add(Vec::<String>::new()),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::String,
        })
    ));
    assert_eq!(values, MultiValues::Int32(vec![1]));
}

#[test]
fn test_multi_values_view_distinguishes_unset_and_concrete_empty() {
    let unset = MultiValues::Unset(DataType::Int32);
    assert!(matches!(unset.view(), MultiValuesRef::Unset(DataType::Int32)));

    let empty = MultiValues::Int32(Vec::new());
    assert!(matches!(empty.view(), MultiValuesRef::Int32(items) if items.is_empty()));
}

#[test]
fn test_multi_values_first_value_and_owned_projection_preserve_type_when_empty() {
    let values = MultiValues::Int64(Vec::new());

    assert_eq!(values.first_value(), Value::Unset(DataType::Int64));
    assert_eq!(values.into_first_value(), Value::Unset(DataType::Int64));
}
