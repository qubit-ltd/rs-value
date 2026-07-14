// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Closed mapping between protocol data types and value storage variants.

/// Invokes a callback with the complete value type table.
///
/// Each row contains, in order: feature attributes, `Value` field attributes,
/// `MultiValues` field attributes, enum variant, Rust storage type,
/// [`qubit_datatype::DataType`], ownership class, natural JSON class, and the
/// public variant documentation for both containers.
macro_rules! for_each_value_type {
    ($macro:ident $(, $arg:expr)*) => {
        $macro! {
            $($arg),*;
            (
                [],
                [],
                [],
                Bool,
                bool,
                ::qubit_datatype::DataType::Bool,
                copy,
                json_bool,
                "Boolean value",
                "Boolean value list"
            ),
            (
                [],
                [],
                [],
                Char,
                char,
                ::qubit_datatype::DataType::Char,
                copy,
                json_string,
                "Character value",
                "Character value list"
            ),
            (
                [],
                [],
                [],
                Int8,
                i8,
                ::qubit_datatype::DataType::Int8,
                copy,
                json_number,
                "8-bit signed integer",
                "8-bit signed integer list"
            ),
            (
                [],
                [],
                [],
                Int16,
                i16,
                ::qubit_datatype::DataType::Int16,
                copy,
                json_number,
                "16-bit signed integer",
                "16-bit signed integer list"
            ),
            (
                [],
                [],
                [],
                Int32,
                i32,
                ::qubit_datatype::DataType::Int32,
                copy,
                json_number,
                "32-bit signed integer",
                "32-bit signed integer list"
            ),
            (
                [],
                [],
                [],
                Int64,
                i64,
                ::qubit_datatype::DataType::Int64,
                copy,
                json_number,
                "64-bit signed integer",
                "64-bit signed integer list"
            ),
            (
                [],
                [serde(with = "crate::wide_integer::int128")],
                [serde(with = "crate::wide_integer::int128_vec")],
                Int128,
                i128,
                ::qubit_datatype::DataType::Int128,
                copy,
                json_string,
                "128-bit signed integer",
                "128-bit signed integer list"
            ),
            (
                [],
                [],
                [],
                UInt8,
                u8,
                ::qubit_datatype::DataType::UInt8,
                copy,
                json_number,
                "8-bit unsigned integer",
                "8-bit unsigned integer list"
            ),
            (
                [],
                [],
                [],
                UInt16,
                u16,
                ::qubit_datatype::DataType::UInt16,
                copy,
                json_number,
                "16-bit unsigned integer",
                "16-bit unsigned integer list"
            ),
            (
                [],
                [],
                [],
                UInt32,
                u32,
                ::qubit_datatype::DataType::UInt32,
                copy,
                json_number,
                "32-bit unsigned integer",
                "32-bit unsigned integer list"
            ),
            (
                [],
                [],
                [],
                UInt64,
                u64,
                ::qubit_datatype::DataType::UInt64,
                copy,
                json_number,
                "64-bit unsigned integer",
                "64-bit unsigned integer list"
            ),
            (
                [],
                [serde(with = "crate::wide_integer::uint128")],
                [serde(with = "crate::wide_integer::uint128_vec")],
                UInt128,
                u128,
                ::qubit_datatype::DataType::UInt128,
                copy,
                json_string,
                "128-bit unsigned integer",
                "128-bit unsigned integer list"
            ),
            (
                [],
                [],
                [],
                IntSize,
                isize,
                ::qubit_datatype::DataType::IntSize,
                copy,
                json_number,
                "Platform-dependent signed integer",
                "Platform-dependent signed integer list"
            ),
            (
                [],
                [],
                [],
                UIntSize,
                usize,
                ::qubit_datatype::DataType::UIntSize,
                copy,
                json_number,
                "Platform-dependent unsigned integer",
                "Platform-dependent unsigned integer list"
            ),
            (
                [],
                [serde(with = "crate::finite_float::float32")],
                [serde(with = "crate::finite_float::float32_vec")],
                Float32,
                f32,
                ::qubit_datatype::DataType::Float32,
                copy,
                json_float,
                "32-bit floating-point number",
                "32-bit floating-point number list"
            ),
            (
                [],
                [serde(with = "crate::finite_float::float64")],
                [serde(with = "crate::finite_float::float64_vec")],
                Float64,
                f64,
                ::qubit_datatype::DataType::Float64,
                copy,
                json_float,
                "64-bit floating-point number",
                "64-bit floating-point number list"
            ),
            (
                [cfg(feature = "big-number")],
                [],
                [],
                BigInteger,
                ::num_bigint::BigInt,
                ::qubit_datatype::DataType::BigInteger,
                owned,
                json_string,
                "Arbitrary-precision integer",
                "Arbitrary-precision integer list"
            ),
            (
                [cfg(feature = "big-number")],
                [],
                [],
                BigDecimal,
                ::bigdecimal::BigDecimal,
                ::qubit_datatype::DataType::BigDecimal,
                owned,
                json_string,
                "Arbitrary-precision decimal",
                "Arbitrary-precision decimal list"
            ),
            (
                [],
                [],
                [],
                String,
                String,
                ::qubit_datatype::DataType::String,
                owned,
                json_string,
                "String value",
                "String value list"
            ),
            (
                [cfg(feature = "chrono")],
                [],
                [],
                Date,
                ::chrono::NaiveDate,
                ::qubit_datatype::DataType::Date,
                copy,
                json_string,
                "Calendar date",
                "Calendar date list"
            ),
            (
                [cfg(feature = "chrono")],
                [],
                [],
                Time,
                ::chrono::NaiveTime,
                ::qubit_datatype::DataType::Time,
                copy,
                json_string,
                "Time of day",
                "Time-of-day list"
            ),
            (
                [cfg(feature = "chrono")],
                [],
                [],
                DateTime,
                ::chrono::NaiveDateTime,
                ::qubit_datatype::DataType::DateTime,
                copy,
                json_string,
                "Date and time",
                "Date-and-time list"
            ),
            (
                [cfg(feature = "chrono")],
                [],
                [],
                Instant,
                ::chrono::DateTime<::chrono::Utc>,
                ::qubit_datatype::DataType::Instant,
                copy,
                json_string,
                "UTC instant",
                "UTC instant list"
            ),
            (
                [],
                [],
                [],
                Duration,
                ::std::time::Duration,
                ::qubit_datatype::DataType::Duration,
                copy,
                json_duration,
                "Duration",
                "Duration list"
            ),
            (
                [cfg(feature = "url")],
                [],
                [],
                Url,
                ::url::Url,
                ::qubit_datatype::DataType::Url,
                owned,
                json_string,
                "URL",
                "URL list"
            ),
            (
                [],
                [],
                [],
                StringMap,
                ::std::collections::HashMap<String, String>,
                ::qubit_datatype::DataType::StringMap,
                owned,
                json_object,
                "Map with string keys and values",
                "String-map list"
            ),
            (
                [cfg(feature = "json")],
                [],
                [],
                Json,
                ::serde_json::Value,
                ::qubit_datatype::DataType::Json,
                owned,
                json_identity,
                "JSON value",
                "JSON value list"
            ),
        }
    };
}
