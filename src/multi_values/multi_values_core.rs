/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Core generic accessors and state methods for `MultiValues`.

use qubit_datatype::DataType;

use crate::IntoValueDefault;
use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;
use super::multi_values_add_arg::MultiValuesAddArg;
use super::multi_values_adder::MultiValuesAdder;
use super::multi_values_constructor_arg::MultiValuesConstructorArg;
use super::multi_values_first_getter::MultiValuesFirstGetter;
use super::multi_values_getter::MultiValuesGetter;
use super::multi_values_multi_adder::MultiValuesMultiAdder;
use super::multi_values_set_arg::MultiValuesSetArg;
use super::multi_values_setter::MultiValuesSetter;
use super::multi_values_setter_slice::MultiValuesSetterSlice;
use super::multi_values_single_setter::MultiValuesSingleSetter;

macro_rules! multi_values_data_type_match {
    ($value:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match $value {
            MultiValues::Empty(dt) => *dt,
            $(MultiValues::$variant(_) => $data_type,)+
        }
    };
}

macro_rules! multi_values_count_match {
    ($value:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match $value {
            MultiValues::Empty(_) => 0,
            $(MultiValues::$variant(values) => values.len(),)+
        }
    };
}

macro_rules! multi_values_clear_match {
    ($value:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match $value {
            MultiValues::Empty(_) => {}
            $(MultiValues::$variant(values) => values.clear(),)+
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
    pub fn new<'a, S>(values: S) -> Self
    where
        S: MultiValuesConstructorArg<'a>,
    {
        values.into_multi_values()
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

    /// Generic getter method with a default value list.
    ///
    /// Returns the supplied default only when the stored list is empty. Type
    /// mismatches are still returned as errors.
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        Self: MultiValuesGetter<T>,
    {
        let values = self.get()?;
        if values.is_empty() {
            Ok(default.into_value_default())
        } else {
            Ok(values)
        }
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

    /// Generic first-value getter with a default value.
    ///
    /// Returns the supplied default only when no first value exists. Type
    /// mismatches are still returned as errors.
    #[inline]
    pub fn get_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        Self: MultiValuesFirstGetter<T>,
    {
        match self.get_first() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
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
    /// use qubit_datatype::DataType;
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
    /// use qubit_datatype::DataType;
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
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline]
    pub fn data_type(&self) -> DataType {
        for_each_multi_value_type!(multi_values_data_type_match, self)
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
    /// use qubit_datatype::DataType;
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
        for_each_multi_value_type!(multi_values_count_match, self)
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
    /// use qubit_datatype::DataType;
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
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.clear();
    /// assert!(values.is_empty());
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        for_each_multi_value_type!(multi_values_clear_match, self)
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
    /// use qubit_datatype::DataType;
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
}
