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
use std::fmt;

/// Defines the private storage representation for the public multi-value
/// container from the shared value-type table.
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
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        /// Internal multiple-values representation.
        ///
        /// Uses an enum to represent multiple values of different types,
        /// providing type-safe storage and access for multiple values.
        ///
        /// This representation is private; downstream code uses
        /// [`MultiValues`] constructors and [`MultiValuesRef`] semantic views
        /// instead of matching storage details.
        ///
        /// # Behavior
        ///
        /// - Stores a homogeneous collection from the closed [`DataType`]
        ///   family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from a concrete empty vector.
        ///
        /// # Equality and hashing
        ///
        /// Equality preserves the collection variant and element order. Float
        /// elements use canonical signed-zero and NaN identity, while map-like
        /// elements hash structurally. Standard hash output is suitable for in-memory
        /// keys but is not a stable persistent fingerprint.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
        /// assert_eq!(values.len(), 3);
        /// assert_eq!(values.get_first_int32().unwrap(), 1);
        ///
        /// let all = values.get_int32s().unwrap();
        /// assert_eq!(all, &[1, 2, 3]);
        ///
        /// values.add(4).unwrap();
        /// assert_eq!(values.len(), 4);
        /// ```
        #[derive(Debug, Clone)]
        pub(crate) enum MultiValuesRepr {
            /// Unset collection with a declared element data type.
            Unset(
                /// Declared element type retained while the collection is unset.
                DataType,
            ),
            $(
                $(#[$cfg])*
                #[doc = $multi_doc]
                $variant(
                    #[doc = concat!("Stored ", $multi_doc, " payload.")]
                    Vec<$type>,
                ),
            )+
        }
    };
}

for_each_value_type!(define_multi_values_enum);

/// Multiple typed runtime values with private storage representation.
#[must_use]
#[derive(Clone)]
pub struct MultiValues {
    pub(crate) repr: MultiValuesRepr,
}

impl fmt::Debug for MultiValues {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.view().fmt(formatter)
    }
}

/// Borrowed semantic view of a [`MultiValues`] value.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum MultiValuesRef<'a> {
    /// An unset collection retaining its declared element type.
    Unset(DataType),
    /// A borrowed homogeneous collection.
    Bool(&'a [bool]),
    /// A borrowed homogeneous collection.
    Char(&'a [char]),
    /// A borrowed homogeneous collection.
    Int8(&'a [i8]),
    /// A borrowed homogeneous collection.
    Int16(&'a [i16]),
    /// A borrowed homogeneous collection.
    Int32(&'a [i32]),
    /// A borrowed homogeneous collection.
    Int64(&'a [i64]),
    /// A borrowed homogeneous collection.
    Int128(&'a [i128]),
    /// A borrowed homogeneous collection.
    UInt8(&'a [u8]),
    /// A borrowed homogeneous collection.
    UInt16(&'a [u16]),
    /// A borrowed homogeneous collection.
    UInt32(&'a [u32]),
    /// A borrowed homogeneous collection.
    UInt64(&'a [u64]),
    /// A borrowed homogeneous collection.
    UInt128(&'a [u128]),
    /// A borrowed homogeneous collection.
    Float32(&'a [f32]),
    /// A borrowed homogeneous collection.
    Float64(&'a [f64]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-integer")]
    BigInteger(&'a [num_bigint::BigInt]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-decimal")]
    BigDecimal(&'a [bigdecimal::BigDecimal]),
    /// A borrowed homogeneous collection.
    String(&'a [String]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Date(&'a [chrono::NaiveDate]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Time(&'a [chrono::NaiveTime]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    DateTime(&'a [chrono::NaiveDateTime]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Instant(&'a [chrono::DateTime<chrono::Utc>]),
    /// A borrowed homogeneous collection.
    Duration(&'a [std::time::Duration]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "url")]
    Url(&'a [url::Url]),
    /// A borrowed homogeneous collection.
    StringMap(&'a [std::collections::HashMap<String, String>]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "json")]
    Json(&'a [serde_json::Value]),
}

macro_rules! impl_multi_values_constructors {
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
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        impl MultiValues {
            /// Creates an unset collection with an explicit element type.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub const fn Unset(data_type: DataType) -> Self {
                Self::new_unset(data_type)
            }

            /// Creates an unset collection with an explicit element type.
            #[inline(always)]
            pub const fn new_unset(data_type: DataType) -> Self {
                Self { repr: MultiValuesRepr::Unset(data_type) }
            }

            $(
                $(#[$cfg])*
                #[allow(non_snake_case)]
                #[doc = concat!("Creates a collection of ", $multi_doc, ".")]
                #[inline(always)]
                pub fn $variant(values: Vec<$type>) -> Self {
                    Self { repr: MultiValuesRepr::$variant(values) }
                }
            )+
        }
    };
}

for_each_value_type!(impl_multi_values_constructors);

impl MultiValues {
    /// Borrows the stable semantic view of this collection.
    #[inline(always)]
    pub fn view(&self) -> MultiValuesRef<'_> {
        match &self.repr {
            MultiValuesRepr::Unset(data_type) => MultiValuesRef::Unset(*data_type),
            MultiValuesRepr::Bool(values) => MultiValuesRef::Bool(values),
            MultiValuesRepr::Char(values) => MultiValuesRef::Char(values),
            MultiValuesRepr::Int8(values) => MultiValuesRef::Int8(values),
            MultiValuesRepr::Int16(values) => MultiValuesRef::Int16(values),
            MultiValuesRepr::Int32(values) => MultiValuesRef::Int32(values),
            MultiValuesRepr::Int64(values) => MultiValuesRef::Int64(values),
            MultiValuesRepr::Int128(values) => MultiValuesRef::Int128(values),
            MultiValuesRepr::UInt8(values) => MultiValuesRef::UInt8(values),
            MultiValuesRepr::UInt16(values) => MultiValuesRef::UInt16(values),
            MultiValuesRepr::UInt32(values) => MultiValuesRef::UInt32(values),
            MultiValuesRepr::UInt64(values) => MultiValuesRef::UInt64(values),
            MultiValuesRepr::UInt128(values) => MultiValuesRef::UInt128(values),
            MultiValuesRepr::Float32(values) => MultiValuesRef::Float32(values),
            MultiValuesRepr::Float64(values) => MultiValuesRef::Float64(values),
            #[cfg(feature = "big-integer")]
            MultiValuesRepr::BigInteger(values) => MultiValuesRef::BigInteger(values),
            #[cfg(feature = "big-decimal")]
            MultiValuesRepr::BigDecimal(values) => MultiValuesRef::BigDecimal(values),
            MultiValuesRepr::String(values) => MultiValuesRef::String(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Date(values) => MultiValuesRef::Date(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Time(values) => MultiValuesRef::Time(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::DateTime(values) => MultiValuesRef::DateTime(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Instant(values) => MultiValuesRef::Instant(values),
            MultiValuesRepr::Duration(values) => MultiValuesRef::Duration(values),
            #[cfg(feature = "url")]
            MultiValuesRepr::Url(values) => MultiValuesRef::Url(values),
            MultiValuesRepr::StringMap(values) => MultiValuesRef::StringMap(values),
            #[cfg(feature = "json")]
            MultiValuesRepr::Json(values) => MultiValuesRef::Json(values),
        }
    }
}

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
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match &self.repr {
                MultiValuesRepr::$variant(v) => Ok(v),
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::NoValue($crate::ValueAbsence::UnsetCollection {
                        data_type: *dt,
                    }))
                }
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
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match &self.repr {
                MultiValuesRepr::$variant(v) => Ok(v.as_slice()),
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::NoValue($crate::ValueAbsence::UnsetCollection {
                        data_type: *dt,
                    }))
                }
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
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$type> {
            match &self.repr {
                MultiValuesRepr::$variant(v) if !v.is_empty() => Ok(v[0]),
                MultiValuesRepr::$variant(_) => {
                    Err(ValueError::NoValue($crate::ValueAbsence::EmptyCollection {
                        data_type: $data_type,
                    }))
                }
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::NoValue($crate::ValueAbsence::UnsetCollection {
                        data_type: *dt,
                    }))
                }
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
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match &self.repr {
                MultiValuesRepr::$variant(v) if !v.is_empty() => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(&v[0]))
                },
                MultiValuesRepr::$variant(_) => {
                    Err(ValueError::NoValue($crate::ValueAbsence::EmptyCollection {
                        data_type: $data_type,
                    }))
                }
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::NoValue($crate::ValueAbsence::UnsetCollection {
                        data_type: *dt,
                    }))
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}
