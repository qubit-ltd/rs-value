// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    ValueError,
};

#[test]
fn test_multi_values_len_and_is_empty_distinguish_unset_from_concrete_empty_values()
 {
    let unset = MultiValues::Unset(DataType::Int32);
    let empty = MultiValues::Int32(Vec::new());
    let values = MultiValues::Int32(vec![1, 2]);

    assert_eq!(unset.len(), 0);
    assert!(unset.is_empty());
    assert!(unset.is_unset());
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(!empty.is_unset());
    assert_eq!(values.len(), 2);
    assert!(!values.is_empty());
}

#[test]
fn test_multi_values_core_tracks_len_and_type_changes() {
    let mut values = MultiValues::Int32(vec![1, 2, 3]);
    assert_eq!(values.len(), 3);
    assert_eq!(values.data_type(), DataType::Int32);

    values.clear();
    assert!(!values.is_unset());
    assert_eq!(values.len(), 0);
    assert_eq!(values.data_type(), DataType::Int32);

    values.set_type(DataType::String);
    assert!(values.is_unset());
    assert_eq!(values.data_type(), DataType::String);
    assert!(matches!(
        values.get_first::<String>(),
        Err(ValueError::NoValue)
    ));
}

#[test]
fn test_multi_values_core_unset_preserves_declared_type() {
    let mut values = MultiValues::Int32(Vec::new());
    assert!(!values.is_unset());

    values.unset();

    assert!(values.is_unset());
    assert_eq!(values.data_type(), DataType::Int32);
    assert_eq!(values.len(), 0);
}

#[test]
fn test_multi_values_is_numeric_is_state_aware() {
    assert!(MultiValues::Int32(vec![1]).is_numeric());
    assert!(MultiValues::Float64(Vec::new()).is_numeric());
    assert!(!MultiValues::Unset(DataType::Int32).is_numeric());
    assert!(!MultiValues::String(Vec::new()).is_numeric());
    assert!(!MultiValues::Bool(vec![true]).is_numeric());
}

#[test]
fn test_multi_values_tagged_serde_rejects_non_finite_floats() {
    for finite in [
        MultiValues::Float32(vec![1.0, 2.5]),
        MultiValues::Float64(vec![1.0, 2.5]),
    ] {
        let json = serde_json::to_string(&finite).unwrap();
        assert_eq!(serde_json::from_str::<MultiValues>(&json).unwrap(), finite);
    }

    assert!(
        serde_json::to_value(MultiValues::Float32(vec![
            1.0,
            f32::NEG_INFINITY,
        ]))
        .is_err()
    );
    assert!(
        serde_json::to_value(MultiValues::Float64(vec![
            1.0,
            f64::NEG_INFINITY,
        ]))
        .is_err()
    );
}

#[test]
fn test_multi_values_core_get_is_strict() {
    let values = MultiValues::Int32(vec![1, 2]);
    assert_eq!(values.get::<i32>().unwrap(), vec![1, 2]);
    assert!(matches!(
        values.get::<String>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
}

#[test]
fn test_multi_values_core_get_first_reads_first_or_default() {
    let values = MultiValues::Int32(vec![8, 13]);
    assert_eq!(values.get_first::<i32>().unwrap(), 8);
    assert_eq!(values.get_first_or::<i32>(99).unwrap(), 8);

    let empty = MultiValues::Unset(DataType::Int32);
    assert_eq!(empty.get_first_or::<i32>(99).unwrap(), 99);
    assert!(matches!(empty.get_first::<i32>(), Err(ValueError::NoValue)));
}

#[test]
fn test_multi_values_core_set_replaces_with_vec_slice_and_single() {
    let mut values = MultiValues::Unset(DataType::Int32);
    values.set(vec![1, 2]);
    assert_eq!(values.get_int32s().unwrap(), &[1, 2]);

    values.set(&[3, 4][..]);
    assert_eq!(values.get_int32s().unwrap(), &[3, 4]);

    values.set(5);
    assert_eq!(values.get_int32s().unwrap(), &[5]);
}

#[test]
fn test_multi_values_core_set_clones_slice_values() {
    let mut values = MultiValues::Unset(DataType::Int64);
    let source = [10i64, 20, 30];
    values.set(&source[..]);

    assert_eq!(values.get_int64s().unwrap(), &[10, 20, 30]);
    assert_eq!(source, [10, 20, 30]);
}

#[test]
fn test_multi_values_core_set_sets_owned_vectors() {
    let mut values = MultiValues::Unset(DataType::String);
    values.set(vec!["alpha".to_string(), "beta".to_string()]);

    assert_eq!(values.data_type(), DataType::String);
    assert_eq!(values.get_strings().unwrap(), &["alpha", "beta"]);
}

#[test]
fn test_multi_values_core_set_single_replaces_existing_list() {
    let mut values = MultiValues::Int32(vec![1, 2, 3]);
    values.set(9);

    assert_eq!(values.data_type(), DataType::Int32);
    assert_eq!(values.get_int32s().unwrap(), &[9]);
}

#[test]
fn test_multi_values_core_add_appends_single_vec_and_slice() {
    let mut values = MultiValues::Int32(vec![1]);
    values.add(2).unwrap();
    values.add(vec![3, 4]).unwrap();
    values.add(&[5, 6][..]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_multi_values_core_add_appends_matching_single_value() {
    let mut values = MultiValues::Bool(vec![true]);
    values.add(false).unwrap();
    assert_eq!(values.get_bools().unwrap(), &[true, false]);

    let mut strings = MultiValues::String(vec!["x".to_string()]);
    assert!(matches!(
        strings.add(1i32),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_multi_values_core_add_initializes_empty_and_extends_existing() {
    let mut values = MultiValues::Unset(DataType::Int32);
    values.add(vec![1, 2]).unwrap();
    values.add(vec![3, 4]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);
}

#[test]
fn test_multi_values_core_add_slice_appends_without_consuming_source() {
    let source = [1u16, 2, 3];
    let mut values = MultiValues::Unset(DataType::UInt16);
    values.add(&source[..]).unwrap();
    values.add(&source[1..]).unwrap();

    assert_eq!(values.get_uint16s().unwrap(), &[1, 2, 3, 2, 3]);
    assert_eq!(source, [1, 2, 3]);
}
