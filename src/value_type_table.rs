// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Closed mapping between protocol data types and value storage variants.

pub(crate) use std::collections::HashMap;
pub(crate) use std::time::Duration;

#[cfg(feature = "big-decimal")]
pub(crate) use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
pub(crate) use chrono::DateTime;
#[cfg(feature = "chrono")]
pub(crate) use chrono::NaiveDate;
#[cfg(feature = "chrono")]
pub(crate) use chrono::NaiveDateTime;
#[cfg(feature = "chrono")]
pub(crate) use chrono::NaiveTime;
#[cfg(feature = "chrono")]
pub(crate) use chrono::Utc;
#[cfg(feature = "big-integer")]
pub(crate) use num_bigint::BigInt;
pub(crate) use qubit_datatype::DataType;
#[cfg(feature = "json")]
pub(crate) use serde_json::Value;
#[cfg(feature = "url")]
pub(crate) use url::Url;

/// Invokes a callback with the complete value type table.
///
/// Each row contains, in order: feature attributes, enum variant, Rust storage
/// type, [`qubit_datatype::DataType`], materialization strategy, natural JSON
/// class, [`qubit_datatype::NumberRef`] projection strategy, public variant
/// documentation for both containers, scalar wire attributes, collection wire
/// attributes, and the stable V1 wire tag.
///
/// The materialization strategy is consumed by generated owned accessors and
/// scalar projection code. `copy` dereferences borrowed storage, while `clone`
/// clones it.
macro_rules! for_each_value_type {
    ($macro:ident $(, $arg:expr)*) => {
        $macro! {
            $($arg),*;
            ([], Bool, bool, $crate::value_type_table::DataType::Bool, copy, json_bool, not_number, "Boolean value", "Boolean value list", [], [], "bool"),
            ([], Char, char, $crate::value_type_table::DataType::Char, copy, json_string, not_number, "Character value", "Character value list", [], [], "char"),
            ([], Int8, i8, $crate::value_type_table::DataType::Int8, copy, json_number, number_copy, "8-bit signed integer", "8-bit signed integer list", [], [], "int8"),
            ([], Int16, i16, $crate::value_type_table::DataType::Int16, copy, json_number, number_copy, "16-bit signed integer", "16-bit signed integer list", [], [], "int16"),
            ([], Int32, i32, $crate::value_type_table::DataType::Int32, copy, json_number, number_copy, "32-bit signed integer", "32-bit signed integer list", [], [], "int32"),
            ([], Int64, i64, $crate::value_type_table::DataType::Int64, copy, json_number, number_copy, "64-bit signed integer", "64-bit signed integer list", [], [], "int64"),
            ([], Int128, i128, $crate::value_type_table::DataType::Int128, copy, json_string, number_copy, "128-bit signed integer", "128-bit signed integer list", [serde(with = "crate::wire::int128")], [serde(with = "crate::wire::int128_vec")], "int128"),
            ([], UInt8, u8, $crate::value_type_table::DataType::UInt8, copy, json_number, number_copy, "8-bit unsigned integer", "8-bit unsigned integer list", [], [], "uint8"),
            ([], UInt16, u16, $crate::value_type_table::DataType::UInt16, copy, json_number, number_copy, "16-bit unsigned integer", "16-bit unsigned integer list", [], [], "uint16"),
            ([], UInt32, u32, $crate::value_type_table::DataType::UInt32, copy, json_number, number_copy, "32-bit unsigned integer", "32-bit unsigned integer list", [], [], "uint32"),
            ([], UInt64, u64, $crate::value_type_table::DataType::UInt64, copy, json_number, number_copy, "64-bit unsigned integer", "64-bit unsigned integer list", [], [], "uint64"),
            ([], UInt128, u128, $crate::value_type_table::DataType::UInt128, copy, json_string, number_copy, "128-bit unsigned integer", "128-bit unsigned integer list", [serde(with = "crate::wire::uint128")], [serde(with = "crate::wire::uint128_vec")], "uint128"),
            ([], Float32, f32, $crate::value_type_table::DataType::Float32, copy, json_float32, number_copy, "32-bit floating-point number", "32-bit floating-point number list", [serde(with = "crate::wire::float32")], [serde(with = "crate::wire::float32_vec")], "float32"),
            ([], Float64, f64, $crate::value_type_table::DataType::Float64, copy, json_float64, number_copy, "64-bit floating-point number", "64-bit floating-point number list", [serde(with = "crate::wire::float64")], [serde(with = "crate::wire::float64_vec")], "float64"),
            ([cfg(feature = "big-integer")], BigInteger, $crate::value_type_table::BigInt, $crate::value_type_table::DataType::BigInteger, clone, json_string, number_ref, "Arbitrary-precision integer", "Arbitrary-precision integer list", [serde(with = "crate::wire::big_integer")], [serde(with = "crate::wire::big_integer_vec")], "biginteger"),
            ([cfg(feature = "big-decimal")], BigDecimal, $crate::value_type_table::BigDecimal, $crate::value_type_table::DataType::BigDecimal, clone, json_string, number_ref, "Arbitrary-precision decimal", "Arbitrary-precision decimal list", [serde(with = "crate::wire::big_decimal")], [serde(with = "crate::wire::big_decimal_vec")], "bigdecimal"),
            ([], String, String, $crate::value_type_table::DataType::String, clone, json_string, not_number, "String value", "String value list", [], [], "string"),
            ([cfg(feature = "chrono")], Date, $crate::value_type_table::NaiveDate, $crate::value_type_table::DataType::Date, copy, json_string, not_number, "Calendar date", "Calendar date list", [serde(with = "crate::wire::date")], [serde(with = "crate::wire::date_vec")], "date"),
            ([cfg(feature = "chrono")], Time, $crate::value_type_table::NaiveTime, $crate::value_type_table::DataType::Time, copy, json_string, not_number, "Time of day", "Time-of-day list", [serde(with = "crate::wire::time")], [serde(with = "crate::wire::time_vec")], "time"),
            ([cfg(feature = "chrono")], DateTime, $crate::value_type_table::NaiveDateTime, $crate::value_type_table::DataType::DateTime, copy, json_string, not_number, "Date and time", "Date-and-time list", [serde(with = "crate::wire::datetime")], [serde(with = "crate::wire::datetime_vec")], "datetime"),
            ([cfg(feature = "chrono")], Instant, $crate::value_type_table::DateTime<$crate::value_type_table::Utc>, $crate::value_type_table::DataType::Instant, copy, json_string, not_number, "UTC instant", "UTC instant list", [serde(with = "crate::wire::instant")], [serde(with = "crate::wire::instant_vec")], "instant"),
            ([], Duration, $crate::value_type_table::Duration, $crate::value_type_table::DataType::Duration, copy, json_duration, not_number, "Duration", "Duration list", [serde(with = "crate::wire::duration")], [serde(with = "crate::wire::duration_vec")], "duration"),
            ([cfg(feature = "url")], Url, $crate::value_type_table::Url, $crate::value_type_table::DataType::Url, clone, json_string, not_number, "URL", "URL list", [serde(with = "crate::wire::url")], [serde(with = "crate::wire::url_vec")], "url"),
            ([], StringMap, $crate::value_type_table::HashMap<String, String>, $crate::value_type_table::DataType::StringMap, clone, json_object, not_number, "Map with string keys and values", "String-map list", [serde(with = "crate::wire::string_map")], [serde(with = "crate::wire::string_map_vec")], "stringmap"),
            ([cfg(feature = "json")], Json, $crate::value_type_table::Value, $crate::value_type_table::DataType::Json, clone, json_identity, not_number, "JSON value", "JSON value list", [serde(with = "crate::wire::json")], [serde(with = "crate::wire::json_vec")], "json"),
        }
    };
}

/// Materializes an owned value from borrowed variant storage.
macro_rules! materialize_stored {
    (copy, $value:expr) => {
        *$value
    };
    (clone, $value:expr) => {
        $value.clone()
    };
}

/// Resolves the scalar enum storage type for a public value family.
macro_rules! value_storage_type {
    (Url, $type:ty) => {
        Box<$type>
    };
    ($variant:ident, $type:ty) => {
        $type
    };
}

/// Moves a public payload into its scalar enum storage representation.
macro_rules! value_storage_new {
    (Url, $value:expr) => {
        Box::new($value)
    };
    ($variant:ident, $value:expr) => {
        $value
    };
}

/// Borrows the public payload stored by a scalar enum variant.
macro_rules! value_storage_ref {
    (Url, $value:expr) => {
        $value.as_ref()
    };
    ($variant:ident, $value:expr) => {
        $value
    };
}

/// Materializes an owned public payload from scalar enum storage.
macro_rules! materialize_value_storage {
    (Url, $materialization:ident, $value:expr) => {
        $value.as_ref().clone()
    };
    ($variant:ident, $materialization:ident, $value:expr) => {
        materialize_stored!($materialization, $value)
    };
}

/// Moves scalar enum storage into a matching multi-value element.
macro_rules! value_storage_into_multi {
    (Url, $value:expr) => {
        *$value
    };
    ($variant:ident, $value:expr) => {
        $value
    };
}
