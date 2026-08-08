// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_value::Value;
use qubit_value::ValueRef;

#[test]
fn test_value_ref_borrows_scalar_payload_without_changing_it() {
    let value = Value::String("ready".to_owned());

    assert!(matches!(value.view(), ValueRef::String(text) if text == "ready"));
    assert_eq!(value.get_string().expect("read string"), "ready");
}
