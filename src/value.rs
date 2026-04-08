/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Single Value Container
//!
//! Provides type-safe storage and access functionality for single values.
//!
//! # Author
//!
//! Haixing Hu

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use qubit_common::lang::DataType;

use super::error::{ValueError, ValueResult};

/// Single value container
///
/// Uses an enum to represent different types of values, providing type-safe value storage and access.
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
/// ```rust,ignore
/// use common_rs::util::value::Value;
///
/// // Create an integer value
/// let value = Value::Int32(42);
/// assert_eq!(value.get_int32().unwrap(), 42);
///
/// // Type conversion
/// let converted = value.as_int64().unwrap();
/// assert_eq!(converted, 42i64);
///
/// // String value
/// let text = Value::String("hello".to_string());
/// assert_eq!(text.get_string().unwrap(), "hello");
/// ```
///
/// # Author
///
/// Haixing Hu
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
    /// 32-bit floating point number
    Float32(f32),
    /// 64-bit floating point number
    Float64(f64),
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
    /// Big integer type
    BigInteger(BigInt),
    /// Big decimal type
    BigDecimal(BigDecimal),
    /// Platform-dependent signed integer (isize)
    IntSize(isize),
    /// Platform-dependent unsigned integer (usize)
    UIntSize(usize),
    /// Duration type (std::time::Duration)
    Duration(Duration),
    /// URL type (url::Url)
    Url(Url),
    /// String map type (HashMap<String, String>)
    StringMap(HashMap<String, String>),
    /// JSON value type (serde_json::Value)
    Json(serde_json::Value),
}

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
/// The macro automatically extracts preceding documentation comments, so you can add `///` comments before macro invocations.
///
/// # Author
///
/// Haixing Hu
///
macro_rules! impl_get_value {
    // Copy type: directly dereference and return
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&self) -> ValueResult<$type> {
            match self {
                Value::$variant(v) => Ok(*v),
                Value::Empty(_) => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Reference type: use conversion function to return reference, fixing lifetime issues
    ($(#[$attr:meta])* ref: $method:ident, $variant:ident, $ret_type:ty, $data_type:expr, $conversion:expr) => {
        $(#[$attr])*
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match self {
                Value::$variant(v) => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(v))
                },
                Value::Empty(_) => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified setter generation macro
///
/// Supports two modes:
/// 1. `copy:` - For types implementing the Copy trait, directly sets the value
/// 2. `owned:` - For non-Copy types, requires owning the value
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you can add `///` comments before macro invocations.
///
/// # Author
///
/// Haixing Hu
///
macro_rules! impl_set_value {
    // Copy type: directly set the value
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            *self = Value::$variant(value);
            Ok(())
        }
    };

    // Owned type: set the owned value
    ($(#[$attr:meta])* owned: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            *self = Value::$variant(value);
            Ok(())
        }
    };
}

impl Value {
    /// Generic constructor method
    ///
    /// Creates a `Value` from any supported type, avoiding direct use of enum variants.
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
    /// ```rust,ignore
    /// use crate::util::value::Value;
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
    pub fn new<T>(value: T) -> Self
    where
        Self: ValueConstructor<T>,
    {
        <Self as ValueConstructor<T>>::from_type(value)
    }

    /// Generic getter method
    ///
    /// Automatically selects the correct getter method based on the target type, performing strict type checking.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to retrieve
    ///
    /// # Returns
    ///
    /// If types match, returns the value of the corresponding type; otherwise returns an error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
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
    pub fn get<T>(&self) -> ValueResult<T>
    where
        Self: ValueGetter<T>,
    {
        <Self as ValueGetter<T>>::get_value(self)
    }

    /// Generic setter method
    ///
    /// Automatically selects the correct setter method based on the target type, performing strict type checking.
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
    /// ```rust,ignore
    /// use crate::util::value::Value;
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
    /// ```rust,ignore
    /// use crate::util::value::{Value, DataType};
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.data_type(), DataType::Int32);
    ///
    /// let empty = Value::Empty(DataType::String);
    /// assert_eq!(empty.data_type(), DataType::String);
    /// ```
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
    /// ```rust,ignore
    /// use crate::util::value::{Value, DataType};
    ///
    /// let value = Value::Int32(42);
    /// assert!(!value.is_empty());
    ///
    /// let empty = Value::Empty(DataType::String);
    /// assert!(empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty(_))
    }

    /// Clear the value while preserving the type
    ///
    /// Sets the current value to empty but retains its data type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{Value, DataType};
    ///
    /// let mut value = Value::Int32(42);
    /// value.clear();
    /// assert!(value.is_empty());
    /// assert_eq!(value.data_type(), DataType::Int32);
    /// ```
    pub fn clear(&mut self) {
        let dt = self.data_type();
        *self = Value::Empty(dt);
    }

    /// Set the data type
    ///
    /// If the new type differs from the current type, clears the value and sets the new type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - The data type to set
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{Value, DataType};
    ///
    /// let mut value = Value::Int32(42);
    /// value.set_type(DataType::String);
    /// assert!(value.is_empty());
    /// assert_eq!(value.data_type(), DataType::String);
    /// ```
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = Value::Empty(data_type);
        }
    }

    // ========================================================================
    // Type-checking getters (strict type matching)
    // ========================================================================

    impl_get_value! {
        /// Get boolean value
        ///
        /// # Returns
        ///
        /// If types match, returns the boolean value; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        ///
        /// let value = Value::Bool(true);
        /// assert_eq!(value.get_bool().unwrap(), true);
        /// ```
        copy: get_bool, Bool, bool, DataType::Bool
    }

    impl_get_value! {
        /// Get character value
        ///
        /// # Returns
        ///
        /// If types match, returns the character value; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        ///
        /// let value = Value::Char('A');
        /// assert_eq!(value.get_char().unwrap(), 'A');
        /// ```
        copy: get_char, Char, char, DataType::Char
    }

    impl_get_value! {
        /// Get int8 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int8 value; otherwise returns an error
        copy: get_int8, Int8, i8, DataType::Int8
    }

    impl_get_value! {
        /// Get int16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int16 value; otherwise returns an error
        copy: get_int16, Int16, i16, DataType::Int16
    }

    impl_get_value! {
        /// Get int32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int32 value; otherwise returns an error
        copy: get_int32, Int32, i32, DataType::Int32
    }

    impl_get_value! {
        /// Get int64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int64 value; otherwise returns an error
        copy: get_int64, Int64, i64, DataType::Int64
    }

    impl_get_value! {
        /// Get int128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int128 value; otherwise returns an error
        copy: get_int128, Int128, i128, DataType::Int128
    }

    impl_get_value! {
        /// Get uint8 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint8 value; otherwise returns an error
        copy: get_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_value! {
        /// Get uint16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint16 value; otherwise returns an error
        copy: get_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_value! {
        /// Get uint32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint32 value; otherwise returns an error
        copy: get_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_value! {
        /// Get uint64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint64 value; otherwise returns an error
        copy: get_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_value! {
        /// Get uint128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint128 value; otherwise returns an error
        copy: get_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_value! {
        /// Get float32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float32 value; otherwise returns an error
        copy: get_float32, Float32, f32, DataType::Float32
    }

    impl_get_value! {
        /// Get float64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float64 value; otherwise returns an error
        copy: get_float64, Float64, f64, DataType::Float64
    }

    impl_get_value! {
        /// Get string reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        ///
        /// let value = Value::String("hello".to_string());
        /// assert_eq!(value.get_string().unwrap(), "hello");
        /// ```
        ref: get_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    impl_get_value! {
        /// Get date value
        ///
        /// # Returns
        ///
        /// If types match, returns the date value; otherwise returns an error
        copy: get_date, Date, NaiveDate, DataType::Date
    }

    impl_get_value! {
        /// Get time value
        ///
        /// # Returns
        ///
        /// If types match, returns the time value; otherwise returns an error
        copy: get_time, Time, NaiveTime, DataType::Time
    }

    impl_get_value! {
        /// Get datetime value
        ///
        /// # Returns
        ///
        /// If types match, returns the datetime value; otherwise returns an error
        copy: get_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_value! {
        /// Get UTC instant value
        ///
        /// # Returns
        ///
        /// If types match, returns the UTC instant value; otherwise returns an error
        copy: get_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_value! {
        /// Get big integer value
        ///
        /// # Returns
        ///
        /// If types match, returns the big integer value; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        /// use num_bigint::BigInt;
        ///
        /// let value = Value::BigInteger(BigInt::from(123456789));
        /// assert_eq!(value.get_biginteger().unwrap(), BigInt::from(123456789));
        /// ```
        ref: get_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    impl_get_value! {
        /// Get big decimal value
        ///
        /// # Returns
        ///
        /// If types match, returns the big decimal value; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        /// use bigdecimal::BigDecimal;
        ///
        /// let value = Value::BigDecimal(BigDecimal::from(123.456));
        /// assert_eq!(value.get_bigdecimal().unwrap(), BigDecimal::from(123.456));
        /// ```
        ref: get_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal, |v: &BigDecimal| v.clone()
    }

    // ========================================================================
    // Type conversion getters (attempt conversion)
    // ========================================================================

    /// Convert to boolean value
    ///
    /// # Returns
    ///
    /// Attempts to convert the value to a boolean, supporting conversion from various types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int32(1);
    /// assert_eq!(value.as_bool().unwrap(), true);
    ///
    /// let value = Value::String("true".to_string());
    /// assert_eq!(value.as_bool().unwrap(), true);
    /// ```
    pub fn as_bool(&self) -> ValueResult<bool> {
        match self {
            Value::Bool(v) => Ok(*v),
            Value::Int8(v) => Ok(*v != 0),
            Value::Int16(v) => Ok(*v != 0),
            Value::Int32(v) => Ok(*v != 0),
            Value::Int64(v) => Ok(*v != 0),
            Value::Int128(v) => Ok(*v != 0),
            Value::UInt8(v) => Ok(*v != 0),
            Value::UInt16(v) => Ok(*v != 0),
            Value::UInt32(v) => Ok(*v != 0),
            Value::UInt64(v) => Ok(*v != 0),
            Value::UInt128(v) => Ok(*v != 0),
            Value::String(s) => s.parse::<bool>().map_err(|_| {
                ValueError::ConversionError(format!("Cannot convert '{}' to boolean", s))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Bool,
            }),
        }
    }

    /// Convert to int32 value
    ///
    /// # Returns
    ///
    /// Attempts to convert the value to i32, supporting conversion from various types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int8(42);
    /// assert_eq!(value.as_int32().unwrap(), 42);
    ///
    /// let value = Value::String("123".to_string());
    /// assert_eq!(value.as_int32().unwrap(), 123);
    /// ```
    pub fn as_int32(&self) -> ValueResult<i32> {
        match self {
            Value::Int32(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as i32),
            Value::Int8(v) => Ok(*v as i32),
            Value::Int16(v) => Ok(*v as i32),
            Value::Int64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("i64 value out of i32 range".to_string())),
            Value::Int128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("i128 value out of i32 range".to_string())
            }),
            Value::UInt8(v) => Ok(*v as i32),
            Value::UInt16(v) => Ok(*v as i32),
            Value::UInt32(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u32 value out of i32 range".to_string())),
            Value::UInt64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u64 value out of i32 range".to_string())),
            Value::UInt128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("u128 value out of i32 range".to_string())
            }),
            Value::Float32(v) => Ok(*v as i32),
            Value::Float64(v) => Ok(*v as i32),
            Value::String(s) => s
                .parse::<i32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to i32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => v.to_i32().ok_or_else(|| {
                ValueError::ConversionError("BigInteger value out of i32 range".to_string())
            }),
            Value::BigDecimal(v) => v.to_i32().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigDecimal value cannot be converted to i32".to_string(),
                )
            }),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Int32,
            }),
        }
    }

    /// Convert to int64 value
    ///
    /// # Returns
    ///
    /// Attempts to convert the value to i64, supporting conversion from various types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.as_int64().unwrap(), 42i64);
    ///
    /// let value = Value::Float64(123.456);
    /// assert_eq!(value.as_int64().unwrap(), 123i64);
    /// ```
    pub fn as_int64(&self) -> ValueResult<i64> {
        match self {
            Value::Int64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as i64),
            Value::Int8(v) => Ok(*v as i64),
            Value::Int16(v) => Ok(*v as i64),
            Value::Int32(v) => Ok(*v as i64),
            Value::Int128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("i128 value out of i64 range".to_string())
            }),
            Value::UInt8(v) => Ok(*v as i64),
            Value::UInt16(v) => Ok(*v as i64),
            Value::UInt32(v) => Ok(*v as i64),
            Value::UInt64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u64 value out of i64 range".to_string())),
            Value::UInt128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("u128 value out of i64 range".to_string())
            }),
            Value::Float32(v) => Ok(*v as i64),
            Value::Float64(v) => Ok(*v as i64),
            Value::String(s) => s
                .parse::<i64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to i64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => v.to_i64().ok_or_else(|| {
                ValueError::ConversionError("BigInteger value out of i64 range".to_string())
            }),
            Value::BigDecimal(v) => v.to_i64().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigDecimal value cannot be converted to i64".to_string(),
                )
            }),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Int64,
            }),
        }
    }

    /// Convert to float64 value
    ///
    /// # Returns
    ///
    /// Attempts to convert the value to f64, supporting conversion from various types
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.as_float64().unwrap(), 42.0);
    ///
    /// let value = Value::Bool(true);
    /// assert_eq!(value.as_float64().unwrap(), 1.0);
    /// ```
    pub fn as_float64(&self) -> ValueResult<f64> {
        match self {
            Value::Float64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Char(v) => Ok(*v as u32 as f64),
            Value::Float32(v) => Ok(*v as f64),
            Value::Int8(v) => Ok(*v as f64),
            Value::Int16(v) => Ok(*v as f64),
            Value::Int32(v) => Ok(*v as f64),
            Value::Int64(v) => Ok(*v as f64),
            Value::Int128(v) => Ok(*v as f64),
            Value::UInt8(v) => Ok(*v as f64),
            Value::UInt16(v) => Ok(*v as f64),
            Value::UInt32(v) => Ok(*v as f64),
            Value::UInt64(v) => Ok(*v as f64),
            Value::UInt128(v) => Ok(*v as f64),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to f64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => v.to_f64().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigInteger value cannot be converted to f64".to_string(),
                )
            }),
            Value::BigDecimal(v) => v.to_f64().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigDecimal value cannot be converted to f64".to_string(),
                )
            }),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Float64,
            }),
        }
    }

    /// Convert to string
    ///
    /// # Returns
    ///
    /// Converts the value to its string representation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.as_string().unwrap(), "42");
    ///
    /// let value = Value::Bool(true);
    /// assert_eq!(value.as_string().unwrap(), "true");
    /// ```
    pub fn as_string(&self) -> ValueResult<String> {
        match self {
            Value::String(v) => Ok(v.clone()),
            Value::Bool(v) => Ok(v.to_string()),
            Value::Char(v) => Ok(v.to_string()),
            Value::Int8(v) => Ok(v.to_string()),
            Value::Int16(v) => Ok(v.to_string()),
            Value::Int32(v) => Ok(v.to_string()),
            Value::Int64(v) => Ok(v.to_string()),
            Value::Int128(v) => Ok(v.to_string()),
            Value::UInt8(v) => Ok(v.to_string()),
            Value::UInt16(v) => Ok(v.to_string()),
            Value::UInt32(v) => Ok(v.to_string()),
            Value::UInt64(v) => Ok(v.to_string()),
            Value::UInt128(v) => Ok(v.to_string()),
            Value::Float32(v) => Ok(v.to_string()),
            Value::Float64(v) => Ok(v.to_string()),
            Value::Date(v) => Ok(v.to_string()),
            Value::Time(v) => Ok(v.to_string()),
            Value::DateTime(v) => Ok(v.to_string()),
            Value::Instant(v) => Ok(v.to_rfc3339()),
            Value::BigInteger(v) => Ok(v.to_string()),
            Value::BigDecimal(v) => Ok(v.to_string()),
            Value::IntSize(v) => Ok(v.to_string()),
            Value::UIntSize(v) => Ok(v.to_string()),
            Value::Duration(v) => Ok(format!("{}ns", v.as_nanos())),
            Value::Url(v) => Ok(v.to_string()),
            Value::StringMap(v) => serde_json::to_string(v)
                .map_err(|e| ValueError::JsonSerializationError(e.to_string())),
            Value::Json(v) => serde_json::to_string(v)
                .map_err(|e| ValueError::JsonSerializationError(e.to_string())),
            Value::Empty(_) => Err(ValueError::NoValue),
        }
    }

    // ========================================================================
    // Type-setting setters (strict type matching)
    // ========================================================================

    impl_set_value! {
        /// Set boolean value
        ///
        /// # Parameters
        ///
        /// * `value` - The boolean value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        ///
        /// let mut value = Value::Empty(DataType::Bool);
        /// value.set_bool(true).unwrap();
        /// assert_eq!(value.get_bool().unwrap(), true);
        /// ```
        copy: set_bool, Bool, bool, DataType::Bool
    }

    impl_set_value! {
        /// Set character value
        ///
        /// # Parameters
        ///
        /// * `value` - The character value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_char, Char, char, DataType::Char
    }

    impl_set_value! {
        /// Set int8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int8 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_int8, Int8, i8, DataType::Int8
    }

    impl_set_value! {
        /// Set int16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int16 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_int16, Int16, i16, DataType::Int16
    }

    impl_set_value! {
        /// Set int32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_int32, Int32, i32, DataType::Int32
    }

    impl_set_value! {
        /// Set int64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_int64, Int64, i64, DataType::Int64
    }

    impl_set_value! {
        /// Set int128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int128 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_int128, Int128, i128, DataType::Int128
    }

    impl_set_value! {
        /// Set uint8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint8 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_uint8, UInt8, u8, DataType::UInt8
    }

    impl_set_value! {
        /// Set uint16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint16 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_uint16, UInt16, u16, DataType::UInt16
    }

    impl_set_value! {
        /// Set uint32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_uint32, UInt32, u32, DataType::UInt32
    }

    impl_set_value! {
        /// Set uint64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_uint64, UInt64, u64, DataType::UInt64
    }

    impl_set_value! {
        /// Set uint128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint128 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_uint128, UInt128, u128, DataType::UInt128
    }

    impl_set_value! {
        /// Set float32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_float32, Float32, f32, DataType::Float32
    }

    impl_set_value! {
        /// Set float64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_float64, Float64, f64, DataType::Float64
    }

    impl_set_value! {
        /// Set string value
        ///
        /// # Parameters
        ///
        /// * `value` - The string value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        ///
        /// let mut value = Value::Empty(DataType::String);
        /// value.set_string("hello".to_string()).unwrap();
        /// assert_eq!(value.get_string().unwrap(), "hello");
        /// ```
        owned: set_string, String, String, DataType::String
    }

    impl_set_value! {
        /// Set date value
        ///
        /// # Parameters
        ///
        /// * `value` - The date value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_date, Date, NaiveDate, DataType::Date
    }

    impl_set_value! {
        /// Set time value
        ///
        /// # Parameters
        ///
        /// * `value` - The time value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_time, Time, NaiveTime, DataType::Time
    }

    impl_set_value! {
        /// Set datetime value
        ///
        /// # Parameters
        ///
        /// * `value` - The datetime value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_set_value! {
        /// Set UTC instant value
        ///
        /// # Parameters
        ///
        /// * `value` - The UTC instant value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        copy: set_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_set_value! {
        /// Set big integer value
        ///
        /// # Parameters
        ///
        /// * `value` - The big integer value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        /// use num_bigint::BigInt;
        ///
        /// let mut value = Value::Empty(DataType::BigInteger);
        /// value.set_biginteger(BigInt::from(123456789)).unwrap();
        /// assert_eq!(value.get_biginteger().unwrap(), BigInt::from(123456789));
        /// ```
        owned: set_biginteger, BigInteger, BigInt, DataType::BigInteger
    }

    impl_set_value! {
        /// Set big decimal value
        ///
        /// # Parameters
        ///
        /// * `value` - The big decimal value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::Value;
        /// use bigdecimal::BigDecimal;
        ///
        /// let mut value = Value::Empty(DataType::BigDecimal);
        /// value.set_bigdecimal(BigDecimal::from(123.456)).unwrap();
        /// assert_eq!(value.get_bigdecimal().unwrap(), BigDecimal::from(123.456));
        /// ```
        owned: set_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_get_value! {
        /// Get isize value
        ///
        /// # Returns
        ///
        /// If types match, returns the isize value; otherwise returns an error
        copy: get_intsize, IntSize, isize, DataType::IntSize
    }

    impl_get_value! {
        /// Get usize value
        ///
        /// # Returns
        ///
        /// If types match, returns the usize value; otherwise returns an error
        copy: get_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_get_value! {
        /// Get Duration value
        ///
        /// # Returns
        ///
        /// If types match, returns the Duration value; otherwise returns an error
        copy: get_duration, Duration, Duration, DataType::Duration
    }

    impl_get_value! {
        /// Get Url reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the Url; otherwise returns an error
        ref: get_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_value! {
        /// Get StringMap reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the HashMap<String, String>; otherwise returns an error
        ref: get_string_map, StringMap, HashMap<String, String>, DataType::StringMap, |v: &HashMap<String, String>| v.clone()
    }

    impl_get_value! {
        /// Get Json value reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the serde_json::Value; otherwise returns an error
        ref: get_json, Json, serde_json::Value, DataType::Json, |v: &serde_json::Value| v.clone()
    }

    impl_set_value! {
        /// Set isize value
        copy: set_intsize, IntSize, isize, DataType::IntSize
    }

    impl_set_value! {
        /// Set usize value
        copy: set_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_set_value! {
        /// Set Duration value
        copy: set_duration, Duration, Duration, DataType::Duration
    }

    impl_set_value! {
        /// Set Url value
        owned: set_url, Url, Url, DataType::Url
    }

    impl_set_value! {
        /// Set StringMap value
        owned: set_string_map, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_set_value! {
        /// Set Json value
        owned: set_json, Json, serde_json::Value, DataType::Json
    }

    /// Create a `Value` from a `serde_json::Value`
    ///
    /// # Parameters
    ///
    /// * `json` - The JSON value to wrap
    ///
    /// # Returns
    ///
    /// Returns a `Value::Json` wrapping the given JSON value
    pub fn from_json_value(json: serde_json::Value) -> Self {
        Value::Json(json)
    }

    /// Create a `Value` from any serializable value by converting it to JSON
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type implementing `Serialize`
    ///
    /// # Parameters
    ///
    /// * `value` - The value to serialize into JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value::Json(...))` on success, or an error if serialization fails
    pub fn from_serializable<T: Serialize>(value: &T) -> ValueResult<Self> {
        let json = serde_json::to_value(value)
            .map_err(|e| ValueError::JsonSerializationError(e.to_string()))?;
        Ok(Value::Json(json))
    }

    /// Deserialize the inner JSON value into a target type
    ///
    /// Only works when `self` is `Value::Json(...)`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type implementing `DeserializeOwned`
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` on success, or an error if the value is not JSON or deserialization fails
    pub fn deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T> {
        match self {
            Value::Json(v) => serde_json::from_value(v.clone())
                .map_err(|e| ValueError::JsonDeserializationError(e.to_string())),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Json,
            }),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Empty(DataType::String)
    }
}

// ============================================================================
// Internal generic conversion traits (private, not exported, to avoid polluting the standard type namespace)
// ============================================================================

/// Internal trait: used to extract specific types from Value
///
/// This trait is not exported in mod.rs, only used for internal implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueGetter<T> {
    fn get_value(&self) -> ValueResult<T>;
}

/// Internal trait: used to create Value from types
///
/// This trait is not exported in mod.rs, only used for internal implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueConstructor<T> {
    fn from_type(value: T) -> Self;
}

/// Internal trait: used to set specific types in Value
///
/// This trait is not exported in mod.rs, only used for internal implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueSetter<T> {
    fn set_value(&mut self, value: T) -> ValueResult<()>;
}

// ============================================================================
// Implementation of internal traits (simplified using macros)
// ============================================================================

macro_rules! impl_value_traits {
    ($type:ty, $variant:ident, $get_method:ident, $set_method:ident) => {
        impl ValueGetter<$type> for Value {
            fn get_value(&self) -> ValueResult<$type> {
                self.$get_method()
            }
        }

        impl ValueSetter<$type> for Value {
            fn set_value(&mut self, value: $type) -> ValueResult<()> {
                self.$set_method(value)
            }
        }

        impl ValueConstructor<$type> for Value {
            fn from_type(value: $type) -> Self {
                Value::$variant(value)
            }
        }
    };
}

// Implementation for Copy types
impl_value_traits!(bool, Bool, get_bool, set_bool);
impl_value_traits!(char, Char, get_char, set_char);
impl_value_traits!(i8, Int8, get_int8, set_int8);
impl_value_traits!(i16, Int16, get_int16, set_int16);
impl_value_traits!(i32, Int32, get_int32, set_int32);
impl_value_traits!(i64, Int64, get_int64, set_int64);
impl_value_traits!(i128, Int128, get_int128, set_int128);
impl_value_traits!(u8, UInt8, get_uint8, set_uint8);
impl_value_traits!(u16, UInt16, get_uint16, set_uint16);
impl_value_traits!(u32, UInt32, get_uint32, set_uint32);
impl_value_traits!(u64, UInt64, get_uint64, set_uint64);
impl_value_traits!(u128, UInt128, get_uint128, set_uint128);
impl_value_traits!(f32, Float32, get_float32, set_float32);
impl_value_traits!(f64, Float64, get_float64, set_float64);
impl_value_traits!(NaiveDate, Date, get_date, set_date);
impl_value_traits!(NaiveTime, Time, get_time, set_time);
impl_value_traits!(NaiveDateTime, DateTime, get_datetime, set_datetime);
impl_value_traits!(DateTime<Utc>, Instant, get_instant, set_instant);
impl_value_traits!(BigInt, BigInteger, get_biginteger, set_biginteger);
impl_value_traits!(BigDecimal, BigDecimal, get_bigdecimal, set_bigdecimal);
impl_value_traits!(isize, IntSize, get_intsize, set_intsize);
impl_value_traits!(usize, UIntSize, get_uintsize, set_uintsize);
impl_value_traits!(Duration, Duration, get_duration, set_duration);

// String needs cloning
impl ValueGetter<String> for Value {
    fn get_value(&self) -> ValueResult<String> {
        self.get_string().map(|s| s.to_string())
    }
}

impl ValueSetter<String> for Value {
    fn set_value(&mut self, value: String) -> ValueResult<()> {
        self.set_string(value)
    }
}

impl ValueConstructor<String> for Value {
    fn from_type(value: String) -> Self {
        Value::String(value)
    }
}

// Special handling for &str - convert to String
impl ValueSetter<&str> for Value {
    fn set_value(&mut self, value: &str) -> ValueResult<()> {
        self.set_string(value.to_string())
    }
}

impl ValueConstructor<&str> for Value {
    fn from_type(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

// Url
impl ValueGetter<Url> for Value {
    fn get_value(&self) -> ValueResult<Url> {
        self.get_url()
    }
}

impl ValueSetter<Url> for Value {
    fn set_value(&mut self, value: Url) -> ValueResult<()> {
        self.set_url(value)
    }
}

impl ValueConstructor<Url> for Value {
    fn from_type(value: Url) -> Self {
        Value::Url(value)
    }
}

// HashMap<String, String>
impl ValueGetter<HashMap<String, String>> for Value {
    fn get_value(&self) -> ValueResult<HashMap<String, String>> {
        self.get_string_map()
    }
}

impl ValueSetter<HashMap<String, String>> for Value {
    fn set_value(&mut self, value: HashMap<String, String>) -> ValueResult<()> {
        self.set_string_map(value)
    }
}

impl ValueConstructor<HashMap<String, String>> for Value {
    fn from_type(value: HashMap<String, String>) -> Self {
        Value::StringMap(value)
    }
}

// serde_json::Value
impl ValueGetter<serde_json::Value> for Value {
    fn get_value(&self) -> ValueResult<serde_json::Value> {
        self.get_json()
    }
}

impl ValueSetter<serde_json::Value> for Value {
    fn set_value(&mut self, value: serde_json::Value) -> ValueResult<()> {
        self.set_json(value)
    }
}

impl ValueConstructor<serde_json::Value> for Value {
    fn from_type(value: serde_json::Value) -> Self {
        Value::Json(value)
    }
}
