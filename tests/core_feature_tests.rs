// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value, ValueError};

#[test]
fn unset_is_distinct_from_concrete_empty_values_without_optional_features() {
    let unset_value = Value::Empty(DataType::String);
    let empty_string = Value::String(String::new());
    assert!(unset_value.is_unset());
    assert!(!empty_string.is_unset());

    let unset_values = MultiValues::Empty(DataType::Int32);
    let empty_values = MultiValues::Int32(Vec::new());
    assert!(unset_values.is_unset());
    assert!(!empty_values.is_unset());
    assert!(empty_values.get_int32s().unwrap().is_empty());
}

#[test]
fn generic_mutation_is_available_without_optional_features() {
    let mut value = Value::Empty(DataType::Int32);
    value.set(42_i32);
    assert_eq!(value.get::<i32>().unwrap(), 42);

    let mut values = MultiValues::Empty(DataType::Int32);
    values.set([1_i32, 2]);
    values.add(&[3_i32, 4][..]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);

    assert!(matches!(
        values.add(true),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::Bool,
        })
    ));
}

#[test]
fn numeric_classification_depends_on_concrete_state() {
    assert!(!Value::Empty(DataType::Int128).is_numeric());
    assert!(Value::Int128(i128::MIN).is_numeric());
    assert!(!MultiValues::Empty(DataType::UInt128).is_numeric());
    assert!(MultiValues::UInt128(Vec::new()).is_numeric());
    assert!(!Value::String("1".to_string()).is_numeric());
}

#[test]
fn named_wrappers_retain_generic_core_access() {
    let mut named = NamedValue::new("port", Value::Int32(8080));
    named.set(9090_i32);
    assert_eq!(named.get_int32().unwrap(), 9090);

    let mut named_values = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080]));
    named_values.add(9090_i32).unwrap();
    assert_eq!(named_values.get_int32s().unwrap(), &[8080, 9090]);
}
