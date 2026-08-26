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
    Unset(DataType),
    /// A boolean value.
    Bool(bool),
    /// A character value.
    Char(char),
    /// A signed integer value.
    Int8(i8),
    /// A signed integer value.
    Int16(i16),
    /// A signed integer value.
    Int32(i32),
    /// A signed integer value.
    Int64(i64),
    /// A signed integer value.
    Int128(i128),
    /// An unsigned integer value.
    UInt8(u8),
    /// An unsigned integer value.
    UInt16(u16),
    /// An unsigned integer value.
    UInt32(u32),
    /// An unsigned integer value.
    UInt64(u64),
    /// An unsigned integer value.
    UInt128(u128),
    /// A 32-bit floating-point value.
    Float32(f32),
    /// A 64-bit floating-point value.
    Float64(f64),
    /// An arbitrary-precision integer.
    #[cfg(feature = "big-integer")]
    BigInteger(&'a num_bigint::BigInt),
    /// An arbitrary-precision decimal.
    #[cfg(feature = "big-decimal")]
    BigDecimal(&'a bigdecimal::BigDecimal),
    /// A string value.
    String(&'a str),
    /// A calendar date.
    #[cfg(feature = "chrono")]
    Date(&'a chrono::NaiveDate),
    /// A time-of-day value.
    #[cfg(feature = "chrono")]
    Time(&'a chrono::NaiveTime),
    /// A date-and-time value.
    #[cfg(feature = "chrono")]
    DateTime(&'a chrono::NaiveDateTime),
    /// A UTC instant.
    #[cfg(feature = "chrono")]
    Instant(&'a chrono::DateTime<chrono::Utc>),
    /// A duration value.
    Duration(&'a std::time::Duration),
    /// A URL value.
    #[cfg(feature = "url")]
    Url(&'a url::Url),
    /// A map with string keys and values.
    StringMap(&'a std::collections::HashMap<String, String>),
    /// A JSON value.
    #[cfg(feature = "json")]
    Json(&'a serde_json::Value),
}
