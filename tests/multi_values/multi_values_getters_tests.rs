/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_value::MultiValues;

#[test]
fn test_multi_values_getters_return_slices_without_copying() {
    let values = MultiValues::String(vec!["red".to_string(), "blue".to_string()]);
    let strings = values.get_strings().unwrap();
    assert_eq!(strings, &["red", "blue"]);
    assert_eq!(strings.len(), values.count());
}
