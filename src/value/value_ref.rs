// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        https://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! Borrowed semantic views for [`crate::Value`].

use qubit_datatype::DataType;

/// Borrowed semantic view of a [`crate::Value`].
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
