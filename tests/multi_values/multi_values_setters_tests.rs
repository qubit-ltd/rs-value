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
fn test_multi_values_named_setters_replace_all_values() {
    let mut values = MultiValues::String(vec!["old".to_string()]);
    values
        .set_strings(vec!["new".to_string(), "next".to_string()])
        .unwrap();
    assert_eq!(values.get_strings().unwrap(), &["new", "next"]);

    values.set_string("single".to_string()).unwrap();
    assert_eq!(values.get_strings().unwrap(), &["single"]);
}
