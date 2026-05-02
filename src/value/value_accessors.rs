/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use num_bigint::BigInt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use qubit_datatype::DataType;

use super::value::Value;
use crate::value_error::{
    ValueError,
    ValueResult,
};

macro_rules! impl_get_value {
    // Copy type: directly dereference and return
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
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
        #[inline]
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
///
macro_rules! impl_set_value {
    // Copy type: directly set the value
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            *self = Value::$variant(value);
            Ok(())
        }
    };

    // Owned type: set the owned value
    ($(#[$attr:meta])* owned: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            *self = Value::$variant(value);
            Ok(())
        }
    };
}

impl Value {
    // ========================================================================
    // Type-checking getters (strict type matching)
    // ========================================================================

    impl_get_value! {
        /// Get boolean value
        ///
        /// # Returns
        ///
        /// If types match, returns the boolean value; otherwise returns an
        /// error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::Value;
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
        /// If types match, returns the character value; otherwise returns an
        /// error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::Value;
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
        /// If types match, returns the int8 value; otherwise returns an error.
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
        /// If types match, returns the int32 value; otherwise returns an error.
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
        /// If types match, returns the int128 value; otherwise returns an error.
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
        /// If types match, returns the uint16 value; otherwise returns an error.
        copy: get_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_value! {
        /// Get uint32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint32 value; otherwise returns an error.
        copy: get_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_value! {
        /// Get uint64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint64 value; otherwise returns an error.
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
        /// If types match, returns the float32 value; otherwise returns an error.
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
        /// If types match, returns a reference to the string; otherwise returns
        /// an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::Value;
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
        /// If types match, returns the date value; otherwise returns an error.
        copy: get_date, Date, NaiveDate, DataType::Date
    }

    impl_get_value! {
        /// Get time value
        ///
        /// # Returns
        ///
        /// If types match, returns the time value; otherwise returns an error.
        copy: get_time, Time, NaiveTime, DataType::Time
    }

    impl_get_value! {
        /// Get datetime value
        ///
        /// # Returns
        ///
        /// If types match, returns the datetime value; otherwise returns an error.
        copy: get_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_value! {
        /// Get UTC instant value
        ///
        /// # Returns
        ///
        /// If types match, returns the UTC instant value; otherwise returns an error.
        copy: get_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_value! {
        /// Get big integer value.
        ///
        /// This method returns a cloned [`BigInt`]. Use
        /// [`Value::get_biginteger_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big integer value; otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::Value;
        /// use num_bigint::BigInt;
        ///
        /// let value = Value::BigInteger(BigInt::from(123456789));
        /// assert_eq!(value.get_biginteger().unwrap(), BigInt::from(123456789));
        /// ```
        ref: get_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    impl_get_value! {
        /// Get big decimal value.
        ///
        /// This method returns a cloned [`BigDecimal`]. Use
        /// [`Value::get_bigdecimal_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big decimal value; otherwise returns an
        /// error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use std::str::FromStr;
        ///
        /// use bigdecimal::BigDecimal;
        /// use qubit_value::Value;
        ///
        /// let bd = BigDecimal::from_str("123.456").unwrap();
        /// let value = Value::BigDecimal(bd.clone());
        /// assert_eq!(value.get_bigdecimal().unwrap(), bd);
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::Value;
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::Value;
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use num_bigint::BigInt;
        /// use qubit_datatype::DataType;
        /// use qubit_value::Value;
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
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use std::str::FromStr;
        ///
        /// use bigdecimal::BigDecimal;
        /// use qubit_datatype::DataType;
        /// use qubit_value::Value;
        ///
        /// let mut value = Value::Empty(DataType::BigDecimal);
        /// let bd = BigDecimal::from_str("123.456").unwrap();
        /// value.set_bigdecimal(bd.clone()).unwrap();
        /// assert_eq!(value.get_bigdecimal().unwrap(), bd);
        /// ```
        owned: set_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_get_value! {
        /// Get isize value
        ///
        /// # Returns
        ///
        /// If types match, returns the isize value; otherwise returns an error.
        copy: get_intsize, IntSize, isize, DataType::IntSize
    }

    impl_get_value! {
        /// Get usize value
        ///
        /// # Returns
        ///
        /// If types match, returns the usize value; otherwise returns an error.
        copy: get_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_get_value! {
        /// Get Duration value
        ///
        /// # Returns
        ///
        /// If types match, returns the Duration value; otherwise returns an
        /// error.
        copy: get_duration, Duration, Duration, DataType::Duration
    }

    impl_get_value! {
        /// Get URL value.
        ///
        /// This method returns a cloned [`Url`]. Use [`Value::get_url_ref`] to
        /// borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the URL value; otherwise returns an error.
        ref: get_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_value! {
        /// Get string map value.
        ///
        /// This method returns a cloned `HashMap<String, String>`. Use
        /// [`Value::get_string_map_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the string map value; otherwise returns an
        /// error.
        ref: get_string_map, StringMap, HashMap<String, String>, DataType::StringMap,
            |v: &HashMap<String, String>| v.clone()
    }

    impl_get_value! {
        /// Get JSON value.
        ///
        /// This method returns a cloned [`serde_json::Value`]. Use
        /// [`Value::get_json_ref`] to borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the JSON value; otherwise returns an error.
        ref: get_json, Json, serde_json::Value, DataType::Json,
            |v: &serde_json::Value| v.clone()
    }

    /// Borrow the inner `BigInt` without cloning.
    pub fn get_biginteger_ref(&self) -> ValueResult<&BigInt> {
        match self {
            Value::BigInteger(v) => Ok(v),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigInteger,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `BigDecimal` without cloning.
    pub fn get_bigdecimal_ref(&self) -> ValueResult<&BigDecimal> {
        match self {
            Value::BigDecimal(v) => Ok(v),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigDecimal,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `Url` without cloning.
    pub fn get_url_ref(&self) -> ValueResult<&Url> {
        match self {
            Value::Url(v) => Ok(v),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Url,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `HashMap<String, String>` without cloning.
    pub fn get_string_map_ref(&self) -> ValueResult<&HashMap<String, String>> {
        match self {
            Value::StringMap(v) => Ok(v),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::StringMap,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner JSON value without cloning.
    pub fn get_json_ref(&self) -> ValueResult<&serde_json::Value> {
        match self {
            Value::Json(v) => Ok(v),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: self.data_type(),
            }),
        }
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

    /// Create a `Value` from a `serde_json::Value`.
    ///
    /// # Parameters
    ///
    /// * `json` - The JSON value to wrap.
    ///
    /// # Returns
    ///
    /// Returns a `Value::Json` wrapping the given JSON value.
    #[inline]
    pub fn from_json_value(json: serde_json::Value) -> Self {
        Value::Json(json)
    }

    /// Create a `Value` from any serializable value by converting it to JSON.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type implementing `Serialize`.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to serialize into JSON.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value::Json(...))` on success, or an error if
    /// serialization fails.
    pub fn from_serializable<T: Serialize>(value: &T) -> ValueResult<Self> {
        let json = serde_json::to_value(value)
            .map_err(|e| ValueError::JsonSerializationError(e.to_string()))?;
        Ok(Value::Json(json))
    }

    /// Deserialize the inner JSON value into a target type.
    ///
    /// Only works when `self` is `Value::Json(...)`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type implementing `DeserializeOwned`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` on success, or an error if the value is not JSON
    /// or deserialization fails.
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
