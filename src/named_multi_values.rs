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

use serde::{
    Deserialize,
    Serialize,
};

use super::multi_values::MultiValues;
use super::named_value::NamedValue;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
