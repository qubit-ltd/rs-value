// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON struct-variant behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_struct_variant() {
    use qubit_value::Value;
    use serde::Serialize;

    #[derive(Serialize)]
    enum Fixture {
        Item { answer: i32 },
    }

    let value = Value::from_serializable(&Fixture::Item { answer: 42 })
        .expect("variant should serialize");
    assert_eq!(
        value.to_json_value().expect("project JSON"),
        serde_json::json!({"Item": {"answer": 42}})
    );
}
