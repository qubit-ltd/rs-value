// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Core generic accessors and state methods for `MultiValues`.

use qubit_datatype::DataType;

use crate::value_error::{ValueError, ValueResult};
use crate::{IntoValueDefault, Value};

use super::multi_values::MultiValues;

macro_rules! multi_values_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(dt) => *dt,
            $($(#[$cfg])* MultiValues::$variant(_) => $data_type,)+
        }
    };
}

macro_rules! multi_values_count_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(_) => 0,
            $($(#[$cfg])* MultiValues::$variant(values) => values.len(),)+
        }
    };
}

macro_rules! multi_values_clear_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(_) => {}
            $($(#[$cfg])* MultiValues::$variant(values) => values.clear(),)+
        }
    };
}

macro_rules! multi_values_append_match {
    ($left:expr, $right:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match ($left, $right) {
            $(
                $(#[$cfg])*
                (MultiValues::$variant(values), MultiValues::$variant(mut other_values)) => {
                    values.append(&mut other_values);
                }
            )+
            (slot @ MultiValues::Unset(_), other_values) => *slot = other_values,
            _ => unreachable!(),
        }
    };
}

macro_rules! multi_values_first_value_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(data_type) => Value::Unset(*data_type),
            $(
                $(#[$cfg])*
                MultiValues::$variant(values) => values
                    .first()
                    .map(|value| materialize_stored!($materialization, value))
                    .map(|value| Value::$variant(value_storage_new!($variant, value)))
                    .unwrap_or(Value::Unset($data_type)),
            )+
        }
    };
}

macro_rules! multi_values_into_first_value_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(data_type) => Value::Unset(data_type),
            $(
                $(#[$cfg])*
                MultiValues::$variant(values) => values
                    .into_iter()
                    .next()
                    .map(|value| Value::$variant(value_storage_new!($variant, value)))
                    .unwrap_or(Value::Unset($data_type)),
            )+
        }
    };
}

macro_rules! multi_values_merge_match {
    ($left:expr, $right:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match ($left, $right) {
            $(
                $(#[$cfg])*
                (MultiValues::$variant(values), MultiValues::$variant(other_values)) => {
                    values.extend_from_slice(other_values)
                }
            )+
            (slot @ MultiValues::Unset(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }
    };
}

macro_rules! value_into_multi_values_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            Value::Unset(data_type) => MultiValues::Unset(data_type),
            $($(#[$cfg])* Value::$variant(value) => {
                MultiValues::$variant(vec![value_storage_into_multi!($variant, value)])
            },)+
        }
    };
}

impl MultiValues {
    /// Creates an unset collection with an explicit declared element type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - Declared element type retained while the collection is
    ///   unset.
    ///
    /// # Returns
    ///
    /// An unset collection carrying `data_type`.
    #[inline(always)]
    pub const fn new_unset(data_type: DataType) -> Self {
        Self::Unset(data_type)
    }

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
    /// # Parameters
    ///
    /// * `values` - Values to convert into a collection.
    ///
    /// # Returns
    ///
    /// Returns `MultiValues` wrapping the converted input values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// // Basic types
    /// let mv = MultiValues::new(vec![1, 2, 3]);
    /// assert_eq!(mv.len(), 3);
    ///
    /// // Strings
    /// let mv = MultiValues::new(vec!["a".to_string(), "b".to_string()]);
    /// assert_eq!(mv.len(), 2);
    /// ```
    #[inline(always)]
    pub fn new<S>(values: S) -> Self
    where
        S: Into<Self>,
    {
        values.into()
    }

    /// Generic getter method for multiple values.
    ///
    /// Performs a strict typed read of all stored values as `Vec<T>`.
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
    /// Returns [`ValueError::NoValue`] when the container is unset with the
    /// requested type, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Examples
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
    #[inline(always)]
    pub fn get<T>(&self) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
    {
        Vec::<T>::try_from(self)
    }

    /// Generic getter method with a default value list.
    ///
    /// Returns the supplied default only when this container is unset. A
    /// concrete empty vector remains an empty result.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type for the strict read.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used only for unset storage.
    ///
    /// # Returns
    ///
    /// The concrete stored list, or `default` for unset storage.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`.
    #[inline]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Strictly reads all values or calls `default` only when storage is unset.
    ///
    /// A concrete empty collection is returned unchanged and type mismatches
    /// are preserved without invoking the callback.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only for unset storage.
    ///
    /// # Returns
    ///
    /// The stored list or the callback result.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] for an incompatible concrete
    /// collection without invoking the callback.
    #[inline]
    pub fn get_or_else<T, F>(&self, default: F) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> Vec<T>,
    {
        match self.get() {
            Err(ValueError::NoValue) if self.is_unset() => Ok(default()),
            result => result,
        }
    }

    /// Generic getter method for the first value
    ///
    /// Reads the first stored value as `T`, performing strict type checking.
    ///
    /// `get_first<T>()` does not do cross-type conversion. When the `converter`
    /// feature is enabled, use `to<T>()` for compatible cross-type conversion.
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
    /// # Examples
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
    #[inline(always)]
    pub fn get_first<T>(&self) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        T::try_from(self)
    }

    /// Generic first-value getter with a default value.
    ///
    /// Returns the supplied default only when the container is unset. A
    /// concrete empty vector returns [`ValueError::NoValue`]; type mismatches
    /// are also preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict first-item read.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used only for unset storage.
    ///
    /// # Returns
    ///
    /// The first concrete item, or `default` for unset storage.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for a concrete empty collection or
    /// [`ValueError::TypeMismatch`] when the stored type differs from `T`.
    #[inline]
    pub fn get_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get_first() {
            Err(ValueError::NoValue) if self.is_unset() => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Strictly reads the first value or calls `default` only when unset.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    /// * `F` - Deferred fallback producing one element.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only for unset storage.
    ///
    /// # Returns
    ///
    /// The first stored item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves empty-collection and type-mismatch errors without invoking
    /// the callback.
    #[inline]
    pub fn get_first_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> T,
    {
        match self.get_first() {
            Err(ValueError::NoValue) if self.is_unset() => Ok(default()),
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
    /// single values, vectors, slices, arrays, and borrowed vectors for
    /// supported element types.
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
    /// # Compile-time restriction
    ///
    /// Unsupported input types fail to compile because they do not implement
    /// `Into<MultiValues>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// // 1) Vec<T>
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// mv.set(vec![42, 100, 200]);
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);
    ///
    /// // 2) &[T]
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// let slice = &[7, 8, 9][..];
    /// mv.set(slice);
    /// assert_eq!(mv.get_int32s().unwrap(), &[7, 8, 9]);
    ///
    /// // 3) Single T
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// mv.set(42);
    /// assert_eq!(mv.get_int32s().unwrap(), &[42]);
    ///
    /// // String example
    /// let mut mv = MultiValues::Unset(DataType::String);
    /// mv.set(vec!["hello".to_string(), "world".to_string()]);
    /// assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);
    /// ```
    #[inline(always)]
    pub fn set<S>(&mut self, values: S)
    where
        S: Into<Self>,
    {
        *self = values.into();
    }

    /// Generic add method
    ///
    /// Appends converted input values to the existing list with strict type
    /// checking.
    ///
    /// Supports any input that can be converted into [`MultiValues`], including
    /// single values, vectors, slices, arrays, and borrowed vectors for
    /// supported element types.
    ///
    /// The converted input must have the same data type as the current
    /// container. An empty container keeps its declared type until
    /// non-empty values of the same type are appended.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Parameters
    ///
    /// * `values` - Values to append.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending, including when the input is empty.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the converted input data type
    /// differs from the current container data type.
    ///
    /// # Examples
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
        if other.is_empty() {
            return Ok(());
        }

        for_each_value_type!(multi_values_append_match, self, other);

        Ok(())
    }

    /// Get the data type of the values
    ///
    /// # Returns
    ///
    /// Returns the data type corresponding to these multiple values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        for_each_value_type!(multi_values_data_type_match, self)
    }

    /// Returns the number of values.
    ///
    /// # Returns
    ///
    /// The number of values contained in these multiple values. An unset
    /// collection has length zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.len(), 3);
    ///
    /// let empty = MultiValues::Unset(DataType::String);
    /// assert_eq!(empty.len(), 0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        for_each_value_type!(multi_values_count_match, self)
    }

    /// Tests whether this collection contains no values.
    ///
    /// Both an unset collection and a concrete empty vector are empty. Use
    /// [`MultiValues::is_unset`] to distinguish those states.
    ///
    /// # Returns
    ///
    /// `true` when [`Self::len`] is zero; otherwise, `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Tests whether this container has no concrete vector.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`MultiValues::Unset`]. A concrete empty vector
    /// returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![]);
    /// assert!(!values.is_unset());
    ///
    /// let empty = MultiValues::Unset(DataType::String);
    /// assert!(empty.is_unset());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_unset(&self) -> bool {
        matches!(self, MultiValues::Unset(_))
    }

    /// Tests whether a concrete collection belongs to the numeric type family.
    ///
    /// A concrete empty numeric vector returns `true`; an unset collection
    /// returns `false`, even when its declared type is numeric.
    ///
    /// # Returns
    ///
    /// `true` for concrete collections with a numeric element type.
    #[inline(always)]
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete vector while preserving its declared data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        *self = MultiValues::Unset(self.data_type());
    }

    /// Clear all values while preserving the type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.clear();
    /// assert_eq!(values.len(), 0);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        for_each_value_type!(multi_values_clear_match, self)
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
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.set_type(DataType::String);
    /// assert!(values.is_unset());
    /// assert_eq!(values.data_type(), DataType::String);
    /// ```
    #[inline]
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = MultiValues::Unset(data_type);
        }
    }

    /// Converts the first element to a single [`Value`].
    ///
    /// Returns `Value::Unset` with the same declared type when no element is
    /// stored.
    ///
    /// # Returns
    ///
    /// A cloned first item, or a typed unset value when no item exists.
    pub fn first_value(&self) -> Value {
        for_each_value_type!(multi_values_first_value_match, self)
    }

    /// Consumes this collection and returns its first item as a [`Value`].
    ///
    /// Empty and unset collections become [`Value::Unset`] with the same data
    /// type. Owned element storage is moved instead of cloned.
    ///
    /// # Returns
    ///
    /// The owned first item, or a typed unset value when no item exists.
    pub fn into_first_value(self) -> Value {
        for_each_value_type!(multi_values_into_first_value_match, self)
    }

    /// Appends all values from another container with the same data type.
    ///
    /// # Parameters
    ///
    /// * `other` - Collection whose values are cloned and appended.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending, including when `other` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when `other` has a different data
    /// type.
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }
        if other.is_empty() {
            return Ok(());
        }
        for_each_value_type!(multi_values_merge_match, self, other);
        Ok(())
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        for_each_value_type!(value_into_multi_values_match, value)
    }
}
