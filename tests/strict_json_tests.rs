// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON serialization behavior.

#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::InvalidValueReason;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::Value;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::ValueError;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serializer;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::ser::SerializeMap;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::json;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::value::RawValue;

/// Emits a duplicate JSON object key through the public conversion boundary.
#[cfg(all(feature = "converter", feature = "json"))]
struct DuplicateKeyProbe;

#[cfg(all(feature = "converter", feature = "json"))]
impl Serialize for DuplicateKeyProbe {
    /// Serializes two entries using the same object key.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("same", &1_u8)?;
        map.serialize_entry("same", &2_u8)?;
        map.end()
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_rejects_non_finite_float() {
    let error = Value::from_serializable(&f64::NAN).expect_err("non-finite value must fail");
    let ValueError::Conversion(error) = error else {
        panic!("strict JSON projection must return a conversion error");
    };
    assert_eq!(
        error.reason(),
        Some(&InvalidValueReason::NonFinite),
        "non-finite JSON input must retain its failure reason"
    );
}

/// Maps strict wide-integer failures to the shared out-of-range reason.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_maps_wide_integer_to_out_of_range() {
    let error = Value::from_serializable(&u128::MAX).expect_err("wide integer must fail");
    let ValueError::Conversion(error) = error else {
        panic!("strict JSON projection must return a conversion error");
    };
    assert_eq!(
        error.reason(),
        Some(&InvalidValueReason::OutOfRange),
        "wide JSON integers must report an out-of-range reason"
    );
}

/// Rejects object keys that collide during strict JSON projection.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_rejects_duplicate_object_key() {
    assert!(
        Value::from_serializable(&DuplicateKeyProbe).is_err(),
        "strict JSON projection must reject duplicate object keys"
    );
}

/// Materializes RawValue as represented JSON instead of a protocol marker.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_materializes_raw_value() {
    let raw = RawValue::from_string(String::from(r#"{"ok":true}"#)).expect("fixture should be valid raw JSON");
    let value = Value::from_serializable(&raw).expect("strict RawValue should project");

    assert_eq!(
        value.to_json_value().expect("project JSON"),
        json!({"ok": true}),
        "RawValue must materialize its represented JSON object"
    );
}
