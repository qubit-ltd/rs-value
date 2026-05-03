/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Single Value Container
//!
//! Provides type-safe storage and access functionality for single values.
//!

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use num_bigint::BigInt;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use qubit_datatype::{
    DataConversionOptions,
    DataConvertTo,
    DataConverter,
    DataType,
};

use crate::value_error::ValueResult;
use crate::{
    IntoValueDefault,
    ValueError,
};

/// Single value container
///
/// Uses an enum to represent different types of values, providing
/// type-safe value storage and access.
///
/// # Features
///
/// - Zero-cost abstraction with compile-time type checking
/// - Supports multiple basic data types
/// - Provides two sets of APIs for type checking and type conversion
/// - Automatic memory management
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
/// // Type conversion
/// let converted = value.to::<i64>().unwrap();
/// assert_eq!(converted, 42i64);
///
/// // String value
/// let text = Value::String("hello".to_string());
/// assert_eq!(text.get_string().unwrap(), "hello");
/// ```
///
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Empty value (has type but no value)
    Empty(DataType),
    /// Boolean value
    Bool(bool),
    /// Character value
    Char(char),
    /// 8-bit signed integer
    Int8(i8),
    /// 16-bit signed integer
    Int16(i16),
    /// 32-bit signed integer
    Int32(i32),
    /// 64-bit signed integer
    Int64(i64),
    /// 128-bit signed integer
    Int128(i128),
    /// 8-bit unsigned integer
    UInt8(u8),
    /// 16-bit unsigned integer
    UInt16(u16),
    /// 32-bit unsigned integer
    UInt32(u32),
    /// 64-bit unsigned integer
    UInt64(u64),
    /// 128-bit unsigned integer
    UInt128(u128),
    /// Platform-dependent signed integer (isize)
    IntSize(isize),
    /// Platform-dependent unsigned integer (usize)
    UIntSize(usize),
    /// 32-bit floating point number
    Float32(f32),
    /// 64-bit floating point number
    Float64(f64),
    /// Big integer type
    BigInteger(BigInt),
    /// Big decimal type
    BigDecimal(BigDecimal),
    /// String
    String(String),
    /// Date
    Date(NaiveDate),
    /// Time
    Time(NaiveTime),
    /// Date and time
    DateTime(NaiveDateTime),
    /// UTC instant
    Instant(DateTime<Utc>),
    /// Duration type (std::time::Duration)
    Duration(Duration),
    /// URL type (url::Url)
    Url(Url),
    /// String map type (HashMap<String, String>)
    StringMap(HashMap<String, String>),
    /// JSON value type (serde_json::Value)
    Json(serde_json::Value),
}

use super::value_constructor::ValueConstructor;
use super::value_converter::ValueConverter;
use super::value_getter::ValueGetter;
use super::value_setter::ValueSetter;

// ============================================================================
// Getter method generation macro
// ============================================================================

/// Unified getter generation macro
///
/// Supports two modes:
/// 1. `copy:` - For types implementing the Copy trait, directly returns the value
/// 2. `ref:` - For non-Copy types, returns a reference
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so
/// you can add `///` comments before macro invocations.
///
///
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
        Self: ValueConstructor<T>,
    {
        <Self as ValueConstructor<T>>::from_type(value)
    }

    /// Generic getter method
    ///
    /// Automatically selects the correct getter method based on the target
    /// type, performing strict type checking.
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
    /// If types match, returns the value of the corresponding type;
    /// otherwise returns an error
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
        Self: ValueGetter<T>,
    {
        <Self as ValueGetter<T>>::get_value(self)
    }

    /// Generic getter method with a default value.
    ///
    /// Returns the supplied default only when this value is empty. Type
    /// mismatches and conversion errors are still returned as errors.
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        Self: ValueGetter<T>,
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
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
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
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
    ///   - `Value::Float32`, `Value::Float64`
    ///   - `Value::String`, parsed as `i32`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `i64`
    ///   - `Value::Int64`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
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
    ///   - `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
    ///     `Value::UInt128`
    ///   - `Value::String`, parsed as `u8`
    /// - `u16`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u16`
    /// - `u32`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u32`
    /// - `u64`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::String`, parsed as `u64`
    /// - `u128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
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
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
    ///   - `Value::String`, parsed as `f32`
    ///   - `Value::BigInteger`, `Value::BigDecimal`
    /// - `f64`
    ///   - `Value::Float64`
    ///   - `Value::Bool`
    ///   - `Value::Char`
    ///   - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
    ///     `Value::Int128`
    ///   - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`,
    ///     `Value::UInt64`, `Value::UInt128`
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
    ///   - `Value::Duration`, formatted as `<nanoseconds>ns`
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
    ///   - `Value::String`, parsed from `<nanoseconds>ns`
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
    pub fn to<T>(&self) -> ValueResult<T>
    where
        Self: ValueConverter<T>,
    {
        <Self as ValueConverter<T>>::convert(self)
    }

    /// Converts this value to `T`, or returns `default` when it is empty.
    ///
    /// Conversion failures from non-empty values are preserved.
    #[inline]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        Self: ValueConverter<T>,
    {
        match self.to() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts this value to `T` using the provided conversion options.
    ///
    /// This method uses the shared [`qubit_datatype`] conversion layer directly,
    /// so options such as string trimming, blank string handling, and boolean
    /// aliases are applied consistently with other value containers.
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
    /// Returns a [`crate::ValueError`] when the value is missing, unsupported, or
    /// invalid for `T` under the provided options.
    #[inline]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        super::value_converters::convert_with_data_converter_with(self, options)
    }

    /// Converts this value to `T` using conversion options, or returns
    /// `default` when it is empty.
    ///
    /// Conversion failures from non-empty values are preserved.
    #[inline]
    pub fn to_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self.to_with(options) {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Generic setter method
    ///
    /// Automatically selects the correct setter method based on the target
    /// type and replaces the current value.
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
    /// * `T` - The target type to set
    ///
    /// # Parameters
    ///
    /// * `value` - The value to set
    ///
    /// # Returns
    ///
    /// If setting succeeds, returns `Ok(())`; otherwise returns an error
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
    /// value.set(42i32).unwrap();
    /// assert_eq!(value.get_int32().unwrap(), 42);
    ///
    /// // Explicitly specify type parameter
    /// value.set::<i32>(100).unwrap();
    /// assert_eq!(value.get_int32().unwrap(), 100);
    ///
    /// // String type
    /// let mut text = Value::Empty(DataType::String);
    /// text.set("hello".to_string()).unwrap();
    /// assert_eq!(text.get_string().unwrap(), "hello");
    /// ```
    #[inline]
    pub fn set<T>(&mut self, value: T) -> ValueResult<()>
    where
        Self: ValueSetter<T>,
    {
        <Self as ValueSetter<T>>::set_value(self, value)
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
        match self {
            Value::Empty(dt) => *dt,
            Value::Bool(_) => DataType::Bool,
            Value::Char(_) => DataType::Char,
            Value::Int8(_) => DataType::Int8,
            Value::Int16(_) => DataType::Int16,
            Value::Int32(_) => DataType::Int32,
            Value::Int64(_) => DataType::Int64,
            Value::Int128(_) => DataType::Int128,
            Value::UInt8(_) => DataType::UInt8,
            Value::UInt16(_) => DataType::UInt16,
            Value::UInt32(_) => DataType::UInt32,
            Value::UInt64(_) => DataType::UInt64,
            Value::UInt128(_) => DataType::UInt128,
            Value::Float32(_) => DataType::Float32,
            Value::Float64(_) => DataType::Float64,
            Value::String(_) => DataType::String,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Instant(_) => DataType::Instant,
            Value::BigInteger(_) => DataType::BigInteger,
            Value::BigDecimal(_) => DataType::BigDecimal,
            Value::IntSize(_) => DataType::IntSize,
            Value::UIntSize(_) => DataType::UIntSize,
            Value::Duration(_) => DataType::Duration,
            Value::Url(_) => DataType::Url,
            Value::StringMap(_) => DataType::StringMap,
            Value::Json(_) => DataType::Json,
        }
    }

    /// Check if the value is empty
    ///
    /// # Returns
    ///
    /// Returns `true` if the value is empty
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert!(!value.is_empty());
    ///
    /// let empty = Value::Empty(DataType::String);
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty(_))
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
    /// assert!(value.is_empty());
    /// assert_eq!(value.data_type(), DataType::Int32);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        let dt = self.data_type();
        *self = Value::Empty(dt);
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
    /// assert!(value.is_empty());
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
