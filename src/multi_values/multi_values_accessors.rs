/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

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

use qubit_common::lang::DataType;

use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;
use super::multi_values_add_arg::MultiValuesAddArg;
use super::multi_values_adder::MultiValuesAdder;
use super::multi_values_constructor::MultiValuesConstructor;
use super::multi_values_first_getter::MultiValuesFirstGetter;
use super::multi_values_getter::MultiValuesGetter;
use super::multi_values_multi_adder::MultiValuesMultiAdder;
use super::multi_values_set_arg::MultiValuesSetArg;
use super::multi_values_setter::MultiValuesSetter;
use super::multi_values_setter_slice::MultiValuesSetterSlice;
use super::multi_values_single_setter::MultiValuesSingleSetter;

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
    /// ```rust
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
    /// ```rust
    /// use qubit_common::lang::DataType;
    /// use qubit_value::MultiValues;
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
        /// ```rust
        /// use qubit_value::MultiValues;
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
        /// ```rust
        /// use qubit_value::MultiValues;
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
        /// ```rust
        /// use qubit_common::lang::DataType;
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
        /// use qubit_common::lang::DataType;
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
        /// use qubit_common::lang::DataType;
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
        /// use qubit_common::lang::DataType;
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
        /// ```rust
        /// use qubit_value::MultiValues;
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
        /// ```rust
        /// use qubit_value::MultiValues;
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
        /// ```rust
        /// use qubit_value::MultiValues;
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
}
