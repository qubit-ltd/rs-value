// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Multiple Values Container
//!
//! Provides type-safe storage and access functionality for multiple values.

use qubit_datatype::DataType;

/// Defines the public multi-value container from the shared value-type table.
macro_rules! define_multi_values_enum {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        /// Multiple values container.
        ///
        /// Uses an enum to represent multiple values of different types,
        /// providing type-safe storage and access for multiple values.
        ///
        /// This enum is non-exhaustive; downstream matches must include a
        /// wildcard arm so future collection variants remain source-compatible.
        ///
        /// # Behavior
        ///
        /// - Stores a homogeneous collection from the closed [`DataType`]
        ///   family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from a concrete empty vector.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
        /// assert_eq!(values.count(), 3);
        /// assert_eq!(values.get_first_int32().unwrap(), 1);
        ///
        /// let all = values.get_int32s().unwrap();
        /// assert_eq!(all, &[1, 2, 3]);
        ///
        /// values.add(4).unwrap();
        /// assert_eq!(values.count(), 4);
        /// ```
        #[must_use]
        #[non_exhaustive]
        #[derive(Debug, Clone)]
        pub enum MultiValues {
            /// Unset collection with a declared element data type.
            Unset(DataType),
            $(
                $(#[$cfg])*
                #[doc = $multi_doc]
                $variant(Vec<$type>),
            )+
        }
    };
}

for_each_value_type!(define_multi_values_enum);

// ============================================================================
// Getter method generation macros
// ============================================================================

/// Unified multiple values getter generation macro
///
/// Generates `get_[xxx]s` methods for `MultiValues`, returning a reference to
/// value slices.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
macro_rules! impl_get_multi_values {
    // Simple type: return slice reference
    ($(#[$attr:meta])* slice: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::NoValue`] when the container is unset"]
        #[doc = "with the requested type, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs. A concrete empty vector returns"]
        #[doc = "an empty slice."]
        #[inline]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match self {
                MultiValues::$variant(v) => Ok(v),
                MultiValues::Unset(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Complex type: return Vec reference (e.g., Vec<String>, Vec<Vec<u8>>)
    ($(#[$attr:meta])* vec: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::NoValue`] when the container is unset"]
        #[doc = "with the requested type, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs. A concrete empty vector returns"]
        #[doc = "an empty slice."]
        #[inline]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match self {
                MultiValues::$variant(v) => Ok(v.as_slice()),
                MultiValues::Unset(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values get_first method generation macro
///
/// Generates `get_first_[xxx]` methods for `MultiValues`, used to get the first
/// value.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
macro_rules! impl_get_first_value {
    // Copy type: directly return value
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::NoValue`] when the requested type matches"]
        #[doc = "but no value is stored, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs."]
        #[inline]
        pub fn $method(&self) -> ValueResult<$type> {
            match self {
                MultiValues::$variant(v) if !v.is_empty() => Ok(v[0]),
                MultiValues::$variant(_) => Err(ValueError::NoValue),
                MultiValues::Unset(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Reference type: return reference
    ($(#[$attr:meta])* ref: $method:ident, $variant:ident, $ret_type:ty, $data_type:expr, $conversion:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::NoValue`] when the requested type matches"]
        #[doc = "but no value is stored, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs."]
        #[inline]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match self {
                MultiValues::$variant(v) if !v.is_empty() => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(&v[0]))
                },
                MultiValues::$variant(_) => Err(ValueError::NoValue),
                MultiValues::Unset(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}
