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

use qubit_common::lang::argument::NumericArgument;
use qubit_common::lang::DataType;

use super::error::{ValueError, ValueResult};

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
/// ```rust,ignore
/// use common_rs::util::value::Value;
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
/// The macro automatically extracts preceding documentation comments, so
/// you can add `///` comments before macro invocations.
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

    // Reference type: use conversion function to return reference,
    // fixing lifetime issues
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
/// The macro automatically extracts preceding documentation comments, so
/// you can add `///` comments before macro invocations.
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
    /// Automatically selects the correct getter method based on the target
    /// type, performing strict type checking.
    ///
    /// `get<T>()` performs strict type matching. It does not do cross-type conversion.
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

    /// Generic conversion method
    ///
    /// Converts the current value to the target type according to the conversion
    /// rules defined by [`ValueConverter<T>`].
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
    ///   - `Value::String`, parsed as `bool`
    /// - `char`
    ///   - `Value::Char`
    /// - `i8`
    ///   - `Value::Int8`
    /// - `i16`
    ///   - `Value::Int16`
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
    /// - `usize`
    ///   - `Value::UIntSize`
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
    /// ```rust,ignore
    /// use crate::util::value::Value;
    ///
    /// let value = Value::Int32(42);
    ///
    /// let num: i64 = value.to().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// let text: String = value.to().unwrap();
    /// assert_eq!(text, "42");
    /// ```
    pub fn to<T>(&self) -> ValueResult<T>
    where
        Self: ValueConverter<T>,
    {
        <Self as ValueConverter<T>>::convert(self)
    }

    /// Generic setter method
    ///
    /// Automatically selects the correct setter method based on the target
    /// type, performing strict type checking.
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
    /// If the new type differs from the current type, clears the value
    /// and sets the new type.
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
        /// If types match, returns a reference to the Url; otherwise
        /// returns an error
        ref: get_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_value! {
        /// Get StringMap reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the
        /// `HashMap<String, String>`; otherwise returns an error
        ref: get_string_map, StringMap, HashMap<String, String>, DataType::StringMap,
            |v: &HashMap<String, String>| v.clone()
    }

    impl_get_value! {
        /// Get Json value reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the `serde_json::Value`;
        /// otherwise returns an error
        ref: get_json, Json, serde_json::Value, DataType::Json,
            |v: &serde_json::Value| v.clone()
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
    /// Returns `Ok(Value::Json(...))` on success, or an error if
    /// serialization fails
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
    /// Returns `Ok(T)` on success, or an error if the value is not JSON
    /// or deserialization fails
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

fn parse_duration_string(s: &str) -> ValueResult<Duration> {
    let trimmed = s.trim();
    let nanos_text = trimmed.strip_suffix("ns").ok_or_else(|| {
        ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: expected '<nanoseconds>ns'",
            s
        ))
    })?;
    let total_nanos = nanos_text.parse::<u128>().map_err(|_| {
        ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: invalid nanoseconds value",
            s
        ))
    })?;
    let secs = total_nanos / 1_000_000_000;
    if secs > u64::MAX as u128 {
        return Err(ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: value out of range",
            s
        )));
    }
    let nanos = (total_nanos % 1_000_000_000) as u32;
    Ok(Duration::new(secs as u64, nanos))
}

fn range_check<T>(value: T, min: T, max: T, target: &str) -> ValueResult<T>
where
    T: NumericArgument + Copy,
{
    value
        .require_in_closed_range("value", min, max)
        .map_err(|e| {
            ValueError::ConversionError(format!("Cannot convert value to {}: {}", target, e))
        })
}

// ============================================================================
// Internal generic conversion traits (private, not exported, to avoid
// polluting the standard type namespace)
// ============================================================================

/// Internal trait: used to extract specific types from Value
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueGetter<T> {
    fn get_value(&self) -> ValueResult<T>;
}

/// Internal trait: used to create Value from types
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueConstructor<T> {
    fn from_type(value: T) -> Self;
}

/// Internal trait: used to set specific types in Value
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait ValueSetter<T> {
    fn set_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to convert Value to target types
///
/// This trait powers `Value::to<T>()`. Each implementation must clearly define
/// which source variants are accepted for the target type.
#[doc(hidden)]
pub trait ValueConverter<T> {
    fn convert(&self) -> ValueResult<T>;
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

macro_rules! impl_strict_value_converter {
    ($(#[$attr:meta])* $type:ty, $get_method:ident) => {
        $(#[$attr])*
        impl ValueConverter<$type> for Value {
            fn convert(&self) -> ValueResult<$type> {
                self.$get_method()
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

/// Target type `String` supports conversion from:
///
/// - `Value::String`
/// - `Value::Bool`, `Value::Char`
/// - all integer and floating-point variants
/// - `Value::Date`, `Value::Time`, `Value::DateTime`, `Value::Instant`
/// - `Value::BigInteger`, `Value::BigDecimal`
/// - `Value::IntSize`, `Value::UIntSize`
/// - `Value::Duration`, formatted as `<nanoseconds>ns`
/// - `Value::Url`
/// - `Value::StringMap`, serialized as JSON text
/// - `Value::Json`, serialized as JSON text
impl ValueConverter<String> for Value {
    fn convert(&self) -> ValueResult<String> {
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

/// Target type `bool` supports conversion from:
///
/// - `Value::Bool`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::String`, parsed as `bool`
impl ValueConverter<bool> for Value {
    fn convert(&self) -> ValueResult<bool> {
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
}

/// Target type `i32` supports conversion from:
///
/// - `Value::Int32`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int64`, `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`, `Value::Float64`
/// - `Value::String`, parsed as `i32`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<i32> for Value {
    fn convert(&self) -> ValueResult<i32> {
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
}

/// Target type `i64` supports conversion from:
///
/// - `Value::Int64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`, `Value::Float64`
/// - `Value::String`, parsed as `i64`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<i64> for Value {
    fn convert(&self) -> ValueResult<i64> {
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
}

/// Target type `f64` supports conversion from:
///
/// - `Value::Float64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`
/// - `Value::String`, parsed as `f64`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<f64> for Value {
    fn convert(&self) -> ValueResult<f64> {
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
            Value::BigInteger(v) => Ok(v.to_f64().unwrap_or(f64::INFINITY)),
            Value::BigDecimal(v) => Ok(v.to_f64().unwrap_or(f64::INFINITY)),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Float64,
            }),
        }
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

/// Target type `Duration` supports conversion from:
///
/// - `Value::Duration`
/// - `Value::String`, parsed from `<nanoseconds>ns`
impl ValueConverter<Duration> for Value {
    fn convert(&self) -> ValueResult<Duration> {
        match self {
            Value::Duration(v) => Ok(*v),
            Value::String(s) => parse_duration_string(s),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Duration,
            }),
        }
    }
}

/// Target type `Url` supports conversion from:
///
/// - `Value::Url`
/// - `Value::String`, parsed as URL text
impl ValueConverter<Url> for Value {
    fn convert(&self) -> ValueResult<Url> {
        match self {
            Value::Url(v) => Ok(v.clone()),
            Value::String(s) => Url::parse(s).map_err(|e| {
                ValueError::ConversionError(format!("Cannot convert '{}' to Url: {}", s, e))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Url,
            }),
        }
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

/// Target type `serde_json::Value` supports conversion from:
///
/// - `Value::Json`
/// - `Value::String`, parsed as JSON text
/// - `Value::StringMap`, converted to a JSON object
impl ValueConverter<serde_json::Value> for Value {
    fn convert(&self) -> ValueResult<serde_json::Value> {
        match self {
            Value::Json(v) => Ok(v.clone()),
            Value::String(s) => serde_json::from_str(s)
                .map_err(|e| ValueError::JsonDeserializationError(e.to_string())),
            Value::StringMap(v) => serde_json::to_value(v)
                .map_err(|e| ValueError::JsonSerializationError(e.to_string())),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Json,
            }),
        }
    }
}

impl_strict_value_converter!(
    /// Target type `char` supports conversion from:
    ///
    /// - `Value::Char`
    char,
    get_char
);
impl_strict_value_converter!(
    /// Target type `i8` supports conversion from:
    ///
    /// - `Value::Int8`
    i8,
    get_int8
);
impl_strict_value_converter!(
    /// Target type `i16` supports conversion from:
    ///
    /// - `Value::Int16`
    i16,
    get_int16
);
impl_strict_value_converter!(
    /// Target type `i128` supports conversion from:
    ///
    /// - `Value::Int128`
    i128,
    get_int128
);
/// Target type `u8` supports conversion from:
///
/// - `Value::UInt8`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt16`, `Value::UInt32`, `Value::UInt64`, `Value::UInt128`
/// - `Value::String`, parsed as `u8`
impl ValueConverter<u8> for Value {
    fn convert(&self) -> ValueResult<u8> {
        match self {
            Value::UInt8(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => {
                let code = range_check(*v as u32, u8::MIN as u32, u8::MAX as u32, "u8")?;
                Ok(code as u8)
            }
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u8")?;
                Ok(n as u8)
            }
            Value::Int16(v) => {
                let n = range_check(*v, u8::MIN as i16, u8::MAX as i16, "u8")?;
                Ok(n as u8)
            }
            Value::Int32(v) => {
                let n = range_check(*v, u8::MIN as i32, u8::MAX as i32, "u8")?;
                Ok(n as u8)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u8::MIN as i64, u8::MAX as i64, "u8")?;
                Ok(n as u8)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u8::MIN as i128, u8::MAX as i128, "u8")?;
                Ok(n as u8)
            }
            Value::UInt16(v) => {
                let n = range_check(*v, u8::MIN as u16, u8::MAX as u16, "u8")?;
                Ok(n as u8)
            }
            Value::UInt32(v) => {
                let n = range_check(*v, u8::MIN as u32, u8::MAX as u32, "u8")?;
                Ok(n as u8)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u8::MIN as u64, u8::MAX as u64, "u8")?;
                Ok(n as u8)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u8::MIN as u128, u8::MAX as u128, "u8")?;
                Ok(n as u8)
            }
            Value::String(s) => s
                .parse::<u8>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u8", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt8,
            }),
        }
    }
}

/// Target type `u16` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u16`
impl ValueConverter<u16> for Value {
    fn convert(&self) -> ValueResult<u16> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => {
                let code = range_check(*v as u32, u16::MIN as u32, u16::MAX as u32, "u16")?;
                Ok(code as u16)
            }
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u16")?;
                Ok(n as u16)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u16")?;
                Ok(n as u16)
            }
            Value::Int32(v) => {
                let n = range_check(*v, u16::MIN as i32, u16::MAX as i32, "u16")?;
                Ok(n as u16)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u16::MIN as i64, u16::MAX as i64, "u16")?;
                Ok(n as u16)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u16::MIN as i128, u16::MAX as i128, "u16")?;
                Ok(n as u16)
            }
            Value::UInt32(v) => {
                let n = range_check(*v, u16::MIN as u32, u16::MAX as u32, "u16")?;
                Ok(n as u16)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u16::MIN as u64, u16::MAX as u64, "u16")?;
                Ok(n as u16)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u16::MIN as u128, u16::MAX as u128, "u16")?;
                Ok(n as u16)
            }
            Value::String(s) => s
                .parse::<u16>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u16", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt16,
            }),
        }
    }
}

/// Target type `u32` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u32`
impl ValueConverter<u32> for Value {
    fn convert(&self) -> ValueResult<u32> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as u32),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u32::MIN as i64, u32::MAX as i64, "u32")?;
                Ok(n as u32)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u32::MIN as i128, u32::MAX as i128, "u32")?;
                Ok(n as u32)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u32::MIN as u64, u32::MAX as u64, "u32")?;
                Ok(n as u32)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u32::MIN as u128, u32::MAX as u128, "u32")?;
                Ok(n as u32)
            }
            Value::String(s) => s
                .parse::<u32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt32,
            }),
        }
    }
}

/// Target type `u64` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u64`
impl ValueConverter<u64> for Value {
    fn convert(&self) -> ValueResult<u64> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok((*v).into()),
            Value::UInt64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok((*v as u32).into()),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int64(v) => {
                let n = range_check(*v, 0i64, i64::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int128(v) => {
                let n = range_check(*v, 0i128, u64::MAX as i128, "u64")?;
                Ok(n as u64)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u64::MIN as u128, u64::MAX as u128, "u64")?;
                Ok(n as u64)
            }
            Value::String(s) => s
                .parse::<u64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt64,
            }),
        }
    }
}

/// Target type `u128` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u128`
impl ValueConverter<u128> for Value {
    fn convert(&self) -> ValueResult<u128> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok((*v).into()),
            Value::UInt64(v) => Ok((*v).into()),
            Value::UInt128(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok((*v as u32).into()),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int64(v) => {
                let n = range_check(*v, 0i64, i64::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int128(v) => {
                let n = range_check(*v, 0i128, i128::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::String(s) => s.parse::<u128>().map_err(|_| {
                ValueError::ConversionError(format!("Cannot convert '{}' to u128", s))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt128,
            }),
        }
    }
}

/// Target type `f32` supports conversion from:
///
/// - `Value::Float32`, `Value::Float64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::String`, parsed as `f32`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<f32> for Value {
    fn convert(&self) -> ValueResult<f32> {
        match self {
            Value::Float32(v) => Ok(*v),
            Value::Float64(v) => {
                if v.is_nan() || v.is_infinite() {
                    Ok(*v as f32)
                } else {
                    let n = range_check(*v, f32::MIN as f64, f32::MAX as f64, "f32")?;
                    Ok(n as f32)
                }
            }
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Char(v) => Ok(*v as u32 as f32),
            Value::Int8(v) => Ok(*v as f32),
            Value::Int16(v) => Ok(*v as f32),
            Value::Int32(v) => Ok(*v as f32),
            Value::Int64(v) => Ok(*v as f32),
            Value::Int128(v) => Ok(*v as f32),
            Value::UInt8(v) => Ok(*v as f32),
            Value::UInt16(v) => Ok(*v as f32),
            Value::UInt32(v) => Ok(*v as f32),
            Value::UInt64(v) => Ok(*v as f32),
            Value::UInt128(v) => Ok(*v as f32),
            Value::String(s) => s
                .parse::<f32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to f32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => Ok(v.to_f32().unwrap_or(f32::INFINITY)),
            Value::BigDecimal(v) => Ok(v.to_f32().unwrap_or(f32::INFINITY)),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Float32,
            }),
        }
    }
}
impl_strict_value_converter!(
    /// Target type `NaiveDate` supports conversion from:
    ///
    /// - `Value::Date`
    NaiveDate,
    get_date
);
impl_strict_value_converter!(
    /// Target type `NaiveTime` supports conversion from:
    ///
    /// - `Value::Time`
    NaiveTime,
    get_time
);
impl_strict_value_converter!(
    /// Target type `NaiveDateTime` supports conversion from:
    ///
    /// - `Value::DateTime`
    NaiveDateTime,
    get_datetime
);
impl_strict_value_converter!(
    /// Target type `DateTime<Utc>` supports conversion from:
    ///
    /// - `Value::Instant`
    DateTime<Utc>,
    get_instant
);
impl_strict_value_converter!(
    /// Target type `BigInt` supports conversion from:
    ///
    /// - `Value::BigInteger`
    BigInt,
    get_biginteger
);
impl_strict_value_converter!(
    /// Target type `BigDecimal` supports conversion from:
    ///
    /// - `Value::BigDecimal`
    BigDecimal,
    get_bigdecimal
);
impl_strict_value_converter!(
    /// Target type `isize` supports conversion from:
    ///
    /// - `Value::IntSize`
    isize,
    get_intsize
);
impl_strict_value_converter!(
    /// Target type `usize` supports conversion from:
    ///
    /// - `Value::UIntSize`
    usize,
    get_uintsize
);
impl_strict_value_converter!(
    /// Target type `HashMap<String, String>` supports conversion from:
    ///
    /// - `Value::StringMap`
    HashMap<String, String>,
    get_string_map
);
