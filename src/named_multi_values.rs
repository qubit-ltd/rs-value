// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Named Multiple Values
//!
//! Provides a lightweight container for binding names to multiple value
//! collections, facilitating human-readable identification of groups of values
//! in configurations, serialization, logging, and other scenarios.

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

use super::multi_values::MultiValues;
use super::named_value::NamedValue;
#[cfg(feature = "json")]
use crate::ValueWireDecodeError;
#[cfg(feature = "json")]
use crate::ValueWireEncodeError;
use crate::ValueWireRefV1;
#[cfg(feature = "json")]
use crate::ValueWireV1;

mod internal;

use self::internal::NamedMultiValuesWireOwned;
use self::internal::NamedMultiValuesWireRef;

/// Named multiple values
///
/// A container that associates a readable name with a set of `MultiValues`,
/// suitable for organizing data in key-value (name-multiple values) scenarios,
/// such as configuration items, command-line parameter aggregation, structured
/// log fields, etc.
///
/// # Features
///
/// - Provides clear name identification for multiple value collections
/// - Exposes the inner [`MultiValues`] through explicit accessors
/// - Supports `serde` serialization and deserialization
///
/// # Use Cases
///
/// - Aggregating a set of ports, hostnames, etc., as semantically meaningful
///   fields
/// - Outputting named multiple value lists in configurations/logs
///
/// # Examples
///
/// ```rust
/// use qubit_value::{NamedMultiValues, MultiValues};
///
/// // Identify a group of ports with the name "ports"
/// let named = NamedMultiValues::new(
///     "ports",
///     MultiValues::Int32(vec![8080, 8081, 8082])
/// );
///
/// assert_eq!(named.name(), "ports");
/// assert_eq!(named.values().len(), 3);
/// ```
///
/// The wrapper intentionally does not forward [`MultiValues`] methods
/// implicitly:
///
/// ```compile_fail
/// use qubit_value::{MultiValues, NamedMultiValues};
///
/// let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080]));
/// let _ = named.len();
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedMultiValues {
    /// Name of the values
    name: String,
    /// Content of the multiple values
    value: MultiValues,
}

impl NamedMultiValues {
    /// Create a new named multiple values
    ///
    /// Associates a given name with `MultiValues`, generating a container that
    /// can be referenced by name.
    ///
    /// # Use Cases
    ///
    /// - Building configuration fields (e.g., `servers`, `ports`, etc.)
    /// - Binding parsed multiple value results to semantic names
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the multiple values
    /// * `value` - Content of the multiple values
    ///
    /// # Returns
    ///
    /// Returns a newly created named multiple values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedMultiValues, MultiValues};
    ///
    /// let named = NamedMultiValues::new(
    ///     "servers",
    ///     MultiValues::String(vec!["s1".to_string(), "s2".to_string()])
    /// );
    /// assert_eq!(named.name(), "servers");
    /// ```
    #[inline]
    pub fn new(name: impl Into<String>, value: MultiValues) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Decodes a complete named collection JSON document with default limits.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    ///
    /// # Returns
    ///
    /// The decoded named collection.
    ///
    /// # Errors
    ///
    /// Returns a JSON, wire-contract, or resource-limit error.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, ValueWireDecodeError> {
        Self::decode_json_slice_with_limits(
            input,
            ValueWireV1::default_json_decode_limits(),
        )
    }

    /// Decodes a complete named collection JSON document with explicit limits.
    ///
    /// The wrapper name and nested collection share one accounting session.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    /// * `limits` - Input and decoded-resource limits.
    ///
    /// # Returns
    ///
    /// The decoded named collection.
    ///
    /// # Errors
    ///
    /// Returns a JSON, wire-contract, or resource-limit error.
    #[cfg(feature = "json")]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: JsonDecodeLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        let session = JsonDecodeSession::from_limits(limits);
        JsonDecoder::new(session)
            .decode_utf8(input)
            .map_err(ValueWireDecodeError::from)
    }

    /// Encodes this named collection into a bounded compact JSON vector with
    /// the default V1 JSON resource profile.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
    }

    /// Encodes this named collection into a bounded compact JSON vector.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .to_vec(self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Encodes this named collection to a writer with the default V1 JSON
    /// profile.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            ValueWireV1::default_json_encode_limits(),
        )
    }

    /// Encodes this named collection to a writer after enforcing JSON budgets.
    #[cfg(feature = "json")]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(writer, self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Get a reference to the name
    ///
    /// # Returns
    ///
    /// Returns a string slice of the name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedMultiValues, MultiValues};
    ///
    /// let named = NamedMultiValues::new("items", MultiValues::Int32(vec![1, 2, 3]));
    /// assert_eq!(named.name(), "items");
    /// ```
    #[inline(always)]
    #[must_use = "the borrowed name should be used"]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set a new name
    ///
    /// # Parameters
    ///
    /// * `name` - The new name
    ///
    /// # Returns
    ///
    /// No return value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::{NamedMultiValues, MultiValues};
    ///
    /// let mut named = NamedMultiValues::new("old", MultiValues::Bool(vec![true]));
    /// named.set_name("new");
    /// assert_eq!(named.name(), "new");
    /// ```
    #[inline(always)]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Borrows the contained values.
    ///
    /// # Returns
    ///
    /// A shared reference to the contained [`MultiValues`].
    #[inline(always)]
    #[must_use = "the borrowed values should be used"]
    pub fn values(&self) -> &MultiValues {
        &self.value
    }

    /// Mutably borrows the contained values.
    ///
    /// # Returns
    ///
    /// An exclusive reference to the contained [`MultiValues`].
    #[inline(always)]
    #[must_use = "the mutable values reference should be used"]
    pub fn values_mut(&mut self) -> &mut MultiValues {
        &mut self.value
    }

    /// Replaces the contained values.
    ///
    /// # Parameters
    ///
    /// * `values` - New collection to store under the existing name.
    #[inline(always)]
    pub fn set_values(&mut self, values: MultiValues) {
        self.value = values;
    }

    /// Consumes this wrapper and returns its owned name and values.
    ///
    /// # Returns
    ///
    /// The `(name, values)` pair without cloning either component.
    #[inline(always)]
    #[must_use = "consuming NamedMultiValues without using its parts loses both fields"]
    pub fn into_parts(self) -> (String, MultiValues) {
        (self.name, self.value)
    }

    /// Convert this named multi-values into a named single value.
    ///
    /// The returned value keeps the same name and uses the first element from
    /// the inner [`MultiValues`]. If there is no element, the returned value is
    /// `Value::Unset` with the same data type.
    ///
    /// # Returns
    ///
    /// A named clone of the first item, or a named typed unset value.
    #[inline]
    pub fn first_named_value(&self) -> NamedValue {
        NamedValue::new(self.name.as_str(), self.value.first_value())
    }

    /// Consumes this container and converts its first item to a named value.
    ///
    /// The owned name and first stored item are moved into the result. An empty
    /// or unset collection produces [`crate::Value::Unset`] with the same data
    /// type.
    ///
    /// # Returns
    ///
    /// A named owned first item, or a named typed unset value.
    #[inline]
    pub fn into_first_named_value(self) -> NamedValue {
        let (name, values) = self.into_parts();
        NamedValue::new(name, values.into_first_value())
    }
}

impl From<NamedValue> for NamedMultiValues {
    /// Construct `NamedMultiValues` from `NamedValue`
    ///
    /// Reuses the name and promotes the single value to a `MultiValues`
    /// containing only one element.
    #[inline]
    fn from(named: NamedValue) -> Self {
        let (name, value) = named.into_parts();
        let value = MultiValues::from(value);
        Self { name, value }
    }
}

impl Serialize for NamedMultiValues {
    /// Serializes the name and its explicitly versioned collection.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = ValueWireRefV1::try_from(self.values())
            .map_err(SerializeError::custom)?;
        NamedMultiValuesWireRef {
            name: self.name(),
            value,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NamedMultiValues {
    /// Deserializes a named collection from the V1 wire contract.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let NamedMultiValuesWireOwned { name, value } =
            NamedMultiValuesWireOwned::deserialize(deserializer)?;
        let value = value.into_container().into_collection().map_err(|_| {
            DeserializeError::custom(
                "named multi-values wire payload must contain a collection",
            )
        })?;
        Ok(Self::new(name, value))
    }
}
