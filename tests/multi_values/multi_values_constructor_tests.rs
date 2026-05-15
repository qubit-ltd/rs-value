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
fn test_multi_values_constructor_builds_typed_variants() {
    let integers = MultiValues::new(vec![1i16, 2, 3]);
    assert_eq!(integers.data_type(), DataType::Int16);
    assert_eq!(integers.get::<i16>().unwrap(), vec![1, 2, 3]);

    let flags = MultiValues::new(vec![true, false]);
    assert_eq!(flags.data_type(), DataType::Bool);
    assert_eq!(flags.get_bools().unwrap(), &[true, false]);
}

#[test]
fn test_multi_values_constructor_accepts_owned_and_borrowed_strings() {
    let owned = MultiValues::new(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(owned.get_strings().unwrap(), &["a", "b"]);

    let borrowed = MultiValues::new(vec!["c", "d"]);
    assert_eq!(borrowed.get::<String>().unwrap(), vec!["c", "d"]);

    let slice = ["e", "f"];
    let from_slice = MultiValues::new(&slice[..]);
    assert_eq!(from_slice.get_strings().unwrap(), &["e", "f"]);
}
