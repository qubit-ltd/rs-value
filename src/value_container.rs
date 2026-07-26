// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit scalar-or-collection value storage.

use crate::{
    MultiValues,
    StrictValueListRead,
    StrictValueRead,
    Value,
    ValueError,
    ValueResult,
};
use qubit_datatype::DataType;
#[cfg(feature = "converter")]
use qubit_datatype::{
    DataConversionOptions,
    DataConversionTarget,
    ScalarStringDataConverters,
};

/// A typed value whose scalar or collection shape is explicit.
///
/// The shape is never inferred from collection length. In particular,
/// `Scalar(Value::Int32(42))` and
/// `Collection(MultiValues::Int32(vec![42]))` remain distinguishable through
/// conversion and serialization boundaries.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueContainer {
    /// One typed value.
    Scalar(
        /// Stored scalar value.
        Value,
    ),
    /// A homogeneous typed collection.
    Collection(
        /// Stored homogeneous collection.
        MultiValues,
    ),
}

macro_rules! impl_value_container_from_table {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        $(
            $(#[$cfg])*
            impl From<$type> for ValueContainer {
                #[inline(always)]
                fn from(value: $type) -> Self {
                    Self::Scalar(Value::$variant(value_storage_new!($variant, value)))
                }
            }

            $(#[$cfg])*
            impl From<Vec<$type>> for ValueContainer {
                #[inline(always)]
                fn from(values: Vec<$type>) -> Self {
                    Self::Collection(MultiValues::$variant(values))
                }
            }

            $(#[$cfg])*
            impl From<&[$type]> for ValueContainer {
                #[inline]
                fn from(values: &[$type]) -> Self {
                    Self::Collection(MultiValues::$variant(values.to_vec()))
                }
            }

            $(#[$cfg])*
            impl From<&Vec<$type>> for ValueContainer {
                #[inline]
                fn from(values: &Vec<$type>) -> Self {
                    Self::Collection(MultiValues::$variant(values.clone()))
                }
            }

            $(#[$cfg])*
            impl<const N: usize> From<[$type; N]> for ValueContainer {
                #[inline]
                fn from(values: [$type; N]) -> Self {
                    Self::Collection(MultiValues::$variant(Vec::from(values)))
                }
            }

            $(#[$cfg])*
            impl<const N: usize> From<&[$type; N]> for ValueContainer {
                #[inline]
                fn from(values: &[$type; N]) -> Self {
                    Self::Collection(MultiValues::$variant(values.to_vec()))
                }
            }
        )+
    };
}

for_each_value_type!(impl_value_container_from_table);

/// Builds a typed collection from one or two same-typed scalar values.
macro_rules! value_container_pair_match {
    ($first:expr, $second:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match ($first, $second) {
            $(
                $(#[$cfg])*
                (Value::$variant(first), Value::$variant(second)) => {
                    MultiValues::$variant(vec![
                        value_storage_into_multi!($variant, first),
                        value_storage_into_multi!($variant, second),
                    ])
                }
            )+
            $(
                $(#[$cfg])*
                (Value::Unset(_), Value::$variant(second)) => {
                    MultiValues::$variant(vec![value_storage_into_multi!($variant, second)])
                }
            )+
            _ => unreachable!(),
        }
    };
}

/// Pushes a same-typed scalar directly into collection storage.
macro_rules! value_container_push_match {
    ($collection:expr, $value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match ($collection, $value) {
            $(
                $(#[$cfg])*
                (MultiValues::$variant(values), Value::$variant(value)) => {
                    values.push(value_storage_into_multi!($variant, value))
                },
            )+
            $(
                $(#[$cfg])*
                (slot @ MultiValues::Unset(_), Value::$variant(value)) => {
                    *slot = MultiValues::$variant(vec![value_storage_into_multi!($variant, value)]);
                }
            )+
            _ => unreachable!(),
        }
    };
}

impl From<&str> for ValueContainer {
    #[inline]
    fn from(value: &str) -> Self {
        Self::Scalar(Value::String(value.to_string()))
    }
}

impl<'a> From<Vec<&'a str>> for ValueContainer {
    #[inline]
    fn from(values: Vec<&'a str>) -> Self {
        Self::Collection(MultiValues::from(values))
    }
}

impl<'a, 'b> From<&'a [&'b str]> for ValueContainer {
    #[inline]
    fn from(values: &'a [&'b str]) -> Self {
        Self::Collection(MultiValues::from(values))
    }
}

impl<'a, 'b> From<&'a Vec<&'b str>> for ValueContainer {
    #[inline]
    fn from(values: &'a Vec<&'b str>) -> Self {
        Self::Collection(MultiValues::from(values))
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for ValueContainer {
    #[inline]
    fn from(values: [&'a str; N]) -> Self {
        Self::Collection(MultiValues::from(values))
    }
}

impl<'a, 'b, const N: usize> From<&'a [&'b str; N]> for ValueContainer {
    #[inline]
    fn from(values: &'a [&'b str; N]) -> Self {
        Self::Collection(MultiValues::from(values))
    }
}

impl From<Value> for ValueContainer {
    #[inline(always)]
    fn from(value: Value) -> Self {
        Self::Scalar(value)
    }
}

impl From<MultiValues> for ValueContainer {
    #[inline(always)]
    fn from(values: MultiValues) -> Self {
        Self::Collection(values)
    }
}

impl ValueContainer {
    /// Returns the stored or declared data type.
    ///
    /// # Returns
    ///
    /// The scalar or collection element type, including the declared type of
    /// unset storage.
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Scalar(value) => value.data_type(),
            Self::Collection(values) => values.data_type(),
        }
    }

    /// Returns whether this container has scalar shape.
    ///
    /// # Returns
    ///
    /// `true` for [`ValueContainer::Scalar`].
    #[inline(always)]
    #[must_use]
    pub const fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar(_))
    }

    /// Returns the contained scalar without consuming this container.
    ///
    /// # Returns
    ///
    /// `Some` for scalar storage, or `None` for collection storage.
    #[inline(always)]
    pub const fn as_scalar(&self) -> Option<&Value> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::Collection(_) => None,
        }
    }

    /// Consumes this container and returns its scalar value.
    ///
    /// # Returns
    ///
    /// The contained scalar value when this container has scalar shape.
    ///
    /// # Errors
    ///
    /// Returns the original container unchanged when it has collection shape.
    #[inline(always)]
    pub fn into_scalar(self) -> Result<Value, Self> {
        match self {
            Self::Scalar(value) => Ok(value),
            Self::Collection(_) => Err(self),
        }
    }

    /// Returns whether this container has collection shape.
    ///
    /// # Returns
    ///
    /// `true` for [`ValueContainer::Collection`].
    #[inline(always)]
    #[must_use]
    pub const fn is_collection(&self) -> bool {
        matches!(self, Self::Collection(_))
    }

    /// Returns the contained collection without consuming this container.
    ///
    /// # Returns
    ///
    /// `Some` for collection storage, or `None` for scalar storage.
    #[inline(always)]
    pub const fn as_collection(&self) -> Option<&MultiValues> {
        match self {
            Self::Scalar(_) => None,
            Self::Collection(values) => Some(values),
        }
    }

    /// Consumes this container and returns its collection values.
    ///
    /// # Returns
    ///
    /// The contained values when this container has collection shape.
    ///
    /// # Errors
    ///
    /// Returns the original container unchanged when it has scalar shape.
    #[inline(always)]
    pub fn into_collection(self) -> Result<MultiValues, Self> {
        match self {
            Self::Scalar(_) => Err(self),
            Self::Collection(values) => Ok(values),
        }
    }

    /// Returns whether the shape contains no concrete value or collection.
    ///
    /// # Returns
    ///
    /// `true` when the contained scalar or collection is unset.
    #[inline(always)]
    #[must_use]
    pub fn is_unset(&self) -> bool {
        match self {
            Self::Scalar(value) => value.is_unset(),
            Self::Collection(values) => values.is_unset(),
        }
    }

    /// Returns zero for unset storage, one for a concrete scalar, or the
    /// concrete collection length.
    ///
    /// # Returns
    ///
    /// The number of concrete values represented by this container.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Scalar(value) => usize::from(!value.is_unset()),
            Self::Collection(values) => values.len(),
        }
    }

    /// Reports whether this container represents no concrete values.
    ///
    /// Unset scalar and collection storage is empty. A concrete empty
    /// collection is also empty, while every concrete scalar remains non-empty,
    /// including an empty string, map, or JSON value.
    ///
    /// # Returns
    ///
    /// `true` when [`Self::len`] is zero; otherwise, `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Strictly reads a scalar or the first collection item as `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Strict target type.
    ///
    /// # Returns
    ///
    /// The scalar value or first collection item.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for unset or empty matching storage and
    /// [`ValueError::TypeMismatch`] when the stored data type differs.
    #[inline(always)]
    pub fn get_first<T>(&self) -> ValueResult<T>
    where
        T: StrictValueRead,
    {
        match self {
            Self::Scalar(value) => value.get(),
            Self::Collection(values) => values.get_first(),
        }
    }

    /// Strictly reads a scalar as a one-item list or all collection items.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Strict target element type.
    ///
    /// # Returns
    ///
    /// A one-item scalar list or all collection items.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for unset matching storage and
    /// [`ValueError::TypeMismatch`] when the stored data type differs.
    #[inline(always)]
    pub fn get_list<T>(&self) -> ValueResult<Vec<T>>
    where
        T: StrictValueListRead,
    {
        match self {
            Self::Scalar(value) => T::read_scalar_list(value),
            Self::Collection(values) => T::read_collection_list(values),
        }
    }

    /// Replaces this container, including its shape, from a supported input.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`ValueContainer`].
    ///
    /// # Parameters
    ///
    /// * `value` - New scalar or collection value.
    #[inline(always)]
    pub fn set<S>(&mut self, value: S)
    where
        S: Into<Self>,
    {
        *self = value.into();
    }

    /// Appends values, promoting scalar storage to collection storage when the
    /// input contains at least one concrete value. Same-typed empty and unset
    /// input is a no-op.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`ValueContainer`].
    ///
    /// # Parameters
    ///
    /// * `values` - Scalar or collection values to append.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending or accepting an empty same-typed input.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the appended values have a
    /// different data type.
    pub fn add<S>(&mut self, values: S) -> ValueResult<()>
    where
        S: Into<Self>,
    {
        let other = values.into();
        let expected = self.data_type();
        let actual = other.data_type();
        if expected != actual {
            return Err(ValueError::TypeMismatch { expected, actual });
        }
        if other.is_empty() {
            return Ok(());
        }

        match other {
            Self::Scalar(value) => {
                self.add_scalar(value, expected);
                Ok(())
            }
            Self::Collection(other) => match self {
                Self::Scalar(value) => {
                    let value =
                        std::mem::replace(value, Value::Unset(expected));
                    let mut collection = MultiValues::from(value);
                    collection.add(other)?;
                    *self = Self::Collection(collection);
                    Ok(())
                }
                Self::Collection(collection) => collection.add(other),
            },
        }
    }

    /// Clears concrete storage while preserving its scalar or collection
    /// shape and data type.
    #[inline(always)]
    pub fn clear(&mut self) {
        match self {
            Self::Scalar(value) => value.clear(),
            Self::Collection(values) => values.clear(),
        }
    }

    /// Removes concrete storage while preserving its shape and data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        match self {
            Self::Scalar(value) => value.unset(),
            Self::Collection(values) => values.unset(),
        }
    }

    /// Converts a scalar or the first collection item to `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Returns
    ///
    /// The converted scalar or first collection item.
    ///
    /// # Errors
    ///
    /// Returns the mapped `qubit-datatype` conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_first<T>(&self) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        self.to_first_with(DataConversionOptions::default_ref())
    }

    /// Converts a scalar or the first collection item using explicit options.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `options` - Conversion options forwarded to the contained value.
    ///
    /// # Returns
    ///
    /// The converted scalar or first collection item.
    ///
    /// # Errors
    ///
    /// Returns the mapped `qubit-datatype` conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_first_with<T>(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(value) => value.to_with(options),
            Self::Collection(values) => values.to_with(options),
        }
    }

    /// Converts a scalar to a list or converts every collection item.
    ///
    /// Scalar strings may be split according to collection conversion options;
    /// strings already stored in a collection are never split again.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target list element type.
    ///
    /// # Returns
    ///
    /// A converted scalar list or all converted collection items.
    ///
    /// # Errors
    ///
    /// Returns the mapped single-value or indexed list conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_list<T>(&self) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        self.to_list_with(DataConversionOptions::default_ref())
    }

    /// Converts to a list using explicit conversion options.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target list element type.
    ///
    /// # Parameters
    ///
    /// * `options` - Conversion options forwarded to the contained value.
    ///
    /// # Returns
    ///
    /// A converted scalar list or all converted collection items.
    ///
    /// # Errors
    ///
    /// Returns the mapped single-value or indexed list conversion error.
    #[cfg(feature = "converter")]
    pub fn to_list_with<T>(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(Value::String(value)) => {
                ScalarStringDataConverters::from(value.as_str())
                    .to_vec_with(options)
                    .map_err(ValueError::from)
            }
            Self::Scalar(value) => {
                value.to_with(options).map(|value| vec![value])
            }
            Self::Collection(values) => values.to_list_with(options),
        }
    }

    /// Appends a concrete scalar, promoting scalar storage to a collection.
    ///
    /// # Parameters
    ///
    /// * `value` - Concrete scalar to append.
    /// * `data_type` - Shared runtime type of `self` and `value`.
    ///
    /// # Constraints
    ///
    /// Callers must ensure `value` is concrete and has `data_type`; the public
    /// mutation entry points validate those invariants before calling this
    /// helper.
    #[inline]
    fn add_scalar(&mut self, value: Value, data_type: DataType) {
        match self {
            Self::Scalar(current) => {
                let current =
                    std::mem::replace(current, Value::Unset(data_type));
                let collection = for_each_value_type!(
                    value_container_pair_match,
                    current,
                    value
                );
                *self = Self::Collection(collection);
            }
            Self::Collection(collection) => {
                for_each_value_type!(
                    value_container_push_match,
                    collection,
                    value
                );
            }
        }
    }
}
