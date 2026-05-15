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

macro_rules! multi_values_append_match {
    ($left:expr, $right:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match ($left, $right) {
            $(
                (MultiValues::$variant(values), MultiValues::$variant(mut other_values)) => {
                    values.append(&mut other_values);
                }
            )+
            (slot @ MultiValues::Empty(_), other_values) => *slot = other_values,
            _ => unreachable!(),
        }
    };
}

impl MultiValues {
    /// Generic constructor method
    ///
    /// Creates `MultiValues` from any supported input form, avoiding direct
    /// use of enum variants at call sites.
    ///
    /// Supported input forms include single values, vectors, slices, arrays,
    /// borrowed vectors, and borrowed string collections for supported element
    /// types.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Returns
    ///
    /// Returns `MultiValues` wrapping the converted input values.
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
    pub fn new<S>(values: S) -> Self
    where
        S: Into<Self>,
    {
        values.into()
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
    /// Returns the list of values when the stored type matches `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`.
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
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
    {
        Vec::<T>::try_from(self)
    }

    /// Generic getter method with a default value list.
    ///
    /// Returns the supplied default only when the stored list is empty. Type
    /// mismatches are still returned as errors.
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
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
    /// Reads the first stored value as `T`, performing strict type checking.
    ///
    /// `get_first<T>()` does not do cross-type conversion. Use [`Self::to`] if
    /// conversion between compatible data types is desired.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target element type to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the first value when the stored type matches `T` and at least
    /// one value exists.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] when the requested type matches but no
    /// value is stored, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
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
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        T::try_from(self)
    }

    /// Generic first-value getter with a default value.
    ///
    /// Returns the supplied default only when no first value exists. Type
    /// mismatches are still returned as errors.
    #[inline]
    pub fn get_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get_first() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Generic setter method
    ///
    /// Replaces the entire list with the converted input values.
    ///
    /// This operation updates the stored type to the input element type and
    /// does not validate runtime compatibility with the previous variant.
    ///
    /// Supports any input that can be converted into [`MultiValues`], including
    /// single values, vectors, slices, arrays, and borrowed vectors for supported
    /// element types.
    ///
    /// Existing values are replaced, and the stored type becomes the converted
    /// input type.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Parameters
    ///
    /// * `values` - The values to set.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` for supported input types. Unsupported input
    /// types fail to compile because they do not implement `Into<MultiValues>`.
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
    pub fn set<S>(&mut self, values: S) -> ValueResult<()>
    where
        S: Into<Self>,
    {
        *self = values.into();
        Ok(())
    }

    /// Generic add method
    ///
    /// Appends converted input values to the existing list with strict type checking.
    ///
    /// Supports any input that can be converted into [`MultiValues`], including
    /// single values, vectors, slices, arrays, and borrowed vectors for supported
    /// element types.
    ///
    /// The converted input must have the same data type as the current container.
    /// An empty container keeps its declared type until non-empty values of the
    /// same type are appended.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the converted input data type
    /// differs from the current container data type.
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
    pub fn add<S>(&mut self, values: S) -> ValueResult<()>
    where
        S: Into<Self>,
    {
        let other = values.into();
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }
        if other.count() == 0 {
            return Ok(());
        }

        for_each_multi_value_type!(multi_values_append_match, self, other);

        Ok(())
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
