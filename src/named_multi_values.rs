/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Named Multiple Values
//!
//! Provides a lightweight container for binding names to multiple value collections,
//! facilitating human-readable identification of groups of values in configurations,
//! serialization, logging, and other scenarios.
//!

use serde::{
    Deserialize,
    Serialize,
};
use std::ops::{
    Deref,
    DerefMut,
};

use super::multi_values::MultiValues;
use super::named_value::NamedValue;

/// Named multiple values
///
/// A container that associates a readable name with a set of `MultiValues`, suitable for
/// organizing data in key-value (name-multiple values) scenarios, such as configuration items,
/// command-line parameter aggregation, structured log fields, etc.
///
/// # Features
///
/// - Provides clear name identification for multiple value collections
/// - Transparently reuses all capabilities of `MultiValues` through `Deref/DerefMut`
/// - Supports `serde` serialization and deserialization
///
/// # Use Cases
///
/// - Aggregating a set of ports, hostnames, etc., as semantically meaningful fields
/// - Outputting named multiple value lists in configurations/logs
///
/// # Example
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
/// assert_eq!(named.count(), 3);
/// ```
///
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedMultiValues {
    /// Name of the values
    name: String,
    /// Content of the multiple values
    value: MultiValues,
}

impl NamedMultiValues {
    /// Create a new named multiple values
    ///
    /// Associates a given name with `MultiValues`, generating a container that can be referenced by name.
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
    /// # Example
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
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::{NamedMultiValues, MultiValues};
    ///
    /// let named = NamedMultiValues::new("items", MultiValues::Int32(vec![1, 2, 3]));
    /// assert_eq!(named.name(), "items");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    // Methods of MultiValues are forwarded through Deref/DerefMut

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
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::{NamedMultiValues, MultiValues};
    ///
    /// let mut named = NamedMultiValues::new("old", MultiValues::Bool(vec![true]));
    /// named.set_name("new");
    /// assert_eq!(named.name(), "new");
    /// ```
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Convert this named multi-values into a named single value.
    ///
    /// The returned value keeps the same name and uses the first element from
    /// the inner [`MultiValues`]. If there is no element, the returned value is
    /// `Value::Empty` with the same data type.
    #[inline]
    pub fn to_named_value(&self) -> NamedValue {
        NamedValue::new(self.name.as_str(), self.value.to_value())
    }

    // Values can be directly assigned or mutable methods called on the inner value through DerefMut
}

/// Transparently delegate read-only methods to the inner `MultiValues` through `Deref`.
impl Deref for NamedMultiValues {
    type Target = MultiValues;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Transparently delegate mutable methods to the inner `MultiValues` through `DerefMut`.
impl DerefMut for NamedMultiValues {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl From<NamedValue> for NamedMultiValues {
    /// Construct `NamedMultiValues` from `NamedValue`
    ///
    /// Reuses the name and promotes the single value to a `MultiValues` containing only one element.
    #[inline]
    fn from(named: NamedValue) -> Self {
        let (name, value) = named.into_parts();
        let value = MultiValues::from(value);
        Self { name, value }
    }
}
