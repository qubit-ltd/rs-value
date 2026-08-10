// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Single Value Container
//!
//! Provides type-safe storage and access functionality for single values.
// qubit-style: allow multiple-public-types

use std::fmt;

#[cfg(feature = "converter")]
use qubit_datatype::DataConversionOptions;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataType;

use super::value_ref::ValueRef;
use crate::IntoValueDefault;
use crate::ValueError;
use crate::value_error::ValueResult;

/// Defines the private storage representation for the public single-value
/// container from the shared value-type table.
macro_rules! define_value_enum {
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
        /// Internal single-value representation.
        ///
        /// Uses an enum to represent different types of values, providing
        /// type-safe value storage and access.
        ///
        /// This representation is private; downstream code uses [`Value`]
        /// constructors and [`ValueRef`] semantic views instead of matching
        /// storage details.
        ///
        /// # Behavior
        ///
        /// - Stores one value from the closed [`DataType`] family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from concrete inner values.
        /// - The URL variant uses boxed storage internally to keep the enum
        ///   compact; use [`Value::new`] and typed getters instead of relying
        ///   on the storage representation of individual variants.
        ///
        /// # Equality and hashing
        ///
        /// Equality preserves enum-variant identity. Signed zero is canonicalized,
        /// every NaN payload within one float width is equal, and unordered payloads
        /// hash structurally. Standard hash output is suitable for in-memory keys but
        /// is not a stable persistent fingerprint.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::Int32(42);
        /// assert_eq!(value.get_int32().unwrap(), 42);
        ///
        /// let number: i32 = value.get().unwrap();
        /// assert_eq!(number, 42);
        ///
        /// let text = Value::String("hello".to_string());
        /// assert_eq!(text.get_string().unwrap(), "hello");
        /// ```
        #[derive(Debug, Clone)]
        pub(crate) enum ValueRepr {
            /// Unset value with a declared data type.
            Unset(
                /// Declared data type retained while the value is unset.
                DataType,
            ),
            $(
                $(#[$cfg])*
                #[doc = $value_doc]
                $variant(
                    #[doc = concat!("Stored ", $value_doc, " payload.")]
                    value_storage_type!($variant, $type),
                ),
            )+
        }
    };
}

for_each_value_type!(define_value_enum);

/// Single typed runtime value with private storage representation.
///
/// Construction and access are expressed through methods and conversions. The
/// concrete enum representation is private so storage optimizations do not
/// become part of the public API.
#[must_use]
#[derive(Clone)]
pub struct Value {
    pub(crate) repr: ValueRepr,
}

impl fmt::Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.view().fmt(formatter)
    }
}

macro_rules! impl_value_constructors {
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
        impl Value {
            /// Creates an unset value with an explicit declared type.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub const fn Unset(data_type: DataType) -> Self {
                Self::new_unset(data_type)
            }

            /// Creates an unset value with an explicit declared type.
            #[inline(always)]
            pub const fn new_unset(data_type: DataType) -> Self {
                Self { repr: ValueRepr::Unset(data_type) }
            }

            $(
                $(#[$cfg])*
                #[allow(non_snake_case)]
                #[doc = concat!("Creates a ", $value_doc, ".")]
                #[inline(always)]
                pub fn $variant(value: $type) -> Self {
                    Self { repr: ValueRepr::$variant(value_storage_new!($variant, value)) }
                }
            )+
        }
    };
}

for_each_value_type!(impl_value_constructors);

impl Value {
    /// Borrows the stable semantic view of this value.
    #[inline(always)]
    pub fn view(&self) -> ValueRef<'_> {
        match &self.repr {
            ValueRepr::Unset(data_type) => ValueRef::Unset(*data_type),
            ValueRepr::Bool(value) => ValueRef::Bool(*value),
            ValueRepr::Char(value) => ValueRef::Char(*value),
            ValueRepr::Int8(value) => ValueRef::Int8(*value),
            ValueRepr::Int16(value) => ValueRef::Int16(*value),
            ValueRepr::Int32(value) => ValueRef::Int32(*value),
            ValueRepr::Int64(value) => ValueRef::Int64(*value),
            ValueRepr::Int128(value) => ValueRef::Int128(*value),
            ValueRepr::UInt8(value) => ValueRef::UInt8(*value),
            ValueRepr::UInt16(value) => ValueRef::UInt16(*value),
            ValueRepr::UInt32(value) => ValueRef::UInt32(*value),
            ValueRepr::UInt64(value) => ValueRef::UInt64(*value),
            ValueRepr::UInt128(value) => ValueRef::UInt128(*value),
            ValueRepr::Float32(value) => ValueRef::Float32(*value),
            ValueRepr::Float64(value) => ValueRef::Float64(*value),
            #[cfg(feature = "big-integer")]
            ValueRepr::BigInteger(value) => ValueRef::BigInteger(value),
            #[cfg(feature = "big-decimal")]
            ValueRepr::BigDecimal(value) => ValueRef::BigDecimal(value),
            ValueRepr::String(value) => ValueRef::String(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Date(value) => ValueRef::Date(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Time(value) => ValueRef::Time(value),
            #[cfg(feature = "chrono")]
            ValueRepr::DateTime(value) => ValueRef::DateTime(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Instant(value) => ValueRef::Instant(value),
            ValueRepr::Duration(value) => ValueRef::Duration(value),
            #[cfg(feature = "url")]
            ValueRepr::Url(value) => ValueRef::Url(value.as_ref()),
            ValueRepr::StringMap(value) => ValueRef::StringMap(value),
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => ValueRef::Json(value),
        }
    }
}

macro_rules! value_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            ValueRepr::Unset(data_type) => *data_type,
            $($(#[$cfg])* ValueRepr::$variant(_) => $data_type,)+
        }
    };
}

// ============================================================================
// Getter method generation macro
// ============================================================================

/// Unified getter generation macro
///
/// Supports two modes:
/// 1. `copy:` - For types implementing the Copy trait, directly returns the
///    value
/// 2. `ref:` - For non-Copy types, returns a reference
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so
/// you can add `///` comments before macro invocations.
impl Value {
    /// Generic constructor method
    ///
    /// Creates a `Value` from any supported type, avoiding direct use of
    /// enum variants.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::new<T>(value)` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`, `&str`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the value to wrap
    ///
    /// # Parameters
    ///
    /// * `value` - Value to wrap.
    ///
    /// # Returns
    ///
    /// Returns a `Value` wrapping the given value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// // Basic types
    /// let v = Value::new(42i32);
    /// assert_eq!(v.get_int32().unwrap(), 42);
    ///
    /// let v = Value::new(true);
    /// assert_eq!(v.get_bool().unwrap(), true);
    ///
    /// // String
    /// let v = Value::new("hello".to_string());
    /// assert_eq!(v.get_string().unwrap(), "hello");
    /// ```
    #[inline(always)]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    /// Generic getter method.
    ///
    /// Performs a strict typed read of the stored value as `T`.
    ///
    /// `get<T>()` performs strict type matching. It does not do cross-type
    /// conversion.
    ///
    /// For example, `Value::Int32(42).get::<i64>()` fails, while
    /// `Value::Int32(42).to::<i64>()` succeeds.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::get<T>()` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to retrieve
    ///
    /// # Returns
    ///
    /// Returns the stored value when its type matches `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with the
    /// requested type, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    ///
    /// // Through type inference
    /// let num: i32 = value.get().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// // Explicitly specify type parameter
    /// let num = value.get::<i32>().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// // Different type
    /// let text = Value::String("hello".to_string());
    /// let s: String = text.get().unwrap();
    /// assert_eq!(s, "hello");
    ///
    /// // Boolean value
    /// let flag = Value::Bool(true);
    /// let b: bool = flag.get().unwrap();
    /// assert_eq!(b, true);
    /// ```
    #[inline(always)]
    pub fn get<T>(&self) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        T::try_from(self)
    }

    /// Generic getter method with a default value.
    ///
    /// Returns the supplied default only when this value is unset. Type
    /// mismatches and conversion errors are still returned as errors.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict read and default value.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used only when `self` is unset.
    ///
    /// # Returns
    ///
    /// The stored value, or `default` when the value is unset.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`.
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Strictly reads this value or calls `default` only when it is unset.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict read and fallback value.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only when this value is unset.
    ///
    /// # Returns
    ///
    /// The stored value, or the callback result for an unset value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`; the callback is not invoked in that case.
    #[inline]
    pub fn get_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> T,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => Ok(default()),
            result => result,
        }
    }

    /// Converts the stored value to another supported data type.
    ///
    /// This method delegates to the authoritative conversion contract in
    /// [`qubit-datatype`](https://docs.rs/qubit-datatype/latest/qubit_datatype/).
    /// The enabled rich-type features determine which source and target
    /// families are available. An unset value is reported as a structured
    /// missing-value conversion error.
    ///
    /// Unlike [`Self::get`], this method permits conversions supported by
    /// [`qubit_datatype::DataConverter`] and applies
    /// [`qubit_datatype::DataConversionOptions`].
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error when the value is unset, the
    /// conversion is unsupported, or the source is invalid for `T`.
    ///
    /// # Returns
    ///
    /// The converted value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.to::<i64>().unwrap(), 42);
    /// assert_eq!(value.to::<String>().unwrap(), "42");
    /// ```
    #[inline(always)]
    #[cfg(feature = "converter")]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        self.to_with(DataConversionOptions::default_ref())
    }

    /// Converts this value to `T`, or returns `default` when storage is unset
    /// or conversion reports a missing value.
    ///
    /// Conversion failures from concrete values are preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset or conversion-
    ///   missing storage.
    ///
    /// # Returns
    ///
    /// The converted value, or `default` for an unset or conversion-missing
    /// value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error for concrete values that cannot be
    /// converted to `T`.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts this value to `T`, or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only when conversion reports a missing
    ///   value.
    ///
    /// # Returns
    ///
    /// The converted value, or the callback result for an unset or
    /// conversion-missing value.
    ///
    /// # Errors
    ///
    /// Preserves conversion errors from concrete values without invoking the
    /// callback.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }

    /// Converts this value to `T` using the provided conversion options.
    ///
    /// This method uses the shared [`qubit_datatype`] conversion layer
    /// directly, so options such as string trimming, blank string handling,
    /// and boolean aliases are applied consistently with other value
    /// containers.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert to.
    ///
    /// # Parameters
    ///
    /// * `options` - Conversion options forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// Returns the converted value on success.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::ValueError`] when the value is missing, unsupported,
    /// or invalid for `T` under the provided options.
    #[inline(always)]
    #[cfg(feature = "converter")]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        super::value_converters::convert_with_data_converter_with(self, options)
    }

    /// Converts this value to `T` using conversion options, or returns
    /// `default` when storage is unset or conversion reports a missing value.
    ///
    /// Conversion failures from concrete values are preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset or conversion-
    ///   missing storage.
    /// * `options` - Conversion options forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted value, or `default` for an unset or conversion-missing
    /// value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error for concrete values that cannot be
    /// converted under `options`.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_with(options) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts this value with `options`, or calls `default` when storage is
    /// unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only for a missing source value.
    /// * `options` - Conversion options forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted value, or the callback result for an unset or
    /// conversion-missing value.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_else_with<T, F>(
        &self,
        default: F,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_with(options) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }

    /// Generic setter method
    ///
    /// Replaces the current value with any supported input value.
    ///
    /// This operation updates the stored type to `T` when needed. It does not
    /// perform runtime type-mismatch validation against the previous variant.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::set<T>(value)` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`, `&str`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - Input type convertible into [`Value`].
    ///
    /// # Parameters
    ///
    /// * `value` - The value to set
    ///
    /// # Compile-time restriction
    ///
    /// Unsupported input types fail to compile because they do not implement
    /// `Into<Value>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let mut value = Value::Unset(DataType::Int32);
    ///
    /// // Through type inference
    /// value.set(42i32);
    /// assert_eq!(value.get_int32().unwrap(), 42);
    ///
    /// // Explicitly specify type parameter
    /// value.set::<i32>(100);
    /// assert_eq!(value.get_int32().unwrap(), 100);
    ///
    /// // String type
    /// let mut text = Value::Unset(DataType::String);
    /// text.set("hello".to_string());
    /// assert_eq!(text.get_string().unwrap(), "hello");
    /// ```
    #[inline(always)]
    pub fn set<T>(&mut self, value: T)
    where
        T: Into<Self>,
    {
        *self = value.into();
    }

    /// Get the data type of the value
    ///
    /// # Returns
    ///
    /// Returns the data type corresponding to this value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.data_type(), DataType::Int32);
    ///
    /// let empty = Value::Unset(DataType::String);
    /// assert_eq!(empty.data_type(), DataType::String);
    /// ```
    ///
    /// ```compile_fail
    /// #![deny(unused_must_use)]
    /// use qubit_value::Value;
    ///
    /// Value::new(42_i32).data_type();
    /// ```
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        for_each_value_type!(value_data_type_match, self)
    }

    /// Tests whether this container has no concrete value.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`Value::Unset`]. An empty string, map, or JSON
    /// container is still a concrete value and returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert!(!value.is_unset());
    ///
    /// let empty = Value::Unset(DataType::String);
    /// assert!(empty.is_unset());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_unset(&self) -> bool {
        matches!(self.repr, ValueRepr::Unset(_))
    }

    /// Tests whether a concrete value belongs to the numeric type family.
    ///
    /// An unset value returns `false`, even when its declared type is numeric.
    ///
    /// # Returns
    ///
    /// `true` for concrete numeric variants; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete value while preserving its declared data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        *self = Value::new_unset(self.data_type());
    }

    /// Set the data type
    ///
    /// If the new type differs from the current type, clears the value
    /// and sets the new type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - The data type to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let mut value = Value::Int32(42);
    /// value.set_type(DataType::String);
    /// assert!(value.is_unset());
    /// assert_eq!(value.data_type(), DataType::String);
    /// ```
    #[inline]
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = Value::new_unset(data_type);
        }
    }
}
