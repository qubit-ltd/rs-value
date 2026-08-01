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
    use serde_json::Number;

    for bits in [
        0xC65B_9806_u32, // -14054.006
        0x4823_0AF3_u32, // 166955.8
        0x9CA9_7CE0_u32, // 0.000000000000000000000000000004358592
        0x4078_7ACD_u32, // 3.8824952
        0x2696_F5F4_u32, // 0.000000000000001047500658
    ] {
        let value = f32::from_bits(bits);
        let projected = Value::Float32(value)
            .to_json_value()
            .expect("project float32");
        let projected_text = serde_json::to_string(&projected).expect("serialize json");

        let legacy_text = serde_json::to_string(&serde_json::Value::Number(
            Number::from_f64(f64::from(value)).expect("finite f64"),
        ))
        .expect("legacy serialize json");

        assert_eq!(
            projected_text,
            value.to_string(),
            "natural json should preserve f32 display text",
        );
        assert_ne!(
            projected_text, legacy_text,
            "this sample should differ from f32->f64 cast path",
        );
    }
}
