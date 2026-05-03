/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Type-specific append accessors for `MultiValues`.

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
    // Add value operations
    // ========================================================================

    impl_add_single_value! {
        /// Add a boolean value
        ///
        /// # Parameters
        ///
        /// * `value` - The boolean value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Bool(vec![true]);
        /// values.add_bool(false).unwrap();
        /// assert_eq!(values.count(), 2);
        /// ```
        add_bool, Bool, bool, DataType::Bool
    }

    impl_add_single_value! {
        /// Add a character value
        ///
        /// # Parameters
        ///
        /// * `value` - The character value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_char, Char, char, DataType::Char
    }

    impl_add_single_value! {
        /// Add an int8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int8 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int8, Int8, i8, DataType::Int8
    }

    impl_add_single_value! {
        /// Add an int16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int16 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int16, Int16, i16, DataType::Int16
    }

    impl_add_single_value! {
        /// Add an int32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int32 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int32, Int32, i32, DataType::Int32
    }

    impl_add_single_value! {
        /// Add an int64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int64 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int64, Int64, i64, DataType::Int64
    }

    impl_add_single_value! {
        /// Add an int128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int128 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int128, Int128, i128, DataType::Int128
    }

    impl_add_single_value! {
        /// Add a uint8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint8 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint8, UInt8, u8, DataType::UInt8
    }

    impl_add_single_value! {
        /// Add a uint16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint16 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint16, UInt16, u16, DataType::UInt16
    }

    impl_add_single_value! {
        /// Add a uint32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint32 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint32, UInt32, u32, DataType::UInt32
    }

    impl_add_single_value! {
        /// Add a uint64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint64 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint64, UInt64, u64, DataType::UInt64
    }

    impl_add_single_value! {
        /// Add a uint128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint128 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint128, UInt128, u128, DataType::UInt128
    }

    impl_add_single_value! {
        /// Add a float32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float32 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float32, Float32, f32, DataType::Float32
    }

    impl_add_single_value! {
        /// Add a float64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float64 value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float64, Float64, f64, DataType::Float64
    }

    impl_add_single_value! {
        /// Add a string
        ///
        /// # Parameters
        ///
        /// * `value` - The string to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_string, String, String, DataType::String
    }

    impl_add_single_value! {
        /// Add a date value
        ///
        /// # Parameters
        ///
        /// * `value` - The date value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_date, Date, NaiveDate, DataType::Date
    }

    impl_add_single_value! {
        /// Add a time value
        ///
        /// # Parameters
        ///
        /// * `value` - The time value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_time, Time, NaiveTime, DataType::Time
    }

    impl_add_single_value! {
        /// Add a datetime value
        ///
        /// # Parameters
        ///
        /// * `value` - The datetime value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_add_single_value! {
        /// Add a UTC instant value
        ///
        /// # Parameters
        ///
        /// * `value` - The UTC instant value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_add_single_value! {
        /// Add a big integer value
        ///
        /// # Parameters
        ///
        /// * `value` - The big integer value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_biginteger, BigInteger, BigInt, DataType::BigInteger
    }

    impl_add_single_value! {
        /// Add a big decimal value
        ///
        /// # Parameters
        ///
        /// * `value` - The big decimal value to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_add_single_value! {
        /// Add an isize value
        add_intsize, IntSize, isize, DataType::IntSize
    }

    impl_add_single_value! {
        /// Add a usize value
        add_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_add_single_value! {
        /// Add a Duration value
        add_duration, Duration, Duration, DataType::Duration
    }

    impl_add_single_value! {
        /// Add a Url value
        add_url, Url, Url, DataType::Url
    }

    impl_add_single_value! {
        /// Add a StringMap value
        add_string_map, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_add_single_value! {
        /// Add a Json value
        add_json, Json, serde_json::Value, DataType::Json
    }

    // ========================================================================
    // Add multiple values operations
    // ========================================================================

    impl_add_multi_values! {
        /// Add multiple boolean values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of boolean values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Bool(vec![true]);
        /// values.add_bools(vec![false, true]).unwrap();
        /// assert_eq!(values.get_bools().unwrap(), &[true, false, true]);
        /// ```
        add_bools, Bool, bool, DataType::Bool
    }

    impl_add_multi_values! {
        /// Add multiple character values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of character values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_chars, Char, char, DataType::Char
    }

    impl_add_multi_values! {
        /// Add multiple int8 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int8 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int8s, Int8, i8, DataType::Int8
    }

    impl_add_multi_values! {
        /// Add multiple int16 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int16 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int16s, Int16, i16, DataType::Int16
    }

    impl_add_multi_values! {
        /// Add multiple int32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int32 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int32s, Int32, i32, DataType::Int32
    }

    impl_add_multi_values! {
        /// Add multiple int64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int64 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int64s, Int64, i64, DataType::Int64
    }

    impl_add_multi_values! {
        /// Add multiple int128 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int128 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int128s, Int128, i128, DataType::Int128
    }

    impl_add_multi_values! {
        /// Add multiple uint8 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint8 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint8s, UInt8, u8, DataType::UInt8
    }

    impl_add_multi_values! {
        /// Add multiple uint16 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint16 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint16s, UInt16, u16, DataType::UInt16
    }

    impl_add_multi_values! {
        /// Add multiple uint32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint32 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint32s, UInt32, u32, DataType::UInt32
    }

    impl_add_multi_values! {
        /// Add multiple uint64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint64 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint64s, UInt64, u64, DataType::UInt64
    }

    impl_add_multi_values! {
        /// Add multiple uint128 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint128 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint128s, UInt128, u128, DataType::UInt128
    }

    impl_add_multi_values! {
        /// Add multiple float32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of float32 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float32s, Float32, f32, DataType::Float32
    }

    impl_add_multi_values! {
        /// Add multiple float64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of float64 values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float64s, Float64, f64, DataType::Float64
    }

    impl_add_multi_values! {
        /// Add multiple string values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of string values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::String(vec!["hello".to_string()]);
        /// values.add_strings(vec!["world".to_string(), "rust".to_string()]).unwrap();
        /// assert_eq!(values.get_strings().unwrap(), &["hello", "world", "rust"]);
        /// ```
        add_strings, String, String, DataType::String
    }

    impl_add_multi_values! {
        /// Add multiple date values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of date values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_dates, Date, NaiveDate, DataType::Date
    }

    impl_add_multi_values! {
        /// Add multiple time values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of time values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_times, Time, NaiveTime, DataType::Time
    }

    impl_add_multi_values! {
        /// Add multiple datetime values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of datetime values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_datetimes, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_add_multi_values! {
        /// Add multiple UTC instant values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of UTC instant values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_instants, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_add_multi_values! {
        /// Add multiple big integer values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of big integer values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bigintegers, BigInteger, BigInt, DataType::BigInteger
    }

    impl_add_multi_values! {
        /// Add multiple big decimal values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of big decimal values to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bigdecimals, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_add_multi_values! {
        /// Add multiple isize values
        add_intsizes, IntSize, isize, DataType::IntSize
    }

    impl_add_multi_values! {
        /// Add multiple usize values
        add_uintsizes, UIntSize, usize, DataType::UIntSize
    }

    impl_add_multi_values! {
        /// Add multiple Duration values
        add_durations, Duration, Duration, DataType::Duration
    }

    impl_add_multi_values! {
        /// Add multiple Url values
        add_urls, Url, Url, DataType::Url
    }

    impl_add_multi_values! {
        /// Add multiple StringMap values
        add_string_maps, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_add_multi_values! {
        /// Add multiple Json values
        add_jsons, Json, serde_json::Value, DataType::Json
    }

    // ========================================================================
    // Add multiple values via slice operations
    // ========================================================================

    impl_add_multi_values_slice! {
        /// Add multiple boolean values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The boolean value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bools_slice, Bool, bool, DataType::Bool
    }

    impl_add_multi_values_slice! {
        /// Add multiple character values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The character value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_chars_slice, Char, char, DataType::Char
    }

    impl_add_multi_values_slice! {
        /// Add multiple int8 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int8 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int8s_slice, Int8, i8, DataType::Int8
    }

    impl_add_multi_values_slice! {
        /// Add multiple int16 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int16 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int16s_slice, Int16, i16, DataType::Int16
    }

    impl_add_multi_values_slice! {
        /// Add multiple int32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int32 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int32s_slice, Int32, i32, DataType::Int32
    }

    impl_add_multi_values_slice! {
        /// Add multiple int64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int64 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int64s_slice, Int64, i64, DataType::Int64
    }

    impl_add_multi_values_slice! {
        /// Add multiple int128 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int128 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_int128s_slice, Int128, i128, DataType::Int128
    }

    impl_add_multi_values_slice! {
        /// Add multiple uint8 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint8 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint8s_slice, UInt8, u8, DataType::UInt8
    }

    impl_add_multi_values_slice! {
        /// Add multiple uint16 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint16 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint16s_slice, UInt16, u16, DataType::UInt16
    }

    impl_add_multi_values_slice! {
        /// Add multiple uint32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint32 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint32s_slice, UInt32, u32, DataType::UInt32
    }

    impl_add_multi_values_slice! {
        /// Add multiple uint64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint64 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint64s_slice, UInt64, u64, DataType::UInt64
    }

    impl_add_multi_values_slice! {
        /// Add multiple uint128 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint128 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_uint128s_slice, UInt128, u128, DataType::UInt128
    }

    impl_add_multi_values_slice! {
        /// Add multiple float32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The float32 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float32s_slice, Float32, f32, DataType::Float32
    }

    impl_add_multi_values_slice! {
        /// Add multiple float64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The float64 value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_float64s_slice, Float64, f64, DataType::Float64
    }

    impl_add_multi_values_slice! {
        /// Add multiple strings via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The string slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_strings_slice, String, String, DataType::String
    }

    impl_add_multi_values_slice! {
        /// Add multiple date values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The date value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_dates_slice, Date, NaiveDate, DataType::Date
    }

    impl_add_multi_values_slice! {
        /// Add multiple time values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The time value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_times_slice, Time, NaiveTime, DataType::Time
    }

    impl_add_multi_values_slice! {
        /// Add multiple datetime values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The datetime value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_datetimes_slice, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_add_multi_values_slice! {
        /// Add multiple UTC instant values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The UTC instant value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_instants_slice, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_add_multi_values_slice! {
        /// Add multiple big integer values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The big integer value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bigintegers_slice, BigInteger, BigInt, DataType::BigInteger
    }

    impl_add_multi_values_slice! {
        /// Add multiple big decimal values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The big decimal value slice to add
        ///
        /// # Returns
        ///
        /// If types match, returns `Ok(())`; otherwise returns an error
        add_bigdecimals_slice, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_add_multi_values_slice! {
        /// Add multiple isize values via slice
        add_intsizes_slice, IntSize, isize, DataType::IntSize
    }

    impl_add_multi_values_slice! {
        /// Add multiple usize values via slice
        add_uintsizes_slice, UIntSize, usize, DataType::UIntSize
    }

    impl_add_multi_values_slice! {
        /// Add multiple Duration values via slice
        add_durations_slice, Duration, Duration, DataType::Duration
    }

    impl_add_multi_values_slice! {
        /// Add multiple Url values via slice
        add_urls_slice, Url, Url, DataType::Url
    }

    impl_add_multi_values_slice! {
        /// Add multiple StringMap values via slice
        add_string_maps_slice, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_add_multi_values_slice! {
        /// Add multiple Json values via slice
        add_jsons_slice, Json, serde_json::Value, DataType::Json
    }
}
