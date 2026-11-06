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
#[cfg(feature = "converter")]
use super::internal::CanonicalJsonFrame;
#[cfg(feature = "json")]
use super::internal::FixedNumberText;
use super::internal::JsonNumberChildren;
#[cfg(feature = "json")]
use super::internal::NumberTextComparator;
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
#[inline]
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
#[inline]
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    DuplicateKeyRejectingJsonValue::deserialize(deserializer).map(DuplicateKeyRejectingJsonValue::into_inner)
}

/// Checks whether a JSON number survives the V1 decoder's number model
/// without changing its `serde_json::Number` identity.
///
/// # Parameters
///
/// * `number` - JSON number whose V1 representation must preserve its identity.
///
/// # Returns
///
/// `Ok(())` when the number round-trips through the V1 number model unchanged.
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
    let mut text = FixedNumberText::new();
    if write!(&mut text, "{number}").is_ok() {
        validate_number_text(number, text.as_str())
    } else {
        validate_number_text(number, &number.to_string())
    }
}

/// Reconstructs the V1 number model from text and checks number identity.
///
/// # Parameters
///
/// * `number` - Original JSON number to compare with the reconstructed value.
/// * `text` - Decimal representation used to reconstruct the V1 number.
///
/// # Returns
///
/// `Ok(())` when the reconstructed number equals `number`.
///
/// # Errors
///
/// Returns `JsonNumberNotRoundTrippable` when the canonical model differs.
fn validate_number_text(number: &Number, text: &str) -> Result<(), ValueWireEncodeError> {
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

/// Compares a JSON number's text with the decimal text of an integer.
///
/// The expected integer is formatted into a fixed-capacity stack buffer, then
/// the JSON number is written into a comparator that checks each text fragment
/// byte-for-byte. Equality requires every expected byte to be consumed.
///
/// # Type Parameters
///
/// * `T` - Integer type whose `Display` implementation produces decimal text.
///
/// # Parameters
///
/// * `number` - JSON number whose textual identity is checked.
/// * `integer` - Integer value exposed by the JSON number API.
///
/// # Returns
///
/// `true` when formatting succeeds and the complete byte sequences match.
#[must_use]
fn number_matches_integer<T: fmt::Display>(number: &Number, integer: T) -> bool {
    let mut expected = FixedNumberText::new();
    if write!(&mut expected, "{integer}").is_err() {
        return false;
    }
    let mut comparator = NumberTextComparator::new(expected.as_bytes());
    write!(&mut comparator, "{number}").is_ok() && comparator.is_equal()
}

/// Checks every number in a JSON tree using iterator frames bounded by depth.
///
/// # Parameters
///
/// * `value` - Borrowed JSON tree whose numbers must preserve their V1
///   identity.
///
/// # Returns
///
/// `Ok(())` when every number round-trips unchanged, including trees without
/// numbers.
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
///
/// Traversal is iterative. The frame stack and object sort entries are
/// scheduling storage; array children are transferred directly into the final
/// output array, so wide arrays do not accumulate a second child-value buffer.
#[must_use]
#[cfg(feature = "converter")]
pub(crate) fn canonicalize_json_value(value: &Value) -> Value {
    let mut frames = Vec::new();
    let mut next = Some(value);
    let mut completed = None;
    loop {
        if let Some(node) = next.take() {
            if let Some(mut frame) = CanonicalJsonFrame::new(node) {
                next = frame.next_child();
                frames.push(frame);
                continue;
            }
            completed = Some(match node {
                Value::Array(_) => Value::Array(Vec::new()),
                Value::Object(_) => Value::Object(Map::new()),
                _ => node.clone(),
            });
        }
        if let Some(child) = completed.take() {
            if let Some(parent) = frames.last_mut() {
                parent.push(child);
            } else {
                return child;
            }
        }
        let Some(frame) = frames.last_mut() else {
            unreachable!("the root value is returned when its frame completes");
        };
        if let Some(child) = frame.next_child() {
            next = Some(child);
        } else {
            completed = Some(frames.pop().expect("last frame exists").finish());
        }
    }
}
