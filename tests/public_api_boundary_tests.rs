/*******************************************************************************
 *
 *    Copyright (c) 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Public API Boundary Tests
//!
//! Verifies that external callers can use the generic APIs without importing
//! the doc-hidden implementation traits that back their bounds.
//!
//! # Author
//!
//! Haixing Hu

use qubit_common::lang::DataType;
use qubit_value::{MultiValues, Value};

#[test]
fn test_value_generic_api_works_without_hidden_trait_imports() {
    let value = Value::new(42i32);

    let strict: i32 = value.get().unwrap();
    let converted: i64 = value.to().unwrap();

    assert_eq!(strict, 42);
    assert_eq!(converted, 42);

    let mut text = Value::Empty(DataType::String);
    text.set("hello").unwrap();

    assert_eq!(text.get_string().unwrap(), "hello");
}

#[test]
fn test_multi_values_generic_api_works_without_hidden_trait_imports() {
    let values = MultiValues::new(vec![1i32, 2, 3]);

    let all: Vec<i32> = values.get().unwrap();
    let first: i32 = values.get_first().unwrap();
    let converted_first: i64 = values.to().unwrap();
    let converted_all: Vec<i64> = values.to_list().unwrap();

    assert_eq!(all, vec![1, 2, 3]);
    assert_eq!(first, 1);
    assert_eq!(converted_first, 1);
    assert_eq!(converted_all, vec![1, 2, 3]);

    let mut values = MultiValues::Empty(DataType::Int32);
    values.set(vec![4i32, 5]).unwrap();
    values.add(6i32).unwrap();
    values.add(&[7i32, 8][..]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[4, 5, 6, 7, 8]);
}
