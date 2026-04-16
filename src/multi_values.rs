/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Multiple Values Container
//!
//! Provides type-safe storage and access functionality for multiple values.
//!
//! # Author
//!
//! Haixing Hu

#![allow(private_bounds)]

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use qubit_common::lang::DataType;

use super::error::{ValueError, ValueResult};
use super::value::Value;

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
/// ```rust,ignore
/// use common_rs::util::value::MultiValues;
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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
/// # Author
///
/// Haixing Hu
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

impl MultiValues {
    /// Generic constructor method
    ///
    /// Creates `MultiValues` from `Vec<T>`, avoiding direct use of enum
    /// variants.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Element type
    ///
    /// # Returns
    ///
    /// Returns `MultiValues` wrapping the given value list
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::MultiValues;
    ///
    /// // Basic types
    /// let mv = MultiValues::new(vec![1, 2, 3]);
    /// assert_eq!(mv.count(), 3);
    ///
    /// // Strings
    /// let mv = MultiValues::new(vec!["a".to_string(), "b".to_string()]);
    /// assert_eq!(mv.count(), 2);
    /// ```
    #[inline]
    pub fn new<T>(values: Vec<T>) -> Self
    where
        Self: MultiValuesConstructor<T>,
    {
        <Self as MultiValuesConstructor<T>>::from_vec(values)
    }

    /// Generic getter method for multiple values
    ///
    /// Automatically selects the correct getter method based on the target
    /// type, performing strict type checking.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target element type to retrieve.
    ///
    /// # Returns
    ///
    /// If types match, returns the list of values; otherwise returns an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::MultiValues;
    ///
    /// let multi = MultiValues::Int32(vec![1, 2, 3]);
    ///
    /// // Through type inference
    /// let nums: Vec<i32> = multi.get().unwrap();
    /// assert_eq!(nums, vec![1, 2, 3]);
    ///
    /// // Explicitly specify type parameter
    /// let nums = multi.get::<i32>().unwrap();
    /// assert_eq!(nums, vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn get<T>(&self) -> ValueResult<Vec<T>>
    where
        Self: MultiValuesGetter<T>,
    {
        <Self as MultiValuesGetter<T>>::get_values(self)
    }

    /// Generic getter method for the first value
    ///
    /// Automatically selects the correct getter method based on the target type,
    /// performing strict type checking.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target element type to retrieve.
    ///
    /// # Returns
    ///
    /// If types match and a value exists, returns the first value; otherwise
    /// returns an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::MultiValues;
    ///
    /// let multi = MultiValues::Int32(vec![42, 100, 200]);
    ///
    /// // Through type inference
    /// let first: i32 = multi.get_first().unwrap();
    /// assert_eq!(first, 42);
    ///
    /// // Explicitly specify type parameter
    /// let first = multi.get_first::<i32>().unwrap();
    /// assert_eq!(first, 42);
    ///
    /// // String type
    /// let multi = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    /// let first: String = multi.get_first().unwrap();
    /// assert_eq!(first, "hello");
    /// ```
    #[inline]
    pub fn get_first<T>(&self) -> ValueResult<T>
    where
        Self: MultiValuesFirstGetter<T>,
    {
        <Self as MultiValuesFirstGetter<T>>::get_first_value(self)
    }

    /// Generic setter method
    ///
    /// Automatically selects the optimal setter path based on the input type,
    /// replacing the entire list.
    ///
    /// This operation updates the stored type to the input element type and
    /// does not validate runtime compatibility with the previous variant.
    ///
    /// Supports three input forms, all unified to this method via internal
    /// dispatch traits:
    ///
    /// - `Vec<T>`: Takes `set_values(Vec<T>)` path with zero additional allocation
    /// - `&[T]`: Takes `set_values_slice(&[T])` path
    /// - `T`: Takes `set_single_value(T)` path
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type, can be `Vec<T>`, `&[T]`, or a single `T`
    ///
    /// # Parameters
    ///
    /// * `values` - The value collection to set, can be `Vec<T>`, `&[T]`, or a
    ///   single `T`
    ///
    /// # Returns
    ///
    /// If setting succeeds, returns `Ok(())`; otherwise returns an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::lang::DataType;
    /// use crate::util::value::MultiValues;
    ///
    /// // 1) Vec<T>
    /// let mut mv = MultiValues::Empty(DataType::Int32);
    /// mv.set(vec![42, 100, 200]).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);
    ///
    /// // 2) &[T]
    /// let mut mv = MultiValues::Empty(DataType::Int32);
    /// let slice = &[7, 8, 9][..];
    /// mv.set(slice).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[7, 8, 9]);
    ///
    /// // 3) Single T
    /// let mut mv = MultiValues::Empty(DataType::Int32);
    /// mv.set(42).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42]);
    ///
    /// // String example
    /// let mut mv = MultiValues::Empty(DataType::String);
    /// mv.set(vec!["hello".to_string(), "world".to_string()]).unwrap();
    /// assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);
    /// ```
    #[inline]
    pub fn set<'a, S>(&mut self, values: S) -> ValueResult<()>
    where
        S: MultiValuesSetArg<'a>,
        Self: MultiValuesSetter<S::Item>
            + MultiValuesSetterSlice<S::Item>
            + MultiValuesSingleSetter<S::Item>,
    {
        values.apply(self)
    }

    /// Generic add method
    ///
    /// Automatically selects the optimal add path based on the input type,
    /// appending elements to the existing list with strict type checking.
    ///
    /// Supports three input forms:
    ///
    /// - `T`: Takes `add_value(T)` path, appending a single element
    /// - `Vec<T>`: Takes `add_values(Vec<T>)` path, batch append (zero additional allocation)
    /// - `&[T]`: Takes `add_values_slice(&[T])` path, batch append (using slice)
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type, can be a single `T`, `Vec<T>`, or `&[T]`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::lang::DataType;
    /// use crate::util::value::MultiValues;
    ///
    /// // 1) Single T
    /// let mut mv = MultiValues::Int32(vec![42]);
    /// mv.add(100).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100]);
    ///
    /// // 2) Vec<T>
    /// mv.add(vec![200, 300]).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200, 300]);
    ///
    /// // 3) &[T]
    /// let slice = &[400, 500][..];
    /// mv.add(slice).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200, 300, 400, 500]);
    /// ```
    #[inline]
    pub fn add<'a, S>(&mut self, values: S) -> ValueResult<()>
    where
        S: MultiValuesAddArg<'a>,
        Self: MultiValuesAdder<S::Item> + MultiValuesMultiAdder<S::Item>,
    {
        values.apply_add(self)
    }

    /// Get the data type of the values
    ///
    /// # Returns
    ///
    /// Returns the data type corresponding to these multiple values
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{MultiValues, DataType};
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline]
    pub fn data_type(&self) -> DataType {
        match self {
            MultiValues::Empty(dt) => *dt,
            MultiValues::Bool(_) => DataType::Bool,
            MultiValues::Char(_) => DataType::Char,
            MultiValues::Int8(_) => DataType::Int8,
            MultiValues::Int16(_) => DataType::Int16,
            MultiValues::Int32(_) => DataType::Int32,
            MultiValues::Int64(_) => DataType::Int64,
            MultiValues::Int128(_) => DataType::Int128,
            MultiValues::UInt8(_) => DataType::UInt8,
            MultiValues::UInt16(_) => DataType::UInt16,
            MultiValues::UInt32(_) => DataType::UInt32,
            MultiValues::UInt64(_) => DataType::UInt64,
            MultiValues::UInt128(_) => DataType::UInt128,
            MultiValues::Float32(_) => DataType::Float32,
            MultiValues::Float64(_) => DataType::Float64,
            MultiValues::String(_) => DataType::String,
            MultiValues::Date(_) => DataType::Date,
            MultiValues::Time(_) => DataType::Time,
            MultiValues::DateTime(_) => DataType::DateTime,
            MultiValues::Instant(_) => DataType::Instant,
            MultiValues::BigInteger(_) => DataType::BigInteger,
            MultiValues::BigDecimal(_) => DataType::BigDecimal,
            MultiValues::IntSize(_) => DataType::IntSize,
            MultiValues::UIntSize(_) => DataType::UIntSize,
            MultiValues::Duration(_) => DataType::Duration,
            MultiValues::Url(_) => DataType::Url,
            MultiValues::StringMap(_) => DataType::StringMap,
            MultiValues::Json(_) => DataType::Json,
        }
    }

    /// Get the number of values
    ///
    /// # Returns
    ///
    /// Returns the number of values contained in these multiple values
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.count(), 3);
    ///
    /// let empty = MultiValues::Empty(DataType::String);
    /// assert_eq!(empty.count(), 0);
    /// ```
    #[inline]
    pub fn count(&self) -> usize {
        match self {
            MultiValues::Empty(_) => 0,
            MultiValues::Bool(v) => v.len(),
            MultiValues::Char(v) => v.len(),
            MultiValues::Int8(v) => v.len(),
            MultiValues::Int16(v) => v.len(),
            MultiValues::Int32(v) => v.len(),
            MultiValues::Int64(v) => v.len(),
            MultiValues::Int128(v) => v.len(),
            MultiValues::UInt8(v) => v.len(),
            MultiValues::UInt16(v) => v.len(),
            MultiValues::UInt32(v) => v.len(),
            MultiValues::UInt64(v) => v.len(),
            MultiValues::UInt128(v) => v.len(),
            MultiValues::Float32(v) => v.len(),
            MultiValues::Float64(v) => v.len(),
            MultiValues::String(v) => v.len(),
            MultiValues::Date(v) => v.len(),
            MultiValues::Time(v) => v.len(),
            MultiValues::DateTime(v) => v.len(),
            MultiValues::Instant(v) => v.len(),
            MultiValues::BigInteger(v) => v.len(),
            MultiValues::BigDecimal(v) => v.len(),
            MultiValues::IntSize(v) => v.len(),
            MultiValues::UIntSize(v) => v.len(),
            MultiValues::Duration(v) => v.len(),
            MultiValues::Url(v) => v.len(),
            MultiValues::StringMap(v) => v.len(),
            MultiValues::Json(v) => v.len(),
        }
    }

    /// Check if empty
    ///
    /// # Returns
    ///
    /// Returns `true` if these multiple values do not contain any values
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{MultiValues, DataType};
    ///
    /// let values = MultiValues::Int32(vec![]);
    /// assert!(values.is_empty());
    ///
    /// let empty = MultiValues::Empty(DataType::String);
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Clear all values while preserving the type
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{MultiValues, DataType};
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.clear();
    /// assert!(values.is_empty());
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        match self {
            MultiValues::Empty(_) => {}
            MultiValues::Bool(v) => v.clear(),
            MultiValues::Char(v) => v.clear(),
            MultiValues::Int8(v) => v.clear(),
            MultiValues::Int16(v) => v.clear(),
            MultiValues::Int32(v) => v.clear(),
            MultiValues::Int64(v) => v.clear(),
            MultiValues::Int128(v) => v.clear(),
            MultiValues::UInt8(v) => v.clear(),
            MultiValues::UInt16(v) => v.clear(),
            MultiValues::UInt32(v) => v.clear(),
            MultiValues::UInt64(v) => v.clear(),
            MultiValues::UInt128(v) => v.clear(),
            MultiValues::Float32(v) => v.clear(),
            MultiValues::Float64(v) => v.clear(),
            MultiValues::String(v) => v.clear(),
            MultiValues::Date(v) => v.clear(),
            MultiValues::Time(v) => v.clear(),
            MultiValues::DateTime(v) => v.clear(),
            MultiValues::Instant(v) => v.clear(),
            MultiValues::BigInteger(v) => v.clear(),
            MultiValues::BigDecimal(v) => v.clear(),
            MultiValues::IntSize(v) => v.clear(),
            MultiValues::UIntSize(v) => v.clear(),
            MultiValues::Duration(v) => v.clear(),
            MultiValues::Url(v) => v.clear(),
            MultiValues::StringMap(v) => v.clear(),
            MultiValues::Json(v) => v.clear(),
        }
    }

    /// Set the data type
    ///
    /// If the new type differs from the current type, clears all values and
    /// sets the new type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - The data type to set
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::{MultiValues, DataType};
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.set_type(DataType::String);
    /// assert!(values.is_empty());
    /// assert_eq!(values.data_type(), DataType::String);
    /// ```
    #[inline]
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = MultiValues::Empty(data_type);
        }
    }

    // ========================================================================
    // Get first value (as single value access)
    // ========================================================================

    impl_get_first_value! {
        /// Get the first boolean value.
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first boolean value;
        /// otherwise returns an error.
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false]);
        /// assert_eq!(values.get_first_bool().unwrap(), true);
        /// ```
        copy: get_first_bool, Bool, bool, DataType::Bool
    }

    impl_get_first_value! {
        /// Get the first character value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first character value;
        /// otherwise returns an error.
        copy: get_first_char, Char, char, DataType::Char
    }

    impl_get_first_value! {
        /// Get the first int8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int8 value;
        /// otherwise returns an error
        copy: get_first_int8, Int8, i8, DataType::Int8
    }

    impl_get_first_value! {
        /// Get the first int16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int16 value;
        /// otherwise returns an error
        copy: get_first_int16, Int16, i16, DataType::Int16
    }

    impl_get_first_value! {
        /// Get the first int32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int32 value;
        /// otherwise returns an error
        copy: get_first_int32, Int32, i32, DataType::Int32
    }

    impl_get_first_value! {
        /// Get the first int64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int64 value;
        /// otherwise returns an error
        copy: get_first_int64, Int64, i64, DataType::Int64
    }

    impl_get_first_value! {
        /// Get the first int128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int128 value;
        /// otherwise returns an error
        copy: get_first_int128, Int128, i128, DataType::Int128
    }

    impl_get_first_value! {
        /// Get the first uint8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint8 value;
        /// otherwise returns an error
        copy: get_first_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_first_value! {
        /// Get the first uint16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint16 value;
        /// otherwise returns an error
        copy: get_first_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_first_value! {
        /// Get the first uint32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint32 value;
        /// otherwise returns an error
        copy: get_first_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_first_value! {
        /// Get the first uint64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint64 value;
        /// otherwise returns an error
        copy: get_first_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_first_value! {
        /// Get the first uint128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint128 value;
        /// otherwise returns an error
        copy: get_first_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_first_value! {
        /// Get the first float32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float32 value;
        /// otherwise returns an error
        copy: get_first_float32, Float32, f32, DataType::Float32
    }

    impl_get_first_value! {
        /// Get the first float64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float64 value;
        /// otherwise returns an error
        copy: get_first_float64, Float64, f64, DataType::Float64
    }

    impl_get_first_value! {
        /// Get the first string reference
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns a reference to the first
        /// string; otherwise returns an error
        ref: get_first_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    impl_get_first_value! {
        /// Get the first date value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first date value;
        /// otherwise returns an error
        copy: get_first_date, Date, NaiveDate, DataType::Date
    }

    impl_get_first_value! {
        /// Get the first time value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first time value;
        /// otherwise returns an error
        copy: get_first_time, Time, NaiveTime, DataType::Time
    }

    impl_get_first_value! {
        /// Get the first datetime value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first datetime value;
        /// otherwise returns an error
        copy: get_first_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_first_value! {
        /// Get the first UTC instant value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first UTC instant
        /// value; otherwise returns an error
        copy: get_first_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_first_value! {
        /// Get the first big integer value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big integer
        /// value; otherwise returns an error
        ref: get_first_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    impl_get_first_value! {
        /// Get the first big decimal value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big decimal
        /// value; otherwise returns an error
        ref: get_first_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal, |v: &BigDecimal| v.clone()
    }

    impl_get_first_value! {
        /// Get the first isize value
        copy: get_first_intsize, IntSize, isize, DataType::IntSize
    }

    impl_get_first_value! {
        /// Get the first usize value
        copy: get_first_uintsize, UIntSize, usize, DataType::UIntSize
    }

    impl_get_first_value! {
        /// Get the first Duration value
        copy: get_first_duration, Duration, Duration, DataType::Duration
    }

    impl_get_first_value! {
        /// Get the first Url value
        ref: get_first_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_first_value! {
        /// Get the first StringMap value
        ref: get_first_string_map, StringMap, HashMap<String, String>, DataType::StringMap, |v: &HashMap<String, String>| v.clone()
    }

    impl_get_first_value! {
        /// Get the first Json value
        ref: get_first_json, Json, serde_json::Value, DataType::Json, |v: &serde_json::Value| v.clone()
    }

    // ========================================================================
    // Get all values (type checking)
    // ========================================================================

    impl_get_multi_values! {
        /// Get reference to all boolean values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the boolean value array;
        /// otherwise returns an error
        ///
        /// # Example
        ///
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false, true]);
        /// assert_eq!(values.get_bools().unwrap(), &[true, false, true]);
        /// ```
        slice: get_bools, Bool, bool, DataType::Bool
    }

    impl_get_multi_values! {
        /// Get reference to all character values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the character value array;
        /// otherwise returns an error
        slice: get_chars, Char, char, DataType::Char
    }

    impl_get_multi_values! {
        /// Get reference to all int8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int8 value array;
        /// otherwise returns an error
        slice: get_int8s, Int8, i8, DataType::Int8
    }

    impl_get_multi_values! {
        /// Get reference to all int16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int16 value array;
        /// otherwise returns an error
        slice: get_int16s, Int16, i16, DataType::Int16
    }

    impl_get_multi_values! {
        /// Get reference to all int32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int32 value array;
        /// otherwise returns an error
        slice: get_int32s, Int32, i32, DataType::Int32
    }

    impl_get_multi_values! {
        /// Get reference to all int64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int64 value array;
        /// otherwise returns an error
        slice: get_int64s, Int64, i64, DataType::Int64
    }

    impl_get_multi_values! {
        /// Get reference to all int128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int128 value array;
        /// otherwise returns an error
        slice: get_int128s, Int128, i128, DataType::Int128
    }

    impl_get_multi_values! {
        /// Get reference to all uint8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint8 value array;
        /// otherwise returns an error
        slice: get_uint8s, UInt8, u8, DataType::UInt8
    }

    impl_get_multi_values! {
        /// Get reference to all uint16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint16 value array;
        /// otherwise returns an error
        slice: get_uint16s, UInt16, u16, DataType::UInt16
    }

    impl_get_multi_values! {
        /// Get reference to all uint32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint32 value array;
        /// otherwise returns an error
        slice: get_uint32s, UInt32, u32, DataType::UInt32
    }

    impl_get_multi_values! {
        /// Get reference to all uint64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint64 value array;
        /// otherwise returns an error
        slice: get_uint64s, UInt64, u64, DataType::UInt64
    }

    impl_get_multi_values! {
        /// Get reference to all uint128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint128 value array;
        /// otherwise returns an error
        slice: get_uint128s, UInt128, u128, DataType::UInt128
    }

    impl_get_multi_values! {
        /// Get reference to all float32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float32 value array;
        /// otherwise returns an error
        slice: get_float32s, Float32, f32, DataType::Float32
    }

    impl_get_multi_values! {
        /// Get reference to all float64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float64 value array;
        /// otherwise returns an error
        slice: get_float64s, Float64, f64, DataType::Float64
    }

    impl_get_multi_values! {
        /// Get reference to all strings
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string array; otherwise
        /// returns an error
        vec: get_strings, String, String, DataType::String
    }

    impl_get_multi_values! {
        /// Get reference to all date values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the date value array;
        /// otherwise returns an error
        slice: get_dates, Date, NaiveDate, DataType::Date
    }

    impl_get_multi_values! {
        /// Get reference to all time values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the time value array;
        /// otherwise returns an error
        slice: get_times, Time, NaiveTime, DataType::Time
    }

    impl_get_multi_values! {
        /// Get reference to all datetime values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the datetime value array;
        /// otherwise returns an error
        slice: get_datetimes, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_multi_values! {
        /// Get reference to all UTC instant values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the UTC instant value array;
        /// otherwise returns an error
        slice: get_instants, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_multi_values! {
        /// Get reference to all big integers
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big integer array;
        /// otherwise returns an error
        vec: get_bigintegers, BigInteger, BigInt, DataType::BigInteger
    }

    impl_get_multi_values! {
        /// Get reference to all big decimals
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big decimal array;
        /// otherwise returns an error
        vec: get_bigdecimals, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_get_multi_values! {
        /// Get reference to all isize values
        slice: get_intsizes, IntSize, isize, DataType::IntSize
    }

    impl_get_multi_values! {
        /// Get reference to all usize values
        slice: get_uintsizes, UIntSize, usize, DataType::UIntSize
    }

    impl_get_multi_values! {
        /// Get reference to all Duration values
        slice: get_durations, Duration, Duration, DataType::Duration
    }

    impl_get_multi_values! {
        /// Get reference to all Url values
        vec: get_urls, Url, Url, DataType::Url
    }

    impl_get_multi_values! {
        /// Get reference to all StringMap values
        vec: get_string_maps, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_get_multi_values! {
        /// Get reference to all Json values
        vec: get_jsons, Json, serde_json::Value, DataType::Json
    }

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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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
        /// ```rust,ignore
        /// use crate::util::value::MultiValues;
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

    /// Convert to a single [`Value`] by taking the first element.
    ///
    /// If there is no element, returns `Value::Empty(self.data_type())`.
    ///
    /// # Returns
    ///
    /// Returns the first element wrapped as [`Value`], or an empty value
    /// preserving the current data type.
    pub fn to_value(&self) -> Value {
        match self {
            MultiValues::Empty(dt) => Value::Empty(*dt),
            MultiValues::Bool(v) => v
                .first()
                .copied()
                .map(Value::Bool)
                .unwrap_or(Value::Empty(DataType::Bool)),
            MultiValues::Char(v) => v
                .first()
                .copied()
                .map(Value::Char)
                .unwrap_or(Value::Empty(DataType::Char)),
            MultiValues::Int8(v) => v
                .first()
                .copied()
                .map(Value::Int8)
                .unwrap_or(Value::Empty(DataType::Int8)),
            MultiValues::Int16(v) => v
                .first()
                .copied()
                .map(Value::Int16)
                .unwrap_or(Value::Empty(DataType::Int16)),
            MultiValues::Int32(v) => v
                .first()
                .copied()
                .map(Value::Int32)
                .unwrap_or(Value::Empty(DataType::Int32)),
            MultiValues::Int64(v) => v
                .first()
                .copied()
                .map(Value::Int64)
                .unwrap_or(Value::Empty(DataType::Int64)),
            MultiValues::Int128(v) => v
                .first()
                .copied()
                .map(Value::Int128)
                .unwrap_or(Value::Empty(DataType::Int128)),
            MultiValues::UInt8(v) => v
                .first()
                .copied()
                .map(Value::UInt8)
                .unwrap_or(Value::Empty(DataType::UInt8)),
            MultiValues::UInt16(v) => v
                .first()
                .copied()
                .map(Value::UInt16)
                .unwrap_or(Value::Empty(DataType::UInt16)),
            MultiValues::UInt32(v) => v
                .first()
                .copied()
                .map(Value::UInt32)
                .unwrap_or(Value::Empty(DataType::UInt32)),
            MultiValues::UInt64(v) => v
                .first()
                .copied()
                .map(Value::UInt64)
                .unwrap_or(Value::Empty(DataType::UInt64)),
            MultiValues::UInt128(v) => v
                .first()
                .copied()
                .map(Value::UInt128)
                .unwrap_or(Value::Empty(DataType::UInt128)),
            MultiValues::IntSize(v) => v
                .first()
                .copied()
                .map(Value::IntSize)
                .unwrap_or(Value::Empty(DataType::IntSize)),
            MultiValues::UIntSize(v) => v
                .first()
                .copied()
                .map(Value::UIntSize)
                .unwrap_or(Value::Empty(DataType::UIntSize)),
            MultiValues::Float32(v) => v
                .first()
                .copied()
                .map(Value::Float32)
                .unwrap_or(Value::Empty(DataType::Float32)),
            MultiValues::Float64(v) => v
                .first()
                .copied()
                .map(Value::Float64)
                .unwrap_or(Value::Empty(DataType::Float64)),
            MultiValues::BigInteger(v) => v
                .first()
                .cloned()
                .map(Value::BigInteger)
                .unwrap_or(Value::Empty(DataType::BigInteger)),
            MultiValues::BigDecimal(v) => v
                .first()
                .cloned()
                .map(Value::BigDecimal)
                .unwrap_or(Value::Empty(DataType::BigDecimal)),
            MultiValues::String(v) => v
                .first()
                .cloned()
                .map(Value::String)
                .unwrap_or(Value::Empty(DataType::String)),
            MultiValues::Date(v) => v
                .first()
                .copied()
                .map(Value::Date)
                .unwrap_or(Value::Empty(DataType::Date)),
            MultiValues::Time(v) => v
                .first()
                .copied()
                .map(Value::Time)
                .unwrap_or(Value::Empty(DataType::Time)),
            MultiValues::DateTime(v) => v
                .first()
                .copied()
                .map(Value::DateTime)
                .unwrap_or(Value::Empty(DataType::DateTime)),
            MultiValues::Instant(v) => v
                .first()
                .copied()
                .map(Value::Instant)
                .unwrap_or(Value::Empty(DataType::Instant)),
            MultiValues::Duration(v) => v
                .first()
                .copied()
                .map(Value::Duration)
                .unwrap_or(Value::Empty(DataType::Duration)),
            MultiValues::Url(v) => v
                .first()
                .cloned()
                .map(Value::Url)
                .unwrap_or(Value::Empty(DataType::Url)),
            MultiValues::StringMap(v) => v
                .first()
                .cloned()
                .map(Value::StringMap)
                .unwrap_or(Value::Empty(DataType::StringMap)),
            MultiValues::Json(v) => v
                .first()
                .cloned()
                .map(Value::Json)
                .unwrap_or(Value::Empty(DataType::Json)),
        }
    }

    /// Merge another multiple values
    ///
    /// Append all values from another multiple values to the current multiple values
    ///
    /// # Parameters
    ///
    /// * `other` - The multiple values to merge
    ///
    /// # Returns
    ///
    /// If types match, returns `Ok(())`; otherwise returns an error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::util::value::MultiValues;
    ///
    /// let mut a = MultiValues::Int32(vec![1, 2]);
    /// let b = MultiValues::Int32(vec![3, 4]);
    /// a.merge(&b).unwrap();
    /// assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);
    /// ```
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }

        match (self, other) {
            (MultiValues::Bool(v), MultiValues::Bool(o)) => v.extend_from_slice(o),
            (MultiValues::Char(v), MultiValues::Char(o)) => v.extend_from_slice(o),
            (MultiValues::Int8(v), MultiValues::Int8(o)) => v.extend_from_slice(o),
            (MultiValues::Int16(v), MultiValues::Int16(o)) => v.extend_from_slice(o),
            (MultiValues::Int32(v), MultiValues::Int32(o)) => v.extend_from_slice(o),
            (MultiValues::Int64(v), MultiValues::Int64(o)) => v.extend_from_slice(o),
            (MultiValues::Int128(v), MultiValues::Int128(o)) => v.extend_from_slice(o),
            (MultiValues::UInt8(v), MultiValues::UInt8(o)) => v.extend_from_slice(o),
            (MultiValues::UInt16(v), MultiValues::UInt16(o)) => v.extend_from_slice(o),
            (MultiValues::UInt32(v), MultiValues::UInt32(o)) => v.extend_from_slice(o),
            (MultiValues::UInt64(v), MultiValues::UInt64(o)) => v.extend_from_slice(o),
            (MultiValues::UInt128(v), MultiValues::UInt128(o)) => v.extend_from_slice(o),
            (MultiValues::Float32(v), MultiValues::Float32(o)) => v.extend_from_slice(o),
            (MultiValues::Float64(v), MultiValues::Float64(o)) => v.extend_from_slice(o),
            (MultiValues::String(v), MultiValues::String(o)) => v.extend_from_slice(o),
            (MultiValues::Date(v), MultiValues::Date(o)) => v.extend_from_slice(o),
            (MultiValues::Time(v), MultiValues::Time(o)) => v.extend_from_slice(o),
            (MultiValues::DateTime(v), MultiValues::DateTime(o)) => v.extend_from_slice(o),
            (MultiValues::Instant(v), MultiValues::Instant(o)) => v.extend_from_slice(o),
            (MultiValues::BigInteger(v), MultiValues::BigInteger(o)) => v.extend_from_slice(o),
            (MultiValues::BigDecimal(v), MultiValues::BigDecimal(o)) => v.extend_from_slice(o),
            (MultiValues::IntSize(v), MultiValues::IntSize(o)) => v.extend_from_slice(o),
            (MultiValues::UIntSize(v), MultiValues::UIntSize(o)) => v.extend_from_slice(o),
            (MultiValues::Duration(v), MultiValues::Duration(o)) => v.extend_from_slice(o),
            (MultiValues::Url(v), MultiValues::Url(o)) => v.extend_from_slice(o),
            (MultiValues::StringMap(v), MultiValues::StringMap(o)) => v.extend(o.iter().cloned()),
            (MultiValues::Json(v), MultiValues::Json(o)) => v.extend(o.iter().cloned()),
            (MultiValues::Empty(_), _) => {}
            _ => unreachable!(),
        }

        Ok(())
    }
}

impl Default for MultiValues {
    #[inline]
    fn default() -> Self {
        MultiValues::Empty(DataType::String)
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        match value {
            Value::Empty(dt) => MultiValues::Empty(dt),
            Value::Bool(v) => MultiValues::Bool(vec![v]),
            Value::Char(v) => MultiValues::Char(vec![v]),
            Value::Int8(v) => MultiValues::Int8(vec![v]),
            Value::Int16(v) => MultiValues::Int16(vec![v]),
            Value::Int32(v) => MultiValues::Int32(vec![v]),
            Value::Int64(v) => MultiValues::Int64(vec![v]),
            Value::Int128(v) => MultiValues::Int128(vec![v]),
            Value::UInt8(v) => MultiValues::UInt8(vec![v]),
            Value::UInt16(v) => MultiValues::UInt16(vec![v]),
            Value::UInt32(v) => MultiValues::UInt32(vec![v]),
            Value::UInt64(v) => MultiValues::UInt64(vec![v]),
            Value::UInt128(v) => MultiValues::UInt128(vec![v]),
            Value::Float32(v) => MultiValues::Float32(vec![v]),
            Value::Float64(v) => MultiValues::Float64(vec![v]),
            Value::String(v) => MultiValues::String(vec![v]),
            Value::Date(v) => MultiValues::Date(vec![v]),
            Value::Time(v) => MultiValues::Time(vec![v]),
            Value::DateTime(v) => MultiValues::DateTime(vec![v]),
            Value::Instant(v) => MultiValues::Instant(vec![v]),
            Value::BigInteger(v) => MultiValues::BigInteger(vec![v]),
            Value::BigDecimal(v) => MultiValues::BigDecimal(vec![v]),
            Value::IntSize(v) => MultiValues::IntSize(vec![v]),
            Value::UIntSize(v) => MultiValues::UIntSize(vec![v]),
            Value::Duration(v) => MultiValues::Duration(vec![v]),
            Value::Url(v) => MultiValues::Url(vec![v]),
            Value::StringMap(v) => MultiValues::StringMap(vec![v]),
            Value::Json(v) => MultiValues::Json(vec![v]),
        }
    }
}

// ============================================================================
// Internal generic conversion traits (private, not exported, to avoid polluting
// the standard type namespace).
// ============================================================================

/// Internal trait: used to extract multiple values from MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesGetter<T> {
    fn get_values(&self) -> ValueResult<Vec<T>>;
}

/// Internal trait: used to extract the first value from MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesFirstGetter<T> {
    fn get_first_value(&self) -> ValueResult<T>;
}

/// Internal trait: used to set specific types in MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSetter<T> {
    fn set_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}

/// Internal dispatch trait: dispatches Vec<T>, &[T], T to optimal set path
#[doc(hidden)]
pub trait MultiValuesSetArg<'a> {
    /// Element type
    type Item: 'a + Clone;

    fn apply(self, target: &mut MultiValues) -> ValueResult<()>;
}

/// Internal trait: used to set specific types in MultiValues via slice
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSetterSlice<T> {
    fn set_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

/// Internal trait: used to set a single value in MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSingleSetter<T> {
    fn set_single_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to add a single value to MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesAdder<T> {
    fn add_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to add multiple values to MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesMultiAdder<T> {
    fn add_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}

/// Internal dispatch trait: dispatches T / Vec<T> / &[T] to optimal add path
#[doc(hidden)]
pub trait MultiValuesAddArg<'a> {
    /// Element type
    type Item: 'a + Clone;

    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()>;
}

/// Internal trait: used to append multiple values to MultiValues via slice
/// (calls add_[xxx]s_slice by type)
#[doc(hidden)]
pub(crate) trait MultiValuesMultiAdderSlice<T> {
    fn add_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

/// Internal trait: used to create MultiValues from Vec<T>
///
/// This trait is not exported in mod.rs, only used for internal implementation,
/// to avoid polluting the standard type namespace
#[doc(hidden)]
pub(crate) trait MultiValuesConstructor<T> {
    fn from_vec(values: Vec<T>) -> Self;
}

// ============================================================================
// Internal trait implementations (simplified using macros)
// ============================================================================

macro_rules! impl_multi_value_traits {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl MultiValuesGetter<$type> for MultiValues {
            #[inline]
            fn get_values(&self) -> ValueResult<Vec<$type>> {
                match self {
                    MultiValues::$variant(v) => Ok(v.clone()),
                    MultiValues::Empty(dt) if *dt == $data_type => Ok(Vec::new()),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }

        impl MultiValuesFirstGetter<$type> for MultiValues {
            #[inline]
            fn get_first_value(&self) -> ValueResult<$type> {
                match self {
                    MultiValues::$variant(v) if !v.is_empty() => Ok(v[0].clone()),
                    MultiValues::$variant(_) => Err(ValueError::NoValue),
                    MultiValues::Empty(dt) if *dt == $data_type => Err(ValueError::NoValue),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }

        impl MultiValuesSetter<$type> for MultiValues {
            #[inline]
            fn set_values(&mut self, values: Vec<$type>) -> ValueResult<()> {
                *self = MultiValues::$variant(values);
                Ok(())
            }
        }

        // Generic From implementation for SetParam is at the top level, not
        // repeated here for specific types.

        impl MultiValuesSetterSlice<$type> for MultiValues {
            #[inline]
            fn set_values_slice(&mut self, values: &[$type]) -> ValueResult<()> {
                // Equivalent to set_[xxx]s_slice: replace entire list with slice
                *self = MultiValues::$variant(values.to_vec());
                Ok(())
            }
        }

        impl MultiValuesSingleSetter<$type> for MultiValues {
            #[inline]
            fn set_single_value(&mut self, value: $type) -> ValueResult<()> {
                *self = MultiValues::$variant(vec![value]);
                Ok(())
            }
        }

        impl MultiValuesAdder<$type> for MultiValues {
            #[inline]
            fn add_value(&mut self, value: $type) -> ValueResult<()> {
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
        }

        // Three types of implementations for local dispatch trait
        impl<'a> MultiValuesSetArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetter<$type>>::set_values(target, self)
            }
        }

        impl<'a> MultiValuesSetArg<'a> for &'a [$type]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetterSlice<$type>>::set_values_slice(target, self)
            }
        }

        impl<'a> MultiValuesSetArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSingleSetter<$type>>::set_single_value(target, self)
            }
        }

        impl MultiValuesMultiAdder<$type> for MultiValues {
            #[inline]
            fn add_values(&mut self, values: Vec<$type>) -> ValueResult<()> {
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
        }

        impl MultiValuesMultiAdderSlice<$type> for MultiValues {
            #[inline]
            fn add_values_slice(&mut self, values: &[$type]) -> ValueResult<()> {
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
        }

        // add dispatch: T / Vec<T> / &[T]
        impl<'a> MultiValuesAddArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesAdder<$type>>::add_value(target, self)
            }
        }

        impl<'a> MultiValuesAddArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdder<$type>>::add_values(target, self)
            }
        }

        impl<'a> MultiValuesAddArg<'a> for &'a [$type]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdderSlice<$type>>::add_values_slice(target, self)
            }
        }

        impl MultiValuesConstructor<$type> for MultiValues {
            #[inline]
            fn from_vec(values: Vec<$type>) -> Self {
                MultiValues::$variant(values)
            }
        }
    };
}

// Implementation for Copy types
impl_multi_value_traits!(bool, Bool, DataType::Bool);
impl_multi_value_traits!(char, Char, DataType::Char);
impl_multi_value_traits!(i8, Int8, DataType::Int8);
impl_multi_value_traits!(i16, Int16, DataType::Int16);
impl_multi_value_traits!(i32, Int32, DataType::Int32);
impl_multi_value_traits!(i64, Int64, DataType::Int64);
impl_multi_value_traits!(i128, Int128, DataType::Int128);
impl_multi_value_traits!(u8, UInt8, DataType::UInt8);
impl_multi_value_traits!(u16, UInt16, DataType::UInt16);
impl_multi_value_traits!(u32, UInt32, DataType::UInt32);
impl_multi_value_traits!(u64, UInt64, DataType::UInt64);
impl_multi_value_traits!(u128, UInt128, DataType::UInt128);
impl_multi_value_traits!(f32, Float32, DataType::Float32);
impl_multi_value_traits!(f64, Float64, DataType::Float64);
impl_multi_value_traits!(String, String, DataType::String);
impl_multi_value_traits!(NaiveDate, Date, DataType::Date);
impl_multi_value_traits!(NaiveTime, Time, DataType::Time);
impl_multi_value_traits!(NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_value_traits!(DateTime<Utc>, Instant, DataType::Instant);
impl_multi_value_traits!(BigInt, BigInteger, DataType::BigInteger);
impl_multi_value_traits!(BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_value_traits!(isize, IntSize, DataType::IntSize);
impl_multi_value_traits!(usize, UIntSize, DataType::UIntSize);
impl_multi_value_traits!(Duration, Duration, DataType::Duration);
impl_multi_value_traits!(Url, Url, DataType::Url);
impl_multi_value_traits!(HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_value_traits!(serde_json::Value, Json, DataType::Json);

// Convenience adaptation: &str supported as input type for String
impl MultiValuesSetArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesSingleSetter<String>>::set_single_value(target, self.to_string())
    }
}

impl MultiValuesSetArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl<'b> MultiValuesSetArg<'_> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl MultiValuesAddArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesAdder<String>>::add_value(target, self.to_string())
    }
}

impl MultiValuesAddArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl<'b> MultiValuesAddArg<'_> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}
