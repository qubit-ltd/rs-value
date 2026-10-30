// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private recursive canonical serializer for JSON values.

use serde::Serialize;
use serde::ser::Error as SerError;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::ser::Serializer;
use serde_json::Value;

/// Borrows a JSON value while recursively sorting object keys.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed JSON tree.
pub(in crate::wire) struct CanonicalJson<'a>(
    /// The JSON value to serialize.
    pub(in crate::wire) &'a Value,
);

impl Serialize for CanonicalJson<'_> {
    /// Serializes this JSON value with dictionary-order object keys.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Number(value) => {
                crate::wire::json::validate_number(value).map_err(S::Error::custom)?;
                value.serialize(serializer)
            }
            Value::String(value) => serializer.serialize_str(value),
            Value::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(&Self(value))?;
                }
                sequence.end()
            }
            Value::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                let mut entries: Vec<_> = values.iter().collect();
                entries.sort_unstable_by_key(|(left, _)| *left);
                for (key, value) in entries {
                    map.serialize_entry(key, &Self(value))?;
                }
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use qubit_budget::json::JsonEncodeLimits;
    use qubit_budget::json::JsonEncodeSession;
    use qubit_budget::json::JsonResource;
    use qubit_json::encode::JsonEncoder;
    use serde_json::Value;

    use super::CanonicalJson;

    #[test]
    fn test_canonical_json_rejects_non_round_trippable_number() {
        for text in ["0.12345678901234567890123", "1e1", "1.2500", "1e400", "-0"] {
            let Ok(Value::Number(number)) = serde_json::from_str::<Value>(text) else {
                continue;
            };
            let rendered = number.to_string();
            let restored = if rendered == "-0" {
                serde_json::Number::from_f64(-0.0)
            } else if rendered.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
                rendered.parse::<f64>().ok().and_then(serde_json::Number::from_f64)
            } else if rendered.starts_with('-') {
                rendered.parse::<i64>().ok().map(serde_json::Number::from)
            } else {
                rendered.parse::<u64>().ok().map(serde_json::Number::from)
            };
            if restored.as_ref() == Some(&number) {
                continue;
            }
            let value = Value::Number(number);
            assert!(serde_json::to_vec(&CanonicalJson(&value)).is_err());
            assert!(
                JsonEncoder::new(JsonEncodeSession::from_limits(
                    JsonEncodeLimits::<JsonResource, usize>::new(),
                ))
                .to_vec(&CanonicalJson(&value))
                .is_err()
            );
        }
    }

    #[test]
    fn test_canonical_json_serializes_standard_number() {
        for number in [
            serde_json::Number::from(42_u64),
            serde_json::Number::from_f64(0.125).expect("finite fixture"),
        ] {
            let value = Value::Number(number);
            assert!(serde_json::to_vec(&CanonicalJson(&value)).is_ok());
            assert!(
                JsonEncoder::new(JsonEncodeSession::from_limits(
                    JsonEncodeLimits::<JsonResource, usize>::new(),
                ))
                .to_vec(&CanonicalJson(&value))
                .is_ok()
            );
        }
    }
}
