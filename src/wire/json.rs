// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde adapter for JSON values.

use std::fmt;
use std::fmt::Write as FmtWrite;

use qubit_json::value::DuplicateKeyRejectingJsonValue;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
#[cfg(feature = "converter")]
use serde_json::Map;
use serde_json::Number;
use serde_json::Value;

use super::internal::CanonicalJson;
use super::internal::JsonNumberChildren;
use crate::value_wire::ValueWireEncodeError;

/// Serializes one JSON value with recursively ordered object keys.
///
/// # Type Parameters
///
/// * `S` - Destination serializer type.
///
/// # Parameters
///
/// * `value` - JSON tree to serialize canonically.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's result.
///
/// # Errors
///
/// Returns `S::Error` when serialization fails.
pub(crate) fn serialize<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    CanonicalJson(value).serialize(serializer)
}

/// Deserializes one JSON value from any Serde representation.
///
/// # Type Parameters
///
/// * `D` - Source deserializer type.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// The decoded JSON tree after duplicate-key validation.
///
/// # Errors
///
/// Returns `D::Error` when decoding fails or an object repeats a key.
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    DuplicateKeyRejectingJsonValue::deserialize(deserializer).map(DuplicateKeyRejectingJsonValue::into_inner)
}

/// Checks whether a JSON number survives the V1 decoder's number model
/// without changing its `serde_json::Number` identity.
///
/// # Errors
///
/// Returns [`ValueWireEncodeError::JsonNumberNotRoundTrippable`] when the
/// number is outside the integer/f64 model or its representation would change.
pub(crate) fn validate_number(number: &Number) -> Result<(), ValueWireEncodeError> {
    if let Some(value) = number.as_i64()
        && number_matches_integer(number, value)
    {
        return Ok(());
    }
    if let Some(value) = number.as_u64()
        && number_matches_integer(number, value)
    {
        return Ok(());
    }
    let text = number.to_string();
    let canonical = if text == "-0" {
        Number::from_f64(-0.0)
    } else if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
        text.parse::<f64>().ok().and_then(Number::from_f64)
    } else if text.starts_with('-') {
        text.parse::<i64>().ok().map(Number::from)
    } else {
        text.parse::<u64>().ok().map(Number::from)
    };
    if canonical.as_ref() == Some(number) {
        Ok(())
    } else {
        Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
    }
}

const MAX_INTEGER_TEXT_LENGTH: usize = 20;

struct FixedNumberText {
    bytes: [u8; MAX_INTEGER_TEXT_LENGTH],
    length: usize,
}

impl FixedNumberText {
    fn new() -> Self {
        Self {
            bytes: [0; MAX_INTEGER_TEXT_LENGTH],
            length: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

impl fmt::Write for FixedNumberText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(end) = self.length.checked_add(value.len()) else {
            return Err(fmt::Error);
        };
        let Some(destination) = self.bytes.get_mut(self.length..end) else {
            return Err(fmt::Error);
        };
        destination.copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}

struct NumberTextComparator<'a> {
    expected: &'a [u8],
    offset: usize,
}

impl NumberTextComparator<'_> {
    fn is_equal(&self) -> bool {
        self.offset == self.expected.len()
    }
}

impl fmt::Write for NumberTextComparator<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(end) = self.offset.checked_add(value.len()) else {
            return Err(fmt::Error);
        };
        let Some(expected) = self.expected.get(self.offset..end) else {
            return Err(fmt::Error);
        };
        if expected != value.as_bytes() {
            return Err(fmt::Error);
        }
        self.offset = end;
        Ok(())
    }
}

fn number_matches_integer<T: fmt::Display>(number: &Number, integer: T) -> bool {
    let mut expected = FixedNumberText::new();
    if write!(&mut expected, "{integer}").is_err() {
        return false;
    }
    let mut comparator = NumberTextComparator {
        expected: expected.as_bytes(),
        offset: 0,
    };
    write!(&mut comparator, "{number}").is_ok() && comparator.is_equal()
}

/// Checks every number in a JSON tree using iterator frames bounded by depth.
///
/// # Errors
///
/// Returns [`ValueWireEncodeError::JsonNumberNotRoundTrippable`] when any
/// number cannot preserve its V1 identity.
pub(crate) fn validate_numbers(value: &Value) -> Result<(), ValueWireEncodeError> {
    let mut frames = Vec::new();
    let mut next = Some(value);
    while next.is_some() || !frames.is_empty() {
        if let Some(node) = next.take() {
            match node {
                Value::Number(number) => validate_number(number)?,
                Value::Array(_) | Value::Object(_) => {
                    frames.push(JsonNumberChildren::new(node).expect("container has child iterator"));
                }
                _ => {}
            }
        }
        while let Some(frame) = frames.last_mut() {
            if let Some(child) = frame.next() {
                next = Some(child);
                break;
            }
            frames.pop();
        }
    }
    Ok(())
}

/// Returns a recursively canonical JSON value for natural projections.
///
/// # Parameters
///
/// * `value` - JSON tree whose object maps are recursively reordered.
///
/// # Returns
///
/// An owned JSON tree with lexicographically ordered object keys.
#[must_use]
#[cfg(feature = "converter")]
pub(crate) fn canonicalize_json_value(value: &Value) -> Value {
    let mut pending = vec![(value, false)];
    let mut completed = Vec::new();
    while let Some((node, ready)) = pending.pop() {
        match node {
            Value::Array(values) if !ready => {
                pending.push((node, true));
                pending.extend(values.iter().rev().map(|child| (child, false)));
            }
            Value::Object(values) if !ready => {
                pending.push((node, true));
                pending.extend(values.values().rev().map(|child| (child, false)));
            }
            Value::Array(values) => {
                let children = completed.split_off(completed.len() - values.len());
                completed.push(Value::Array(children));
            }
            Value::Object(values) => {
                let children = completed.split_off(completed.len() - values.len());
                let mut entries: Vec<_> = values.keys().zip(children).collect();
                entries.sort_unstable_by_key(|(left, _)| *left);
                let mut object = Map::with_capacity(entries.len());
                for (key, child) in entries {
                    object.insert(key.clone(), child);
                }
                completed.push(Value::Object(object));
            }
            _ => completed.push(node.clone()),
        }
    }
    completed.pop().expect("one root is completed")
}
