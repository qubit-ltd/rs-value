// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed semantic views for [`crate::MultiValues`].

use qubit_datatype::DataType;

/// Borrowed semantic view of a [`crate::MultiValues`] value.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum MultiValuesRef<'a> {
    /// An unset collection retaining its declared element type.
    Unset(DataType),
    /// A borrowed homogeneous collection.
    Bool(&'a [bool]),
    /// A borrowed homogeneous collection.
    Char(&'a [char]),
    /// A borrowed homogeneous collection.
    Int8(&'a [i8]),
    /// A borrowed homogeneous collection.
    Int16(&'a [i16]),
    /// A borrowed homogeneous collection.
    Int32(&'a [i32]),
    /// A borrowed homogeneous collection.
    Int64(&'a [i64]),
    /// A borrowed homogeneous collection.
    Int128(&'a [i128]),
    /// A borrowed homogeneous collection.
    UInt8(&'a [u8]),
    /// A borrowed homogeneous collection.
    UInt16(&'a [u16]),
    /// A borrowed homogeneous collection.
    UInt32(&'a [u32]),
    /// A borrowed homogeneous collection.
    UInt64(&'a [u64]),
    /// A borrowed homogeneous collection.
    UInt128(&'a [u128]),
    /// A borrowed homogeneous collection.
    Float32(&'a [f32]),
    /// A borrowed homogeneous collection.
    Float64(&'a [f64]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-integer")]
    BigInteger(&'a [num_bigint::BigInt]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-decimal")]
    BigDecimal(&'a [bigdecimal::BigDecimal]),
    /// A borrowed homogeneous collection.
    String(&'a [String]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Date(&'a [chrono::NaiveDate]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Time(&'a [chrono::NaiveTime]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    DateTime(&'a [chrono::NaiveDateTime]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Instant(&'a [chrono::DateTime<chrono::Utc>]),
    /// A borrowed homogeneous collection.
    Duration(&'a [std::time::Duration]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "url")]
    Url(&'a [url::Url]),
    /// A borrowed homogeneous collection.
    StringMap(&'a [std::collections::HashMap<String, String>]),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "json")]
    Json(&'a [serde_json::Value]),
}
