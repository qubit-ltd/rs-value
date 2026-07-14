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

use serde::{Deserialize, Serialize};

use qubit_datatype::DataType;
#[cfg(feature = "converter")]
use qubit_datatype::{DataConversionError, DataConversionOptions, DataConvertTo, DataConverter};

use crate::value_error::ValueResult;
use crate::{IntoValueDefault, ValueError};

/// Single value container
///
/// Uses an enum to represent different types of values, providing
/// type-safe value storage and access.
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
/// // Create an integer value
/// let value = Value::Int32(42);
/// assert_eq!(value.get_int32().unwrap(), 42);
///
/// // Strict generic access
/// let number: i32 = value.get().unwrap();
/// assert_eq!(number, 42);
///
/// // String value
/// let text = Value::String("hello".to_string());
/// assert_eq!(text.get_string().unwrap(), "hello");
/// ```
macro_rules! define_value_enum {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                [$($value_attr:meta),*],
                [$($multi_attr:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $ownership:ident,
                $json_class:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum Value {
            /// Unset value with a declared data type.
            Empty(DataType),
            $(
                $(#[$cfg])*
                $(#[$value_attr])*
                #[doc = $value_doc]
                $variant($type),
            )+
        }
    };
}

for_each_value_type!(define_value_enum);

macro_rules! value_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], [$($value_attr:meta),*], [$($multi_attr:meta),*], $variant:ident, $type:ty, $data_type:expr, $ownership:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            Value::Empty(data_type) => *data_type,
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
    /// - `isize`, `usize`
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
    /// - `isize`, `usize`
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
    #[inline]
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

    /// Generic conversion method
    ///
    /// Converts the current value to the target type according to the shared
    /// value conversion rules.
    ///
    /// # Supported Target Types And Source Variants
    ///
    /// `Value::to<T>()` currently supports the following target types:
    ///
    /// - `bool`
    ///   - `Value::Bool`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::String`, parsed as `1`, `0`, or ASCII case-insensitive
    ///     `true` / `false`
    /// - `char`
    ///   - `Value::Char`
    /// - `i8`
    ///   - `Value::Int8`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - all integer variants
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i8`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `i16`
    ///   - `Value::Int16`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - all integer variants
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i16`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `i32`
    ///   - `Value::Int32`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int64`, `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i32`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `i64`
    ///   - `Value::Int64`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i64`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `i128`
    ///   - `Value::Int128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - all integer variants
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i128`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `u8`
    ///   - `Value::UInt8`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::UInt16`, `Value::UInt32`, `Value::UInt64`, `Value::UInt128`
    ///   - `Value::String`, parsed as `u8`
    /// - `u16`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u16`
    /// - `u32`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u32`
    /// - `u64`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u64`
    /// - `u128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u128`
    /// - `f32`
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::String`, parsed as `f32`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `f64`
    ///   - `Value::Float64`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::Float32`
    ///   - `Value::String`, parsed as `f64`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `String`
    ///   - `Value::String`
    ///   - `Value::Bool`, `Value::Char`
    ///   - all integer and floating-point variants
    ///   - `Value::Date`, `Value::Time`, `Value::DateTime`, `Value::Instant`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    ///   - `Value::IntSize`, `Value::UIntSize`
    ///   - `Value::Duration`, formatted with the configured duration unit. The
    ///     default conversion options use milliseconds and append the unit
    ///     suffix, for example `1500ms`.
    ///   - `Value::Url`
    ///   - `Value::StringMap`, serialized as JSON text
    ///   - `Value::Json`, serialized as JSON text
    /// - `NaiveDate`
    ///   - `Value::Date`
    /// - `NaiveTime`
    ///   - `Value::Time`
    /// - `NaiveDateTime`
    ///   - `Value::DateTime`
    /// - `DateTime<Utc>`
    ///   - `Value::Instant`
    /// - `BigInt`
    ///   - `Value::BigInteger`
    /// - `BigDecimal`
    ///   - `Value::BigDecimal`
    /// - `isize`
    ///   - `Value::IntSize`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - all integer variants
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `isize`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `usize`
    ///   - `Value::UIntSize`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - all integer variants
    ///   - `Value::String`, parsed as `usize`
    /// - `Duration`
    ///   - `Value::Duration`
    ///   - integer variants and `Value::BigInteger`, interpreted in the
    ///     configured duration unit
    ///   - `Value::String`, parsed as duration text. Explicit suffixes `ns`,
    ///     `us`, `ms`, `s`, `m`, `h`, and `d` are supported; text without a
    ///     suffix uses the configured duration unit.
    /// - `Url`
    ///   - `Value::Url`
    ///   - `Value::String`, parsed as URL text
    /// - `HashMap<String, String>`
    ///   - `Value::StringMap`
    /// - `serde_json::Value`
    ///   - `Value::Json`
    ///   - `Value::String`, parsed as JSON text
    ///   - `Value::StringMap`, converted to a JSON object
    ///
    /// Any target type not listed above is not supported by `Value::to<T>()`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert to
    ///
    /// # Returns
    ///
    /// Returns the converted value on success, or an error if conversion is not
    /// supported or fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    ///
    /// let num: i64 = value.to().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// let text: String = value.to().unwrap();
    /// assert_eq!(text, "42");
    /// ```
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
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
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self.to() {
            Err(ValueError::DataConversion(DataConversionError::Missing { .. })) => {
                Ok(default.into_value_default())
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
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
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
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self.to_with(options) {
            Err(ValueError::DataConversion(DataConversionError::Missing { .. })) => {
                Ok(default.into_value_default())
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
    /// - `isize`, `usize`
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
    /// let mut value = Value::Empty(DataType::Int32);
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
    /// let mut text = Value::Empty(DataType::String);
    /// text.set("hello".to_string());
    /// assert_eq!(text.get_string().unwrap(), "hello");
    /// ```
    #[inline]
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
    /// let empty = Value::Empty(DataType::String);
    /// assert_eq!(empty.data_type(), DataType::String);
    /// ```
    #[inline]
    pub fn data_type(&self) -> DataType {
        for_each_value_type!(value_data_type_match, self)
    }

    /// Tests whether this container has no concrete value.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`Value::Empty`]. An empty string, map, or JSON
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
    /// let empty = Value::Empty(DataType::String);
    /// assert!(empty.is_unset());
    /// ```
    #[inline]
    pub fn is_unset(&self) -> bool {
        matches!(self, Value::Empty(_))
    }

    /// Tests whether a concrete value belongs to the numeric type family.
    ///
    /// An unset value returns `false`, even when its declared type is numeric.
    #[inline]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete value while preserving its declared data type.
    #[inline]
    pub fn unset(&mut self) {
        *self = Value::Empty(self.data_type());
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
    #[inline]
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
            *self = Value::Empty(data_type);
        }
    }
}

impl Default for Value {
    #[inline]
    fn default() -> Self {
        Value::Empty(DataType::String)
    }
}
