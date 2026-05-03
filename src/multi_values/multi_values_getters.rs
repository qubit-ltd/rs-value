/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Type-specific read accessors for `MultiValues`.

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
use url::Url;

use qubit_datatype::DataType;

use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;

impl MultiValues {
    // ========================================================================
    // Get first value (as single value access)
    // ========================================================================

    impl_get_first_value! {
        /// Get the first boolean value.
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first boolean value;
        /// otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false]);
        /// assert_eq!(values.get_first_bool().unwrap(), true);
        /// ```
        copy: get_first_bool, Bool, bool, DataType::Bool
    }

    impl_get_first_value! {
        /// Get the first character value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first character value;
        /// otherwise returns an error.
        copy: get_first_char, Char, char, DataType::Char
    }

    impl_get_first_value! {
        /// Get the first int8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int8 value;
        /// otherwise returns an error
        copy: get_first_int8, Int8, i8, DataType::Int8
    }

    impl_get_first_value! {
        /// Get the first int16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int16 value;
        /// otherwise returns an error
        copy: get_first_int16, Int16, i16, DataType::Int16
    }

    impl_get_first_value! {
        /// Get the first int32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int32 value;
        /// otherwise returns an error
        copy: get_first_int32, Int32, i32, DataType::Int32
    }

    impl_get_first_value! {
        /// Get the first int64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int64 value;
        /// otherwise returns an error
        copy: get_first_int64, Int64, i64, DataType::Int64
    }

    impl_get_first_value! {
        /// Get the first int128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int128 value;
        /// otherwise returns an error
        copy: get_first_int128, Int128, i128, DataType::Int128
    }

    impl_get_first_value! {
        /// Get the first uint8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint8 value;
        /// otherwise returns an error
        copy: get_first_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_first_value! {
        /// Get the first uint16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint16 value;
        /// otherwise returns an error
        copy: get_first_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_first_value! {
        /// Get the first uint32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint32 value;
        /// otherwise returns an error
        copy: get_first_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_first_value! {
        /// Get the first uint64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint64 value;
        /// otherwise returns an error
        copy: get_first_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_first_value! {
        /// Get the first uint128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint128 value;
        /// otherwise returns an error
        copy: get_first_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_first_value! {
        /// Get the first float32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float32 value;
        /// otherwise returns an error
        copy: get_first_float32, Float32, f32, DataType::Float32
    }

    impl_get_first_value! {
        /// Get the first float64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float64 value;
        /// otherwise returns an error
        copy: get_first_float64, Float64, f64, DataType::Float64
    }

    impl_get_first_value! {
        /// Get the first string reference
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns a reference to the first
        /// string; otherwise returns an error
        ref: get_first_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    impl_get_first_value! {
        /// Get the first date value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first date value;
        /// otherwise returns an error
        copy: get_first_date, Date, NaiveDate, DataType::Date
    }

    impl_get_first_value! {
        /// Get the first time value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first time value;
        /// otherwise returns an error
        copy: get_first_time, Time, NaiveTime, DataType::Time
    }

    impl_get_first_value! {
        /// Get the first datetime value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first datetime value;
        /// otherwise returns an error
        copy: get_first_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_first_value! {
        /// Get the first UTC instant value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first UTC instant
        /// value; otherwise returns an error
        copy: get_first_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_first_value! {
        /// Get the first big integer value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big integer
        /// value; otherwise returns an error
        ref: get_first_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    impl_get_first_value! {
        /// Get the first big decimal value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big decimal
        /// value; otherwise returns an error
        ref: get_first_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal, |v: &BigDecimal| v.clone()
    }

    impl_get_first_value! {
        /// Get the first isize value
        copy: get_first_intsize, IntSize, isize, DataType::IntSize
    }

    impl_get_first_value! {
        /// Get the first usize value
        copy: get_first_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_get_first_value! {
        /// Get the first Duration value
        copy: get_first_duration, Duration, Duration, DataType::Duration
    }

    impl_get_first_value! {
        /// Get the first Url value
        ref: get_first_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_first_value! {
        /// Get the first StringMap value
        ref: get_first_string_map, StringMap, HashMap<String, String>, DataType::StringMap, |v: &HashMap<String, String>| v.clone()
    }

    impl_get_first_value! {
        /// Get the first Json value
        ref: get_first_json, Json, serde_json::Value, DataType::Json, |v: &serde_json::Value| v.clone()
    }

    // ========================================================================
    // Get all values (type checking)
    // ========================================================================

    impl_get_multi_values! {
        /// Get reference to all boolean values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the boolean value array;
        /// otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false, true]);
        /// assert_eq!(values.get_bools().unwrap(), &[true, false, true]);
        /// ```
        slice: get_bools, Bool, bool, DataType::Bool
    }

    impl_get_multi_values! {
        /// Get reference to all character values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the character value array;
        /// otherwise returns an error
        slice: get_chars, Char, char, DataType::Char
    }

    impl_get_multi_values! {
        /// Get reference to all int8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int8 value array;
        /// otherwise returns an error
        slice: get_int8s, Int8, i8, DataType::Int8
    }

    impl_get_multi_values! {
        /// Get reference to all int16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int16 value array;
        /// otherwise returns an error
        slice: get_int16s, Int16, i16, DataType::Int16
    }

    impl_get_multi_values! {
        /// Get reference to all int32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int32 value array;
        /// otherwise returns an error
        slice: get_int32s, Int32, i32, DataType::Int32
    }

    impl_get_multi_values! {
        /// Get reference to all int64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int64 value array;
        /// otherwise returns an error
        slice: get_int64s, Int64, i64, DataType::Int64
    }

    impl_get_multi_values! {
        /// Get reference to all int128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int128 value array;
        /// otherwise returns an error
        slice: get_int128s, Int128, i128, DataType::Int128
    }

    impl_get_multi_values! {
        /// Get reference to all uint8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint8 value array;
        /// otherwise returns an error
        slice: get_uint8s, UInt8, u8, DataType::UInt8
    }

    impl_get_multi_values! {
        /// Get reference to all uint16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint16 value array;
        /// otherwise returns an error
        slice: get_uint16s, UInt16, u16, DataType::UInt16
    }

    impl_get_multi_values! {
        /// Get reference to all uint32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint32 value array;
        /// otherwise returns an error
        slice: get_uint32s, UInt32, u32, DataType::UInt32
    }

    impl_get_multi_values! {
        /// Get reference to all uint64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint64 value array;
        /// otherwise returns an error
        slice: get_uint64s, UInt64, u64, DataType::UInt64
    }

    impl_get_multi_values! {
        /// Get reference to all uint128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint128 value array;
        /// otherwise returns an error
        slice: get_uint128s, UInt128, u128, DataType::UInt128
    }

    impl_get_multi_values! {
        /// Get reference to all float32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float32 value array;
        /// otherwise returns an error
        slice: get_float32s, Float32, f32, DataType::Float32
    }

    impl_get_multi_values! {
        /// Get reference to all float64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float64 value array;
        /// otherwise returns an error
        slice: get_float64s, Float64, f64, DataType::Float64
    }

    impl_get_multi_values! {
        /// Get reference to all strings
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string array; otherwise
        /// returns an error
        vec: get_strings, String, String, DataType::String
    }

    impl_get_multi_values! {
        /// Get reference to all date values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the date value array;
        /// otherwise returns an error
        slice: get_dates, Date, NaiveDate, DataType::Date
    }

    impl_get_multi_values! {
        /// Get reference to all time values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the time value array;
        /// otherwise returns an error
        slice: get_times, Time, NaiveTime, DataType::Time
    }

    impl_get_multi_values! {
        /// Get reference to all datetime values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the datetime value array;
        /// otherwise returns an error
        slice: get_datetimes, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_multi_values! {
        /// Get reference to all UTC instant values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the UTC instant value array;
        /// otherwise returns an error
        slice: get_instants, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_multi_values! {
        /// Get reference to all big integers
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big integer array;
        /// otherwise returns an error
        vec: get_bigintegers, BigInteger, BigInt, DataType::BigInteger
    }

    impl_get_multi_values! {
        /// Get reference to all big decimals
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big decimal array;
        /// otherwise returns an error
        vec: get_bigdecimals, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_get_multi_values! {
        /// Get reference to all isize values
        slice: get_intsizes, IntSize, isize, DataType::IntSize
    }

    impl_get_multi_values! {
        /// Get reference to all usize values
        slice: get_uintsizes, UIntSize, usize, DataType::UIntSize
    }

    impl_get_multi_values! {
        /// Get reference to all Duration values
        slice: get_durations, Duration, Duration, DataType::Duration
    }

    impl_get_multi_values! {
        /// Get reference to all Url values
        vec: get_urls, Url, Url, DataType::Url
    }

    impl_get_multi_values! {
        /// Get reference to all StringMap values
        vec: get_string_maps, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_get_multi_values! {
        /// Get reference to all Json values
        vec: get_jsons, Json, serde_json::Value, DataType::Json
    }
}
