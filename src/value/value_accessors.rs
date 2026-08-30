// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict typed accessors for scalar runtime values.
// qubit-style: allow source-test-pair
// Tests are intentionally distributed across behavior-specific files under
// tests/value/ rather than collected in value_accessors_tests.rs.

use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::DateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "chrono")]
use chrono::NaiveDateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveTime;
#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::DataConversionError;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::DataFormat;
use qubit_datatype::DataType;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::InvalidValueReason;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_json::value::JsonValueEncodeError;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_json::value::JsonValueEncoder;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Deserialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::de::DeserializeOwned;
#[cfg(feature = "url")]
use url::Url;

use super::value::Value;
use super::value::ValueRepr;
use crate::ValueMissing;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

/// Implements one strict typed getter from the shared value table.
macro_rules! impl_get_value {
    // Copy type: directly dereference and return
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the value is unset with"]
        #[doc = "the requested type, or [`ValueError::TypeMismatch`] when the"]
        #[doc = "stored data type differs."]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$type> {
            match &self.repr {
                ValueRepr::$variant(v) => Ok(*v),
                ValueRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetScalar {
                        data_type: *dt,
                    }))
                }
                ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: *dt,
                }),
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
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the value is unset with"]
        #[doc = "the requested type, or [`ValueError::TypeMismatch`] when the"]
        #[doc = "stored data type differs."]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match &self.repr {
                ValueRepr::$variant(v) => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(v))
                },
                ValueRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetScalar {
                        data_type: *dt,
                    }))
                }
                ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: *dt,
                }),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

impl Value {
    /// Creates a `Value` from a `serde_json::Value`.
    ///
    /// # Parameters
    ///
    /// * `json` - The JSON value to wrap.
    ///
    /// # Returns
    ///
    /// A `Value::Json` wrapping the given JSON value.
    #[inline(always)]
    #[cfg(feature = "json")]
    pub fn from_json_value(json: serde_json::Value) -> Self {
        Value::Json(json)
    }

    /// Creates a `Value` from any serializable value by converting it to JSON.
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
    /// A `Value::Json` containing the serialized representation.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Conversion`] with
    /// [`InvalidValueReason::NonFinite`] when any nested float is non-finite,
    /// or [`InvalidValueReason::Serialization`] when Serde cannot represent
    /// the input as JSON.
    #[cfg(all(feature = "converter", feature = "json"))]
    pub fn from_serializable<T: ?Sized + Serialize>(value: &T) -> ValueResult<Self> {
        let json = JsonValueEncoder::new().encode(value).map_err(|error| {
            let reason = match error {
                JsonValueEncodeError::NonFiniteFloat => InvalidValueReason::NonFinite,
                JsonValueEncodeError::Serialization => InvalidValueReason::Serialization {
                    format: DataFormat::Json,
                },
            };
            ValueError::from(DataConversionError::invalid(DataType::Json, DataType::Json, reason))
        })?;
        Ok(Value::Json(json))
    }

    // ========================================================================
    // Type-checking getters (strict type matching)
    // ========================================================================

    impl_get_value! {
        /// Get boolean value
        ///
        /// # Returns
        ///
        /// If types match, returns the boolean value; see `# Errors`.
        ///
        /// # Examples
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
        /// If types match, returns the character value; see `# Errors`.
        ///
        /// # Examples
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
        /// If types match, returns the int8 value; see `# Errors`.
        copy: get_int8, Int8, i8, DataType::Int8
    }

    impl_get_value! {
        /// Get int16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int16 value; see `# Errors`.
        copy: get_int16, Int16, i16, DataType::Int16
    }

    impl_get_value! {
        /// Get int32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int32 value; see `# Errors`.
        copy: get_int32, Int32, i32, DataType::Int32
    }

    impl_get_value! {
        /// Get int64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int64 value; see `# Errors`.
        copy: get_int64, Int64, i64, DataType::Int64
    }

    impl_get_value! {
        /// Get int128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int128 value; see `# Errors`.
        copy: get_int128, Int128, i128, DataType::Int128
    }

    impl_get_value! {
        /// Get uint8 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint8 value; see `# Errors`.
        copy: get_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_value! {
        /// Get uint16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint16 value; see `# Errors`.
        copy: get_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_value! {
        /// Get uint32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint32 value; see `# Errors`.
        copy: get_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_value! {
        /// Get uint64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint64 value; see `# Errors`.
        copy: get_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_value! {
        /// Get uint128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint128 value; see `# Errors`.
        copy: get_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_value! {
        /// Get float32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float32 value; see `# Errors`.
        copy: get_float32, Float32, f32, DataType::Float32
    }

    impl_get_value! {
        /// Get float64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float64 value; see `# Errors`.
        copy: get_float64, Float64, f64, DataType::Float64
    }

    impl_get_value! {
        /// Get string reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::String("hello".to_string());
        /// assert_eq!(value.get_string().unwrap(), "hello");
        /// ```
        ref: get_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get date value
        ///
        /// # Returns
        ///
        /// If types match, returns the date value; see `# Errors`.
        copy: get_date, Date, NaiveDate, DataType::Date
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get time value
        ///
        /// # Returns
        ///
        /// If types match, returns the time value; see `# Errors`.
        copy: get_time, Time, NaiveTime, DataType::Time
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get datetime value
        ///
        /// # Returns
        ///
        /// If types match, returns the datetime value; see `# Errors`.
        copy: get_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get UTC instant value
        ///
        /// # Returns
        ///
        /// If types match, returns the UTC instant value; see `# Errors`.
        copy: get_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    #[cfg(feature = "big-integer")]
    impl_get_value! {
        /// Get big integer value.
        ///
        /// This method returns a cloned [`BigInt`]. Use
        /// [`Value::get_biginteger_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big integer value; see `# Errors`.
        ///
        /// # Examples
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

    #[cfg(feature = "big-decimal")]
    impl_get_value! {
        /// Get big decimal value.
        ///
        /// This method returns a cloned [`BigDecimal`]. Use
        /// [`Value::get_bigdecimal_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big decimal value; see `# Errors`.
        ///
        /// # Examples
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

    impl_get_value! {
        /// Get Duration value
        ///
        /// # Returns
        ///
        /// If types match, returns the Duration value; see `# Errors`.
        copy: get_duration, Duration, Duration, DataType::Duration
    }

    #[cfg(feature = "url")]
    impl_get_value! {
        /// Get URL value.
        ///
        /// This method returns a cloned [`Url`]. Use [`Value::get_url_ref`] to
        /// borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the URL value; see `# Errors`.
        ref: get_url, Url, Url, DataType::Url, Url::clone
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
        /// If types match, returns the string map value; see `# Errors`.
        ref: get_string_map, StringMap, HashMap<String, String>, DataType::StringMap,
            |v: &HashMap<String, String>| v.clone()
    }

    #[cfg(feature = "json")]
    impl_get_value! {
        /// Get JSON value.
        ///
        /// This method returns a cloned [`serde_json::Value`]. Use
        /// [`Value::get_json_ref`] to borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the JSON value; see `# Errors`.
        ref: get_json, Json, serde_json::Value, DataType::Json,
            |v: &serde_json::Value| v.clone()
    }

    /// Borrow the inner `BigInt` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored integer.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::BigInteger`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[cfg(feature = "big-integer")]
    #[inline(always)]
    pub fn get_biginteger_ref(&self) -> ValueResult<&BigInt> {
        match &self.repr {
            ValueRepr::BigInteger(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::BigInteger => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::BigInteger,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigInteger,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `BigDecimal` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored decimal.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::BigDecimal`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[cfg(feature = "big-decimal")]
    #[inline(always)]
    pub fn get_bigdecimal_ref(&self) -> ValueResult<&BigDecimal> {
        match &self.repr {
            ValueRepr::BigDecimal(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::BigDecimal => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::BigDecimal,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigDecimal,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `Url` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored URL.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::Url`, or [`ValueError::TypeMismatch`] when the stored data
    /// type differs.
    #[cfg(feature = "url")]
    #[inline(always)]
    pub fn get_url_ref(&self) -> ValueResult<&Url> {
        match &self.repr {
            ValueRepr::Url(v) => Ok(v.as_ref()),
            ValueRepr::Unset(dt) if *dt == DataType::Url => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Url,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Url,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `HashMap<String, String>` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored string map.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::StringMap`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[inline(always)]
    pub fn get_string_map_ref(&self) -> ValueResult<&HashMap<String, String>> {
        match &self.repr {
            ValueRepr::StringMap(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::StringMap => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::StringMap,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::StringMap,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner JSON value without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored JSON value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::Json`, or [`ValueError::TypeMismatch`] when the stored data
    /// type differs.
    #[cfg(feature = "json")]
    #[inline(always)]
    pub fn get_json_ref(&self) -> ValueResult<&serde_json::Value> {
        match &self.repr {
            ValueRepr::Json(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::Json => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: self.data_type(),
            }),
        }
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
    /// Returns `Ok(T)` on success.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when this value is
    /// `Value::Unset(DataType::Json)`,
    /// [`ValueError::TypeMismatch`] when this value has a non-JSON data type,
    /// or [`ValueError::Conversion`] when JSON deserialization fails.
    #[cfg(all(feature = "converter", feature = "json"))]
    pub fn deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T> {
        match &self.repr {
            ValueRepr::Json(v) => Deserialize::deserialize(v).map_err(|_| {
                ValueError::from(DataConversionError::invalid(
                    DataType::Json,
                    DataType::Json,
                    InvalidValueReason::Deserialization {
                        format: DataFormat::Json,
                    },
                ))
            }),
            ValueRepr::Unset(dt) if *dt == DataType::Json => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: self.data_type(),
            }),
        }
    }
}
