/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Multiple Values Container
//!
//! Provides type-safe storage and access functionality for multiple values.
//!

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use num_bigint::BigInt;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use qubit_datatype::DataType;

/// Multiple values container
///
/// Uses an enum to represent multiple values of different types, providing
/// type-safe storage and access for multiple values.
///
/// # Features
///
/// - Supports collections of multiple basic data types.
/// - Provides two sets of APIs for type checking and type conversion.
/// - Supports unified access to single and multiple values.
/// - Automatic memory management.
///
/// # Example
///
/// ```rust
/// use qubit_value::MultiValues;
///
/// // Create integer multiple values
/// let mut values = MultiValues::Int32(vec![1, 2, 3]);
/// assert_eq!(values.count(), 3);
/// assert_eq!(values.get_first_int32().unwrap(), 1);
///
/// // Get all values
/// let all = values.get_int32s().unwrap();
/// assert_eq!(all, &[1, 2, 3]);
///
/// // Use generic method to add value
/// values.add(4).unwrap();
/// assert_eq!(values.count(), 4);
/// ```
///
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultiValues {
    /// Empty value (has type but no values)
    Empty(DataType),
    /// Boolean value list
    Bool(Vec<bool>),
    /// Character value list
    Char(Vec<char>),
    /// i8 list
    Int8(Vec<i8>),
    /// i16 list
    Int16(Vec<i16>),
    /// i32 list
    Int32(Vec<i32>),
    /// i64 list
    Int64(Vec<i64>),
    /// i128 list
    Int128(Vec<i128>),
    /// u8 list
    UInt8(Vec<u8>),
    /// u16 list
    UInt16(Vec<u16>),
    /// u32 list
    UInt32(Vec<u32>),
    /// u64 list
    UInt64(Vec<u64>),
    /// u128 list
    UInt128(Vec<u128>),
    /// isize list
    IntSize(Vec<isize>),
    /// usize list
    UIntSize(Vec<usize>),
    /// f32 list
    Float32(Vec<f32>),
    /// f64 list
    Float64(Vec<f64>),
    /// Big integer list
    BigInteger(Vec<BigInt>),
    /// Big decimal list
    BigDecimal(Vec<BigDecimal>),
    /// String list
    String(Vec<String>),
    /// Date list
    Date(Vec<NaiveDate>),
    /// Time list
    Time(Vec<NaiveTime>),
    /// DateTime list
    DateTime(Vec<NaiveDateTime>),
    /// UTC instant list
    Instant(Vec<DateTime<Utc>>),
    /// Duration list
    Duration(Vec<Duration>),
    /// Url list
    Url(Vec<Url>),
    /// StringMap list
    StringMap(Vec<HashMap<String, String>>),
    /// Json list
    Json(Vec<serde_json::Value>),
}

// ============================================================================
// Getter method generation macros
// ============================================================================

/// Unified multiple values getter generation macro
///
/// Generates `get_[xxx]s` methods for `MultiValues`, returning a reference to
/// value slices.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_get_multi_values {
    // Simple type: return slice reference
    ($(#[$attr:meta])* slice: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match self {
                MultiValues::$variant(v) => Ok(v),
                MultiValues::Empty(dt) if *dt == $data_type => Ok(&[]),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Complex type: return Vec reference (e.g., Vec<String>, Vec<Vec<u8>>)
    ($(#[$attr:meta])* vec: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match self {
                MultiValues::$variant(v) => Ok(v.as_slice()),
                MultiValues::Empty(dt) if *dt == $data_type => Ok(&[]),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values get_first method generation macro
///
/// Generates `get_first_[xxx]` methods for `MultiValues`, used to get the first
/// value.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_get_first_value {
    // Copy type: directly return value
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&self) -> ValueResult<$type> {
            match self {
                MultiValues::$variant(v) if !v.is_empty() => Ok(v[0]),
                MultiValues::$variant(_) => Err(ValueError::NoValue),
                MultiValues::Empty(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Reference type: return reference
    ($(#[$attr:meta])* ref: $method:ident, $variant:ident, $ret_type:ty, $data_type:expr, $conversion:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match self {
                MultiValues::$variant(v) if !v.is_empty() => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(&v[0]))
                },
                MultiValues::$variant(_) => Err(ValueError::NoValue),
                MultiValues::Empty(dt) if *dt == $data_type => Err(ValueError::NoValue),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values add method generation macro
///
/// Generates `add_[xxx]` methods for `MultiValues`, used to add a single value.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_add_single_value {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            match self {
                MultiValues::$variant(v) => {
                    v.push(value);
                    Ok(())
                }
                MultiValues::Empty(dt) if *dt == $data_type => {
                    *self = MultiValues::$variant(vec![value]);
                    Ok(())
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values add multiple method generation macro
///
/// Generates `add_[xxx]s` methods for `MultiValues`, used to add multiple values.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_add_multi_values {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&mut self, values: Vec<$type>) -> ValueResult<()> {
            match self {
                MultiValues::$variant(v) => {
                    v.extend(values);
                    Ok(())
                }
                MultiValues::Empty(dt) if *dt == $data_type => {
                    *self = MultiValues::$variant(values);
                    Ok(())
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values add from slice method generation macro
///
/// Generates `add_[xxx]s_slice` methods for `MultiValues`, used to append
/// multiple values at once from a slice.
///
///
macro_rules! impl_add_multi_values_slice {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[inline]
        pub fn $method(&mut self, values: &[$type]) -> ValueResult<()> {
            match self {
                MultiValues::$variant(v) => {
                    v.extend_from_slice(values);
                    Ok(())
                }
                MultiValues::Empty(dt) if *dt == $data_type => {
                    *self = MultiValues::$variant(values.to_vec());
                    Ok(())
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values single value set method generation macro
///
/// Generates `set_[xxx]` methods for `MultiValues`, used to set a single value
/// (replacing the entire list).
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_set_single_value {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&mut self, value: $type) -> ValueResult<()> {
            *self = MultiValues::$variant(vec![value]);
            Ok(())
        }
    };
}

/// Unified multiple values set method generation macro
///
/// Generates `set_[xxx]s` methods for `MultiValues`, used to set the entire
/// value list.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_set_multi_values {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&mut self, values: Vec<$type>) -> ValueResult<()> {
            *self = MultiValues::$variant(values);
            Ok(())
        }
    };
}

/// Unified multiple values set (slice) method generation macro
///
/// Generates `set_[xxx]s_slice` methods for `MultiValues`, used to set the
/// entire value list from a slice.
///
/// This method directly replaces the internally stored list without type
/// matching checks, behaving consistently with `set_[xxx]s`.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
///
///
macro_rules! impl_set_multi_values_slice {
    ($(#[$attr:meta])* $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        pub fn $method(&mut self, values: &[$type]) -> ValueResult<()> {
            *self = MultiValues::$variant(values.to_vec());
            Ok(())
        }
    };
}
