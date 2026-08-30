// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed semantic views for [`crate::Value`].

use qubit_datatype::DataType;

/// Borrowed semantic view of a [`crate::Value`].
///
/// # Type Parameters
///
/// * `'a` - Lifetime of payloads borrowed from the source value.
///
/// # Examples
///
/// ```
/// use qubit_value::{Value, ValueRef};
///
/// let value = Value::from(42_i32);
/// assert!(matches!(value.view(), ValueRef::Int32(42)));
/// ```
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ValueRef<'a> {
    /// An unset value retaining its declared type.
    Unset(
        /// Declared type retained while no concrete value is stored.
        DataType,
    ),
    /// A boolean value.
    Bool(
        /// Copied boolean payload.
        bool,
    ),
    /// A character value.
    Char(
        /// Copied character payload.
        char,
    ),
    /// A signed integer value.
    Int8(
        /// Copied signed integer payload.
        i8,
    ),
    /// A signed integer value.
    Int16(
        /// Copied signed integer payload.
        i16,
    ),
    /// A signed integer value.
    Int32(
        /// Copied signed integer payload.
        i32,
    ),
    /// A signed integer value.
    Int64(
        /// Copied signed integer payload.
        i64,
    ),
    /// A signed integer value.
    Int128(
        /// Copied signed integer payload.
        i128,
    ),
    /// An unsigned integer value.
    UInt8(
        /// Copied unsigned integer payload.
        u8,
    ),
    /// An unsigned integer value.
    UInt16(
        /// Copied unsigned integer payload.
        u16,
    ),
    /// An unsigned integer value.
    UInt32(
        /// Copied unsigned integer payload.
        u32,
    ),
    /// An unsigned integer value.
    UInt64(
        /// Copied unsigned integer payload.
        u64,
    ),
    /// An unsigned integer value.
    UInt128(
        /// Copied unsigned integer payload.
        u128,
    ),
    /// A 32-bit floating-point value.
    Float32(
        /// Copied floating-point payload.
        f32,
    ),
    /// A 64-bit floating-point value.
    Float64(
        /// Copied floating-point payload.
        f64,
    ),
    /// An arbitrary-precision integer.
    #[cfg(feature = "big-integer")]
    BigInteger(
        /// Borrowed arbitrary-precision integer payload.
        &'a num_bigint::BigInt,
    ),
    /// An arbitrary-precision decimal.
    #[cfg(feature = "big-decimal")]
    BigDecimal(
        /// Borrowed arbitrary-precision decimal payload.
        &'a bigdecimal::BigDecimal,
    ),
    /// A string value.
    String(
        /// Borrowed UTF-8 string payload.
        &'a str,
    ),
    /// A calendar date.
    #[cfg(feature = "chrono")]
    Date(
        /// Borrowed calendar-date payload.
        &'a chrono::NaiveDate,
    ),
    /// A time-of-day value.
    #[cfg(feature = "chrono")]
    Time(
        /// Borrowed time-of-day payload.
        &'a chrono::NaiveTime,
    ),
    /// A date-and-time value.
    #[cfg(feature = "chrono")]
    DateTime(
        /// Borrowed local date-and-time payload.
        &'a chrono::NaiveDateTime,
    ),
    /// A UTC instant.
    #[cfg(feature = "chrono")]
    Instant(
        /// Borrowed UTC instant payload.
        &'a chrono::DateTime<chrono::Utc>,
    ),
    /// A duration value.
    Duration(
        /// Borrowed non-negative duration payload.
        &'a std::time::Duration,
    ),
    /// A URL value.
    #[cfg(feature = "url")]
    Url(
        /// Borrowed parsed URL payload.
        &'a url::Url,
    ),
    /// A map with string keys and values.
    StringMap(
        /// Borrowed string-keyed map payload.
        &'a std::collections::HashMap<String, String>,
    ),
    /// A JSON value.
    #[cfg(feature = "json")]
    Json(
        /// Borrowed JSON tree payload.
        &'a serde_json::Value,
    ),
}
