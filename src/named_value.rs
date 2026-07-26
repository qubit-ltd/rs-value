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

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::ValueWireV1;

use super::value::Value;

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
    /// Name of the value
    name: String,
    /// Content of the value
    value: Value,
}

/// Borrowed wire representation of a named scalar value.
#[derive(Serialize)]
struct NamedValueWireRef<'a> {
    /// Name associated with the scalar value.
    name: &'a str,
    /// Independently versioned scalar value.
    value: ValueWireV1,
}

/// Owned wire representation of a named scalar value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedValueWireOwned {
    /// Name associated with the scalar value.
    name: String,
    /// Independently versioned scalar value.
    value: ValueWireV1,
}

impl NamedValue {
    /// Create a new named value
    ///
    /// Creates a binding instance between a name and a value.
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the value
    /// * `value` - Content of the value
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
    #[inline]
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Get a reference to the name
    ///
    /// Returns a read-only name slice bound to this value.
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

    /// Set a new name
    ///
    /// Updates the name bound to the current instance.
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

    /// Borrows the contained value.
    ///
    /// # Returns
    ///
    /// A shared reference to the contained [`Value`].
    #[inline(always)]
    #[must_use = "the borrowed value should be used"]
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Mutably borrows the contained value.
    ///
    /// # Returns
    ///
    /// An exclusive reference to the contained [`Value`].
    #[inline(always)]
    #[must_use = "the mutable value reference should be used"]
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Replaces the contained value.
    ///
    /// # Parameters
    ///
    /// * `value` - New value to store under the existing name.
    #[inline(always)]
    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }

    /// Consumes this wrapper and returns its owned name and value.
    ///
    /// # Returns
    ///
    /// The `(name, value)` pair without cloning either component.
    #[inline(always)]
    #[must_use = "consuming NamedValue without using its parts loses both fields"]
    pub fn into_parts(self) -> (String, Value) {
        (self.name, self.value)
    }
}

impl Serialize for NamedValue {
    /// Serializes the name and its explicitly versioned scalar value.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = ValueWireV1::try_from(self.value.clone())
            .map_err(serde::ser::Error::custom)?;
        NamedValueWireRef {
            name: self.name(),
            value,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NamedValue {
    /// Deserializes a named scalar value from the V1 wire contract.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let NamedValueWireOwned { name, value } =
            NamedValueWireOwned::deserialize(deserializer)?;
        let value = value.into_container().into_scalar().map_err(|_| {
            serde::de::Error::custom(
                "named value wire payload must contain a scalar",
            )
        })?;
        Ok(Self::new(name, value))
    }
}
