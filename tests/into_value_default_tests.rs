/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_value::IntoValueDefault;

#[test]
fn test_into_value_default_accepts_scalar_and_string_references() {
    let number: i32 = 7.into_value_default();
    assert_eq!(number, 7);

    let text: String = "fallback".into_value_default();
    assert_eq!(text, "fallback");

    let owned = String::from("owned");
    let cloned: String = (&owned).into_value_default();
    assert_eq!(cloned, owned);
}

#[test]
fn test_into_value_default_accepts_collection_references() {
    let values: Vec<i32> = (&[1, 2, 3][..]).into_value_default();
    assert_eq!(values, vec![1, 2, 3]);

    let words: Vec<String> = vec!["a", "b"].into_value_default();
    assert_eq!(words, vec!["a".to_string(), "b".to_string()]);
}
