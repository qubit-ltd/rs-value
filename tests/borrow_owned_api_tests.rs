// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueError;

#[test]
fn test_borrowed_scalar_and_collection_reads_keep_storage() {
    let text = String::from("hello");
    let ptr = text.as_ptr();
    let value = Value::String(text);
    let text: &str = value.get_ref().unwrap();
    assert_eq!(text, "hello");
    assert_eq!(text.as_ptr(), ptr);

    let values = MultiValues::String(vec![String::from("a"), String::from("b")]);
    let vec_ptr = values.get_slice::<String>().unwrap().as_ptr();
    let first: &String = values.get_first_ref().unwrap();
    let all: &[String] = values.get_slice().unwrap();
    assert_eq!(first, "a");
    assert_eq!(all.len(), 2);
    assert_eq!(all.as_ptr(), vec_ptr);

    let container = ValueContainer::from(vec![1_i32, 2, 3]);
    assert_eq!(container.get_slice::<i32>().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_borrowed_reads_preserve_missing_and_mismatch_errors() {
    let unset = Value::new_unset(DataType::String);
    assert!(matches!(unset.get_ref::<String>(), Err(ValueError::Missing(_))));

    let empty = MultiValues::String(Vec::new());
    assert!(matches!(empty.get_first_ref::<String>(), Err(ValueError::Missing(_))));
    let empty_slice: &[String] = empty.get_slice().unwrap();
    assert!(empty_slice.is_empty());

    let wrong = Value::Int32(1);
    assert!(matches!(
        wrong.get_ref::<String>(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_consuming_reads_move_storage() {
    let text = String::from("owned");
    let ptr = text.as_ptr();
    let value = Value::String(text);
    let text = String::try_from(value).unwrap();
    assert_eq!(text, "owned");
    assert_eq!(text.as_ptr(), ptr);

    let source = vec![1, 2];
    let ptr = source.as_ptr();
    let values = MultiValues::Int32(source);
    let items = Vec::<i32>::try_from(values).unwrap();
    assert_eq!(items, vec![1, 2]);
    assert_eq!(items.as_ptr(), ptr);
}
