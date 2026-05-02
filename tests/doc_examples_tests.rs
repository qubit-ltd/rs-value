/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Regression tests for public documentation examples.

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    NamedMultiValues,
    NamedValue,
    Value,
};

#[test]
fn test_doc_example_single_value_operations() {
    let v = Value::new(8080i32);
    let port: i32 = v.get().unwrap();
    assert_eq!(port, 8080);
    assert_eq!(v.get_int32().unwrap(), 8080);
    assert_eq!(v.to::<i64>().unwrap(), 8080i64);
    assert_eq!(v.to::<String>().unwrap(), "8080".to_string());

    let mut any = Value::Int32(42);
    any.clear();
    assert!(any.is_empty());
    assert_eq!(any.data_type(), DataType::Int32);
    any.set_type(DataType::String);
    any.set("hello").unwrap();
    assert_eq!(any.get_string().unwrap(), "hello");
}

#[test]
fn test_doc_example_multi_value_operations() {
    let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
    assert_eq!(ports.count(), 3);
    assert_eq!(ports.get_int32s().unwrap(), &[8080, 8081, 8082]);

    ports.add(8083).unwrap();
    ports.add(vec![8084, 8085]).unwrap();
    ports.add(&[8086, 8087][..]).unwrap();

    ports.set(vec![9001, 9002]).unwrap();
    assert_eq!(ports.get_int32s().unwrap(), &[9001, 9002]);

    let mut a = MultiValues::Int32(vec![1, 2]);
    let b = MultiValues::Int32(vec![3, 4]);
    a.merge(&b).unwrap();
    assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);

    let single = a.to_value();
    let first_val: i32 = single.get().unwrap();
    assert_eq!(first_val, 1);
}

#[test]
fn test_doc_example_named_value_operations() {
    let mut nv = NamedValue::new("timeout", Value::new(30i32));
    assert_eq!(nv.name(), "timeout");
    let timeout: i32 = nv.get().unwrap();
    assert_eq!(timeout, 30);

    nv.set_name("read_timeout");
    nv.set(45i32).unwrap();
    assert_eq!(nv.get_int32().unwrap(), 45);

    let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
    nmv.add(8082).unwrap();
    let first_port: i32 = nmv.get_first().unwrap();
    assert_eq!(first_port, 8080);

    let first_named = nmv.to_named_value();
    assert_eq!(first_named.name(), "ports");
    let val: i32 = first_named.get().unwrap();
    assert_eq!(val, 8080);
}
