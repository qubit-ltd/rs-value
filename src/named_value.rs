// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Named Single Value
//!
//! Provides a named container for single values, allowing readable identifiers
//! to be added to individual values in complex configurations or structures.
//!
//! Suitable for scenarios such as log annotation, configuration item
//! encapsulation, and preserving strongly typed values in key-value pairs.

#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_json::decode::JsonDecoder;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as DeserializeError;
use serde::ser::Error as SerializeError;

use self::internal::NamedValueWireOwned;
use self::internal::NamedValueWireRef;
use super::value::Value;
#[cfg(feature = "json")]
use crate::ValueWireDecodeError;
#[cfg(feature = "json")]
use crate::ValueWireEncodeError;
#[cfg(feature = "json")]
use crate::ValueWireEncodePreflight;
use crate::ValueWireRefV1;
#[cfg(feature = "json")]
use crate::ValueWireV1;

mod internal;

/// Named single value
///
/// Associates a human-readable name with a single [`Value`], facilitating
/// identification, retrieval, and display in configurations, parameter passing,
/// and complex data structures.
///
/// # Features
///
/// - Provides stable name identification for values
/// - Exposes the inner [`Value`] through explicit accessors
/// - Supports `serde` serialization and deserialization
///
/// # Use Cases
///
/// - Configuration item encapsulation (e.g., `"port"`, `"timeout"`, etc.)
/// - Named output of key values in logs/monitoring
/// - Quick location by name in collections
///
/// # Deserialization boundaries
///
/// [`Deserialize`] validates the V1 wire schema and requires a scalar payload,
/// but it does not create a resource budget by itself. For a complete,
/// untrusted JSON document, use `NamedValue::decode_json_slice` or
/// `NamedValue::decode_json_slice_with_limits`. When this type is embedded in a
/// larger document, deserialize it through a resource-bounded outer decoder.
/// Bounded helpers report target-deserialization failures as `InvalidJson`
/// metadata containing only the Serde category and location. Generic Serde
/// errors retain the surrounding deserializer's semantics and may include
/// input-derived text, such as field names. Reader and writer I/O error sources
/// may also contain application-defined details. Resource limits do not
/// sanitize diagnostics; apply the application's logging policy to all error
/// sources.
///
/// # Examples
///
/// ```rust
/// use qubit_value::{NamedValue, Value};
///
/// let named = NamedValue::new("flag", Value::Bool(true));
/// assert!(named.value().get_bool().unwrap());
/// ```
///
/// The wrapper intentionally does not forward [`Value`] methods implicitly:
///
/// ```compile_fail
/// use qubit_value::{NamedValue, Value};
///
/// let named = NamedValue::new("flag", Value::Bool(true));
/// let _ = named.get_bool();
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedValue {
    /// Owned label, stored independently of the value and its runtime type.
    name: String,
    /// Owned scalar payload, retaining its runtime type and unset state.
    value: Value,
}

impl NamedValue {
    /// Create a new named value
    ///
    /// Converts the name into owned storage and moves `value` into the wrapper
    /// without cloning it. The name is not required to be unique or nonempty.
    ///
    /// # Type Parameters
    ///
    /// * `impl Into<String>` - Name source converted into owned storage.
    ///
    /// # Parameters
    ///
    /// * `name` - Label converted into an owned string.
    /// * `value` - Scalar payload whose ownership is transferred to the
    ///   wrapper.
    ///
    /// # Returns
    ///
    /// Returns a newly created [`NamedValue`] instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let named = NamedValue::new("timeout", Value::Int32(30));
    /// assert_eq!(named.name(), "timeout");
    /// ```
    #[inline(always)]
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Decodes a complete named scalar JSON document with default limits.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    ///
    /// # Returns
    ///
    /// The decoded named scalar.
    ///
    /// # Errors
    ///
    /// Returns a JSON, wire-contract, or resource-limit error.
    #[cfg(feature = "json")]
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn decode_json_slice(input: &[u8]) -> Result<Self, ValueWireDecodeError> {
        Self::decode_json_slice_with_limits(input, ValueWireV1::default_json_decode_limits())
    }

    /// Decodes a complete named scalar JSON document with explicit limits.
    ///
    /// The wrapper name and nested scalar share one accounting session.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    /// * `limits` - Input and decoded-resource limits.
    ///
    /// # Returns
    ///
    /// The decoded named scalar.
    ///
    /// # Errors
    ///
    /// Returns a JSON, wire-contract, or resource-limit error.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: JsonDecodeLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        let session = JsonDecodeSession::from_limits(limits);
        JsonDecoder::new(session)
            .decode_unique_utf8(input)
            .map_err(ValueWireDecodeError::from)
    }

    /// Encodes this named scalar into a bounded compact JSON vector with the
    /// default V1 JSON resource profile.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for the complete named scalar.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] for resource or serialization failures.
    #[cfg(feature = "json")]
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
    }

    /// Encodes this named scalar into a bounded compact JSON vector.
    ///
    /// # Parameters
    ///
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for the complete named scalar.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits` or the
    /// named scalar cannot be serialized.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        ValueWireEncodePreflight::new(limits).check_value(self.value())?;
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .to_vec(self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Encodes this named scalar to a writer with the default V1 JSON profile.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete named scalar document.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete document is written.
    ///
    /// The complete document is buffered and admitted before this method writes
    /// its bytes to the destination. Preflight, serialization, or budget
    /// rejection does not write this document. An I/O error may leave an
    /// accepted prefix in the writer; do not blindly retry a complete
    /// document on the same destination.
    ///
    /// This method does not explicitly flush the writer. Success means the
    /// writer accepted all document bytes, not that they were flushed,
    /// persisted, or received by a remote peer. Pass `&mut writer` to
    /// retain ownership of a buffered writer, then call `flush()` and
    /// handle its error separately. Passing an owned writer may invoke that
    /// writer's drop behavior; errors from an implicit drop-time flush are
    /// not returned by this method.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] for resource, serialization, or writer
    /// failures.
    #[cfg(feature = "json")]
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn to_json_writer<W>(&self, writer: W) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(writer, ValueWireV1::default_json_encode_limits())
    }

    /// Encodes this named scalar to a writer after enforcing JSON budgets.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete named scalar document.
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete document is written.
    ///
    /// The complete document is buffered and admitted before this method writes
    /// its bytes to the destination. Preflight, serialization, or budget
    /// rejection does not write this document. An I/O error may leave an
    /// accepted prefix in the writer; do not blindly retry a complete
    /// document on the same destination.
    ///
    /// This method does not explicitly flush the writer. Success means the
    /// writer accepted all document bytes, not that they were flushed,
    /// persisted, or received by a remote peer. Pass `&mut writer` to
    /// retain ownership of a buffered writer, then call `flush()` and
    /// handle its error separately. Passing an owned writer may invoke that
    /// writer's drop behavior; errors from an implicit drop-time flush are
    /// not returned by this method.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits`, the
    /// named scalar cannot be serialized, or `writer` rejects output.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn to_json_writer_with_limits<W>(
        &self,
        mut writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        ValueWireEncodePreflight::new(limits).check_value(self.value())?;
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(&mut writer, self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Get a reference to the name
    ///
    /// Borrows the stored label without allocating. The slice remains valid
    /// while this wrapper is shared-borrowed, preventing name replacement
    /// through the wrapper during that borrow.
    ///
    /// # Returns
    ///
    /// Returns a string slice `&str` of the name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let named = NamedValue::new("host", Value::String("localhost".to_string()));
    /// assert_eq!(named.name(), "host");
    /// ```
    #[inline(always)]
    #[must_use = "the borrowed name should be used"]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Borrows the contained value without cloning its payload.
    ///
    /// The reference is tied to the shared borrow of this wrapper; mutation
    /// through the wrapper requires that borrow to end.
    ///
    /// # Returns
    ///
    /// A shared reference to the contained [`Value`].
    #[inline(always)]
    #[must_use = "the borrowed value should be used"]
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Borrows the contained value exclusively, allowing in-place updates.
    ///
    /// The reference is tied to the exclusive borrow of this wrapper. Updates
    /// can change the payload or its runtime type without changing the name.
    ///
    /// # Returns
    ///
    /// An exclusive reference to the contained [`Value`].
    #[inline]
    #[must_use = "the mutable value reference should be used"]
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Set a new name
    ///
    /// Converts the new label into owned storage and drops the previous name,
    /// leaving the contained value unchanged. Empty and duplicate labels are
    /// accepted without validation.
    ///
    /// # Type Parameters
    ///
    /// * `impl Into<String>` - Name source converted into owned storage.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let mut named = NamedValue::new("old_name", Value::Bool(true));
    /// named.set_name("new_name");
    /// assert_eq!(named.name(), "new_name");
    /// ```
    #[inline(always)]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Moves a new value into this wrapper and drops the previous payload.
    ///
    /// The replacement may change the runtime type or unset state; the stored
    /// name is preserved and the new payload is not cloned.
    ///
    /// # Parameters
    ///
    /// * `value` - New value to store under the existing name.
    #[inline]
    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    /// Consumes this wrapper and returns its owned name and value.
    ///
    /// # Returns
    ///
    /// The `(name, value)` pair without cloning either component.
    #[inline]
    #[must_use = "consuming NamedValue without using its parts loses both fields"]
    pub fn into_parts(self) -> (String, Value) {
        (self.name, self.value)
    }
}

impl Serialize for NamedValue {
    /// Serializes the name and its explicitly versioned scalar value.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Serde serializer that determines the output format.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Destination serializer receiving the named V1 scalar
    ///   envelope.
    ///
    /// # Returns
    ///
    /// The serializer's success value after serializing the complete envelope.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` when the scalar cannot be represented by the V1 wire
    /// contract or the serializer rejects the envelope.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = ValueWireRefV1::try_from(self.value()).map_err(SerializeError::custom)?;
        NamedValueWireRef {
            name: self.name(),
            value,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NamedValue {
    /// Deserializes a named scalar value from the V1 wire contract.
    ///
    /// This implementation validates the wire schema and scalar shape, but
    /// inherits resource accounting from `deserializer`. Callers handling a
    /// complete, untrusted JSON document should use the bounded JSON helpers on
    /// [`NamedValue`] instead of an unbounded Serde entry point.
    /// Generic deserializer errors may contain input-derived text. The bounded
    /// helpers instead expose only Serde category and location for target
    /// deserialization failures; resource limits do not sanitize other errors.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Serde deserializer that supplies the input and any outer budget.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source containing one named V1 scalar envelope.
    ///
    /// # Returns
    ///
    /// The decoded name and scalar value.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` for an invalid V1 envelope, an unsupported payload,
    /// or a payload whose shape is not scalar.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let NamedValueWireOwned { name, value } = NamedValueWireOwned::deserialize(deserializer)?;
        let value = value.into_container().into_scalar().map_err(|_| {
            DeserializeError::custom("named value wire payload must contain a scalar")
        })?;
        Ok(Self::new(name, value))
    }
}
