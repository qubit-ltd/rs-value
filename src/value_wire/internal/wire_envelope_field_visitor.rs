// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! String and UTF-8 byte visitor for one fixed Wire field set.

use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;

use super::wire_envelope_field::WireEnvelopeField;

/// Deserializes string representations of fixed field names.
pub(super) struct WireEnvelopeFieldVisitor;

impl<'de> Visitor<'de> for WireEnvelopeFieldVisitor {
    /// The fixed field identifier produced by this visitor.
    type Value = WireEnvelopeField;

    /// Describes the accepted field-name representation.
    ///
    /// # Parameters
    ///
    /// - `formatter`: The formatter receiving the expected representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing the description fails.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    /// Classifies a borrowed string field name.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The deserializer error type.
    ///
    /// # Parameters
    ///
    /// - `value`: The decoded field name.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier.
    ///
    /// # Errors
    ///
    /// Returns an unknown-field error when `value` is not declared here.
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match value {
            "version" => Ok(WireEnvelopeField::Version),
            "value" => Ok(WireEnvelopeField::Value),
            _ => Err(E::unknown_field(value, &["version", "value"])),
        }
    }

    /// Classifies a deserializer-borrowed string field name.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The deserializer error type.
    ///
    /// # Parameters
    ///
    /// - `value`: The decoded field name borrowed from the input.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier.
    ///
    /// # Errors
    ///
    /// Returns an unknown-field error when `value` is not declared here.
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value)
    }

    /// Classifies an owned string field name.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The deserializer error type.
    ///
    /// # Parameters
    ///
    /// - `value`: The decoded field name.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier; the owned input is dropped after
    /// classification.
    ///
    /// # Errors
    ///
    /// Returns an unknown-field error when `value` is not declared here.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(&value)
    }

    /// Classifies a UTF-8 byte string using the former String key behavior.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The deserializer error type.
    ///
    /// # Parameters
    ///
    /// - `value`: The bytes supplied by the deserializer.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier for valid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an invalid-value error for invalid UTF-8 or an unknown-field
    /// error for an undeclared valid string.
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match std::str::from_utf8(value) {
            Ok(value) => self.visit_str(value),
            Err(_) => Err(E::invalid_value(Unexpected::Bytes(value), &self)),
        }
    }

    /// Classifies an owned UTF-8 byte string.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The deserializer error type.
    ///
    /// # Parameters
    ///
    /// - `value`: The byte buffer supplied by the deserializer.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier for valid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an invalid-value error for invalid UTF-8 or an unknown-field
    /// error for an undeclared valid string.
    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match String::from_utf8(value) {
            Ok(value) => self.visit_str(&value),
            Err(error) => Err(E::invalid_value(
                Unexpected::Bytes(&error.into_bytes()),
                &self,
            )),
        }
    }
}
