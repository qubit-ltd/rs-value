// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON serialization behavior.

#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serializer;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::ser::SerializeMap;

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
    use qubit_value::Value;

    assert!(Value::from_serializable(&f64::NAN).is_err());
}

/// Rejects object keys that collide during strict JSON projection.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_rejects_duplicate_object_key() {
    use qubit_value::Value;

    assert!(Value::from_serializable(&DuplicateKeyProbe).is_err());
}

/// Materializes RawValue as represented JSON instead of a protocol marker.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_materializes_raw_value() {
    use qubit_value::Value;
    use serde_json::json;
    use serde_json::value::RawValue;

    let raw = RawValue::from_string(String::from(r#"{"ok":true}"#))
        .expect("fixture should be valid raw JSON");
    let value =
        Value::from_serializable(&raw).expect("strict RawValue should project");

    assert_eq!(
        value.to_json_value().expect("project JSON"),
        json!({"ok": true})
    );
}
