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
    values
        .add("b".to_string())
        .expect("adding one string to string values should succeed");
    values
        .add(vec!["c".to_string(), "d".to_string()])
        .expect("adding an owned string vector to string values should succeed");
    values
        .add(&["e".to_string()][..])
        .expect("adding a borrowed string slice to string values should succeed");

    assert_eq!(
        values
            .get_strings()
            .expect("string additions should preserve the string variant"),
        &["a", "b", "c", "d", "e"],
        "generic adders should append each input in order after existing values"
    );
}
