/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Named Single Value
//!
//! Provides a named container for single values, allowing readable identifiers
//! to be added to individual values in complex configurations or structures.
//!
//! Suitable for scenarios such as log annotation, configuration item encapsulation,
//! and preserving strongly typed values in key-value pairs.
//!
//! # Author
//!
//! Haixing Hu

use serde::{
    Deserialize,
    Serialize,
};
use std::ops::{
    Deref,
    DerefMut,
};

use super::value::Value;

/// Named single value
///
/// Associates a human-readable name with a single [`Value`], facilitating identification,
/// retrieval, and display in configurations, parameter passing, and complex data structures.
///
/// # Features
///
/// - Provides stable name identification for values
/// - Automatically dereferences to the inner [`Value`] via `Deref`, allowing direct access to [`Value`] methods
/// - Supports `serde` serialization and deserialization
///
/// # Use Cases
///
/// - Configuration item encapsulation (e.g., `"port"`, `"timeout"`, etc.)
/// - Named output of key values in logs/monitoring
/// - Quick location by name in collections
///
/// # Example
///
/// ```rust
/// use qubit_value::{NamedValue, Value};
///
/// let named = NamedValue::new("flag", Value::Bool(true));
/// // Call Value methods through Deref
/// assert_eq!(named.to::<bool>().unwrap(), true);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedValue {
    /// Name of the value
    name: String,
    /// Content of the value
    value: Value,
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
    /// # Example
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
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let named = NamedValue::new("host", Value::String("localhost".to_string()));
    /// assert_eq!(named.name(), "host");
    /// ```
    #[inline]
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
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let mut named = NamedValue::new("old_name", Value::Bool(true));
    /// named.set_name("new_name");
    /// assert_eq!(named.name(), "new_name");
    /// ```
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Consume the instance and return `(name, value)`.
    #[inline]
    pub fn into_parts(self) -> (String, Value) {
        (self.name, self.value)
    }
}

impl Deref for NamedValue {
    type Target = Value;

    /// Dereference to the inner [`Value`]
    ///
    /// Allows direct invocation of methods on [`Value`], for example: `named.to::<i32>()`.
    ///
    /// # Returns
    ///
    /// Returns an immutable reference `&Value` to the inner value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::{NamedValue, Value};
    ///
    /// let named = NamedValue::new("flag", Value::Bool(true));
    /// // Call Value methods through Deref
    /// assert_eq!(named.to::<bool>().unwrap(), true);
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for NamedValue {
    /// Mutable dereference to the inner [`Value`]
    ///
    /// Allows in-place modification of the inner value (provided [`Value`] itself offers corresponding mutable methods).
    ///
    /// # Returns
    ///
    /// Returns a mutable reference `&mut Value` to the inner value.
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
