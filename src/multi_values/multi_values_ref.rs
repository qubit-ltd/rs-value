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
///
/// # Type Parameters
///
/// * `'a` - Lifetime of element slices borrowed from the source collection.
///
/// # Examples
///
/// ```
/// use qubit_value::{MultiValues, MultiValuesRef};
///
/// let values = MultiValues::from(vec![1_i32, 2]);
/// assert!(matches!(values.view(), MultiValuesRef::Int32(items) if items == [1, 2]));
/// ```
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum MultiValuesRef<'a> {
    /// An unset collection retaining its declared element type.
    Unset(
        /// Declared element type retained while no collection is stored.
        DataType,
    ),
    /// A borrowed homogeneous collection.
    Bool(
        /// Borrowed boolean elements.
        &'a [bool],
    ),
    /// A borrowed homogeneous collection.
    Char(
        /// Borrowed character elements.
        &'a [char],
    ),
    /// A borrowed homogeneous collection.
    Int8(
        /// Borrowed signed integer elements.
        &'a [i8],
    ),
    /// A borrowed homogeneous collection.
    Int16(
        /// Borrowed signed integer elements.
        &'a [i16],
    ),
    /// A borrowed homogeneous collection.
    Int32(
        /// Borrowed signed integer elements.
        &'a [i32],
    ),
    /// A borrowed homogeneous collection.
    Int64(
        /// Borrowed signed integer elements.
        &'a [i64],
    ),
    /// A borrowed homogeneous collection.
    Int128(
        /// Borrowed signed integer elements.
        &'a [i128],
    ),
    /// A borrowed homogeneous collection.
    UInt8(
        /// Borrowed unsigned integer elements.
        &'a [u8],
    ),
    /// A borrowed homogeneous collection.
    UInt16(
        /// Borrowed unsigned integer elements.
        &'a [u16],
    ),
    /// A borrowed homogeneous collection.
    UInt32(
        /// Borrowed unsigned integer elements.
        &'a [u32],
    ),
    /// A borrowed homogeneous collection.
    UInt64(
        /// Borrowed unsigned integer elements.
        &'a [u64],
    ),
    /// A borrowed homogeneous collection.
    UInt128(
        /// Borrowed unsigned integer elements.
        &'a [u128],
    ),
    /// A borrowed homogeneous collection.
    Float32(
        /// Borrowed floating-point elements.
        &'a [f32],
    ),
    /// A borrowed homogeneous collection.
    Float64(
        /// Borrowed floating-point elements.
        &'a [f64],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-integer")]
    BigInteger(
        /// Borrowed arbitrary-precision integer elements.
        &'a [num_bigint::BigInt],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "big-decimal")]
    BigDecimal(
        /// Borrowed arbitrary-precision decimal elements.
        &'a [bigdecimal::BigDecimal],
    ),
    /// A borrowed homogeneous collection.
    String(
        /// Borrowed UTF-8 string elements.
        &'a [String],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Date(
        /// Borrowed calendar-date elements.
        &'a [chrono::NaiveDate],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Time(
        /// Borrowed time-of-day elements.
        &'a [chrono::NaiveTime],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    DateTime(
        /// Borrowed local date-and-time elements.
        &'a [chrono::NaiveDateTime],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "chrono")]
    Instant(
        /// Borrowed UTC instant elements.
        &'a [chrono::DateTime<chrono::Utc>],
    ),
    /// A borrowed homogeneous collection.
    Duration(
        /// Borrowed non-negative duration elements.
        &'a [std::time::Duration],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "url")]
    Url(
        /// Borrowed parsed URL elements.
        &'a [url::Url],
    ),
    /// A borrowed homogeneous collection.
    StringMap(
        /// Borrowed string-keyed map elements.
        &'a [std::collections::HashMap<String, String>],
    ),
    /// A borrowed homogeneous collection.
    #[cfg(feature = "json")]
    Json(
        /// Borrowed JSON tree elements.
        &'a [serde_json::Value],
    ),
}
