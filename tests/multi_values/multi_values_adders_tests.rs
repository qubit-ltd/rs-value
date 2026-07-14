// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_value::MultiValues;

#[test]
fn test_multi_values_generic_adders_append_expected_values() {
    let mut values = MultiValues::String(vec!["a".to_string()]);
    values.add("b".to_string()).unwrap();
    values.add(vec!["c".to_string(), "d".to_string()]).unwrap();
    values.add(&["e".to_string()][..]).unwrap();

    assert_eq!(values.get_strings().unwrap(), &["a", "b", "c", "d", "e"]);
}
