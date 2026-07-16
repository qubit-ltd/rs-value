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

use qubit_datatype::DataType;
#[cfg(feature = "converter")]
use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConversionTarget,
};

use crate::value_error::ValueResult;
use crate::{
    IntoValueDefault,
    ValueError,
};

/// Defines the public single-value container from the shared value-type table.
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
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        /// Single value container.
        ///
        /// Uses an enum to represent different types of values, providing
        /// type-safe value storage and access.
        ///
        /// This enum is non-exhaustive; downstream matches must include a
        /// wildcard arm so future value variants remain source-compatible.
        ///
        /// # Behavior
        ///
        /// - Stores one value from the closed [`DataType`] family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from concrete inner values.
        ///
        /// # Example
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
        #[must_use]
        #[non_exhaustive]
        #[derive(Debug, Clone)]
        pub enum Value {
            /// Unset value with a declared data type.
            Unset(DataType),
            $(
                $(#[$cfg])*
                #[doc = $value_doc]
                $variant($type),
            )+
        }
    };
}

for_each_value_type!(define_value_enum);

macro_rules! value_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            Value::Unset(data_type) => *data_type,
            $($(#[$cfg])* Value::$variant(_) => $data_type,)+
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
    /// Creates an unset scalar with an explicit declared type.
    ///
    /// # Arguments
    ///
    /// * `data_type` - Declared type retained while no concrete value exists.
    ///
    /// # Returns
    ///
    /// An unset scalar carrying `data_type`.
    #[inline(always)]
    pub const fn new_unset(data_type: DataType) -> Self {
        Self::Unset(data_type)
    }

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
    /// # Returns
    ///
    /// Returns a `Value` wrapping the given value
    ///
    /// # Example
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
    #[inline]
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
    /// Returns [`ValueError::NoValue`] when the value is unset with the
    /// requested type, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Example
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
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
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
    /// # Example
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

    /// Converts this value to `T`, or returns `default` when it is unset.
    ///
    /// Conversion failures from concrete values are preserved.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to() {
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
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
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        super::value_converters::convert_with_data_converter_with(self, options)
    }

    /// Converts this value to `T` using conversion options, or returns
    /// `default` when it is unset.
    ///
    /// Conversion failures from concrete values are preserved.
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
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
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
    /// # Example
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
    /// # Example
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
    #[inline(always)]
    #[must_use]
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
    /// # Example
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
        matches!(self, Value::Unset(_))
    }

    /// Tests whether a concrete value belongs to the numeric type family.
    ///
    /// An unset value returns `false`, even when its declared type is numeric.
    #[inline]
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete value while preserving its declared data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        *self = Value::Unset(self.data_type());
    }

    /// Clear the value while preserving the type
    ///
    /// Sets the current value to empty but retains its data type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let mut value = Value::Int32(42);
    /// value.clear();
    /// assert!(value.is_unset());
    /// assert_eq!(value.data_type(), DataType::Int32);
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        self.unset();
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
    /// # Example
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
            *self = Value::Unset(data_type);
        }
    }
}
