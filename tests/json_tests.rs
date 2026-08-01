// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_scalar() {
    use qubit_value::Value;

    assert_eq!(
        Value::Int32(42).to_json_value().expect("project scalar"),
        serde_json::json!(42),
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_float32_with_display_roundtrip() {
    use qubit_value::Value;

    let value = 1.2_f32;
    let projected = Value::Float32(value)
        .to_json_value()
        .expect("project float32");

    assert_eq!(
        serde_json::to_string(&projected).expect("serialize json"),
        value.to_string(),
    );
}
