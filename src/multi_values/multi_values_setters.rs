/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Type-specific replacement accessors for `MultiValues`.

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

use crate::value_error::ValueResult;

use super::multi_values::MultiValues;

impl MultiValues {
    // ========================================================================
    // Set value operations
    // ========================================================================

    impl_set_multi_values! {
        /// Set all boolean values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of boolean values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Empty(DataType::Bool);
        /// values.set_bools(vec![true, false, true]).unwrap();
        /// assert_eq!(values.get_bools().unwrap(), &[true, false, true]);
        /// ```
        set_bools, Bool, bool, DataType::Bool
    }

    impl_set_multi_values! {
        /// Set all character values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of character values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_chars, Char, char, DataType::Char
    }

    impl_set_multi_values! {
        /// Set all int8 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int8 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int8s, Int8, i8, DataType::Int8
    }

    impl_set_multi_values! {
        /// Set all int16 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int16 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int16s, Int16, i16, DataType::Int16
    }

    impl_set_multi_values! {
        /// Set all int32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int32 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int32s, Int32, i32, DataType::Int32
    }

    impl_set_multi_values! {
        /// Set all int64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int64 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int64s, Int64, i64, DataType::Int64
    }

    impl_set_multi_values! {
        /// Set all int128 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of int128 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int128s, Int128, i128, DataType::Int128
    }

    impl_set_multi_values! {
        /// Set all uint8 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint8 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint8s, UInt8, u8, DataType::UInt8
    }

    impl_set_multi_values! {
        /// Set all uint16 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint16 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint16s, UInt16, u16, DataType::UInt16
    }

    impl_set_multi_values! {
        /// Set all uint32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint32 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint32s, UInt32, u32, DataType::UInt32
    }

    impl_set_multi_values! {
        /// Set all uint64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint64 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint64s, UInt64, u64, DataType::UInt64
    }

    impl_set_multi_values! {
        /// Set all uint128 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of uint128 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint128s, UInt128, u128, DataType::UInt128
    }

    impl_set_multi_values! {
        /// Set all float32 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of float32 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float32s, Float32, f32, DataType::Float32
    }

    impl_set_multi_values! {
        /// Set all float64 values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of float64 values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float64s, Float64, f64, DataType::Float64
    }

    impl_set_multi_values! {
        /// Set all string values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of string values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Empty(DataType::String);
        /// values.set_strings(vec!["hello".to_string(), "world".to_string()]).unwrap();
        /// assert_eq!(values.get_strings().unwrap(), &["hello", "world"]);
        /// ```
        set_strings, String, String, DataType::String
    }

    impl_set_multi_values! {
        /// Set all date values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of date values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_dates, Date, NaiveDate, DataType::Date
    }

    impl_set_multi_values! {
        /// Set all time values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of time values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_times, Time, NaiveTime, DataType::Time
    }

    impl_set_multi_values! {
        /// Set all datetime values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of datetime values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_datetimes, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_set_multi_values! {
        /// Set all UTC instant values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of UTC instant values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_instants, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_set_multi_values! {
        /// Set all big integer values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of big integer values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bigintegers, BigInteger, BigInt, DataType::BigInteger
    }

    impl_set_multi_values! {
        /// Set all big decimal values
        ///
        /// # Parameters
        ///
        /// * `values` - The list of big decimal values to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bigdecimals, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_set_multi_values! {
        /// Set all isize values
        set_intsizes, IntSize, isize, DataType::IntSize
    }

    impl_set_multi_values! {
        /// Set all usize values
        set_uintsizes, UIntSize, usize, DataType::UIntSize
    }

    impl_set_multi_values! {
        /// Set all Duration values
        set_durations, Duration, Duration, DataType::Duration
    }

    impl_set_multi_values! {
        /// Set all Url values
        set_urls, Url, Url, DataType::Url
    }

    impl_set_multi_values! {
        /// Set all StringMap values
        set_string_maps, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_set_multi_values! {
        /// Set all Json values
        set_jsons, Json, serde_json::Value, DataType::Json
    }

    // ========================================================================
    // Set all values via slice operations
    // ========================================================================

    impl_set_multi_values_slice! {
        /// Set all boolean values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The boolean value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bools_slice, Bool, bool, DataType::Bool
    }

    impl_set_multi_values_slice! {
        /// Set all character values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The character value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_chars_slice, Char, char, DataType::Char
    }

    impl_set_multi_values_slice! {
        /// Set all int8 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int8 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int8s_slice, Int8, i8, DataType::Int8
    }

    impl_set_multi_values_slice! {
        /// Set all int16 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int16 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int16s_slice, Int16, i16, DataType::Int16
    }

    impl_set_multi_values_slice! {
        /// Set all int32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int32 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int32s_slice, Int32, i32, DataType::Int32
    }

    impl_set_multi_values_slice! {
        /// Set all int64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int64 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int64s_slice, Int64, i64, DataType::Int64
    }

    impl_set_multi_values_slice! {
        /// Set all int128 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The int128 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int128s_slice, Int128, i128, DataType::Int128
    }

    impl_set_multi_values_slice! {
        /// Set all uint8 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint8 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint8s_slice, UInt8, u8, DataType::UInt8
    }

    impl_set_multi_values_slice! {
        /// Set all uint16 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint16 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint16s_slice, UInt16, u16, DataType::UInt16
    }

    impl_set_multi_values_slice! {
        /// Set all uint32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint32 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint32s_slice, UInt32, u32, DataType::UInt32
    }

    impl_set_multi_values_slice! {
        /// Set all uint64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint64 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint64s_slice, UInt64, u64, DataType::UInt64
    }

    impl_set_multi_values_slice! {
        /// Set all uint128 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The uint128 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint128s_slice, UInt128, u128, DataType::UInt128
    }

    impl_set_multi_values_slice! {
        /// Set all float32 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The float32 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float32s_slice, Float32, f32, DataType::Float32
    }

    impl_set_multi_values_slice! {
        /// Set all float64 values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The float64 value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float64s_slice, Float64, f64, DataType::Float64
    }

    impl_set_multi_values_slice! {
        /// Set all string values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The string value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_strings_slice, String, String, DataType::String
    }

    impl_set_multi_values_slice! {
        /// Set all date values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The date value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_dates_slice, Date, NaiveDate, DataType::Date
    }

    impl_set_multi_values_slice! {
        /// Set all time values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The time value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_times_slice, Time, NaiveTime, DataType::Time
    }

    impl_set_multi_values_slice! {
        /// Set all datetime values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The datetime value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_datetimes_slice, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_set_multi_values_slice! {
        /// Set all UTC instant values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The UTC instant value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_instants_slice, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_set_multi_values_slice! {
        /// Set all big integer values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The big integer value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bigintegers_slice, BigInteger, BigInt, DataType::BigInteger
    }

    impl_set_multi_values_slice! {
        /// Set all big decimal values via slice
        ///
        /// # Parameters
        ///
        /// * `values` - The big decimal value slice to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bigdecimals_slice, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_set_multi_values_slice! {
        /// Set all isize values via slice
        set_intsizes_slice, IntSize, isize, DataType::IntSize
    }

    impl_set_multi_values_slice! {
        /// Set all usize values via slice
        set_uintsizes_slice, UIntSize, usize, DataType::UIntSize
    }

    impl_set_multi_values_slice! {
        /// Set all Duration values via slice
        set_durations_slice, Duration, Duration, DataType::Duration
    }

    impl_set_multi_values_slice! {
        /// Set all Url values via slice
        set_urls_slice, Url, Url, DataType::Url
    }

    impl_set_multi_values_slice! {
        /// Set all StringMap values via slice
        set_string_maps_slice, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_set_multi_values_slice! {
        /// Set all Json values via slice
        set_jsons_slice, Json, serde_json::Value, DataType::Json
    }

    // ========================================================================
    // Set single value operations
    // ========================================================================

    impl_set_single_value! {
        /// Set single boolean value
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
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Empty(DataType::Bool);
        /// values.set_bool(true).unwrap();
        /// assert_eq!(values.get_bools().unwrap(), &[true]);
        /// ```
        set_bool, Bool, bool, DataType::Bool
    }

    impl_set_single_value! {
        /// Set single character value
        ///
        /// # Parameters
        ///
        /// * `value` - The character value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_char, Char, char, DataType::Char
    }

    impl_set_single_value! {
        /// Set single int8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int8 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int8, Int8, i8, DataType::Int8
    }

    impl_set_single_value! {
        /// Set single int16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int16 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int16, Int16, i16, DataType::Int16
    }

    impl_set_single_value! {
        /// Set single int32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int32, Int32, i32, DataType::Int32
    }

    impl_set_single_value! {
        /// Set single int64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int64, Int64, i64, DataType::Int64
    }

    impl_set_single_value! {
        /// Set single int128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The int128 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_int128, Int128, i128, DataType::Int128
    }

    impl_set_single_value! {
        /// Set single uint8 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint8 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint8, UInt8, u8, DataType::UInt8
    }

    impl_set_single_value! {
        /// Set single uint16 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint16 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint16, UInt16, u16, DataType::UInt16
    }

    impl_set_single_value! {
        /// Set single uint32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint32, UInt32, u32, DataType::UInt32
    }

    impl_set_single_value! {
        /// Set single uint64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint64, UInt64, u64, DataType::UInt64
    }

    impl_set_single_value! {
        /// Set single uint128 value
        ///
        /// # Parameters
        ///
        /// * `value` - The uint128 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_uint128, UInt128, u128, DataType::UInt128
    }

    impl_set_single_value! {
        /// Set single float32 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float32 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float32, Float32, f32, DataType::Float32
    }

    impl_set_single_value! {
        /// Set single float64 value
        ///
        /// # Parameters
        ///
        /// * `value` - The float64 value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_float64, Float64, f64, DataType::Float64
    }

    impl_set_single_value! {
        /// Set single string value
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
        /// ```rust
        /// use qubit_datatype::DataType;
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Empty(DataType::String);
        /// values.set_string("hello".to_string()).unwrap();
        /// assert_eq!(values.get_strings().unwrap(), &["hello"]);
        /// ```
        set_string, String, String, DataType::String
    }

    impl_set_single_value! {
        /// Set single date value
        ///
        /// # Parameters
        ///
        /// * `value` - The date value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_date, Date, NaiveDate, DataType::Date
    }

    impl_set_single_value! {
        /// Set single time value
        ///
        /// # Parameters
        ///
        /// * `value` - The time value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_time, Time, NaiveTime, DataType::Time
    }

    impl_set_single_value! {
        /// Set single datetime value
        ///
        /// # Parameters
        ///
        /// * `value` - The datetime value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_set_single_value! {
        /// Set single UTC instant value
        ///
        /// # Parameters
        ///
        /// * `value` - The UTC instant value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_set_single_value! {
        /// Set single big integer value
        ///
        /// # Parameters
        ///
        /// * `value` - The big integer value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_biginteger, BigInteger, BigInt, DataType::BigInteger
    }

    impl_set_single_value! {
        /// Set single big decimal value
        ///
        /// # Parameters
        ///
        /// * `value` - The big decimal value to set
        ///
        /// # Returns
        ///
        /// If setting succeeds, returns `Ok(())`; otherwise returns an error
        set_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_set_single_value! {
        /// Set single isize value
        set_intsize, IntSize, isize, DataType::IntSize
    }

    impl_set_single_value! {
        /// Set single usize value
        set_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_set_single_value! {
        /// Set single Duration value
        set_duration, Duration, Duration, DataType::Duration
    }

    impl_set_single_value! {
        /// Set single Url value
        set_url, Url, Url, DataType::Url
    }

    impl_set_single_value! {
        /// Set single StringMap value
        set_string_map, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_set_single_value! {
        /// Set single Json value
        set_json, Json, serde_json::Value, DataType::Json
    }
}
