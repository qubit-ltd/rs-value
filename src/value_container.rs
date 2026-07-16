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
    DataConvertTo,
    DataConverter,
    DataTypeOf,
    ScalarStringDataConverters,
};

/// A typed value whose scalar or collection shape is explicit.
///
/// The shape is never inferred from collection length. In particular,
/// `Scalar(Value::Int32(42))` and
/// `Collection(MultiValues::Int32(vec![42]))` remain distinguishable through
/// conversion and serialization boundaries.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueContainer {
    /// One typed value.
    Scalar(Value),
    /// A homogeneous typed collection.
    Collection(MultiValues),
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
                    Self::Scalar(Value::$variant(value))
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
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Scalar(value) => value.data_type(),
            Self::Collection(values) => values.data_type(),
        }
    }

    /// Returns whether this container has scalar shape.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline]
    pub fn is_unset(&self) -> bool {
        match self {
            Self::Scalar(value) => value.is_unset(),
            Self::Collection(values) => values.is_unset(),
        }
    }

    /// Returns zero for unset storage, one for a concrete scalar, or the
    /// concrete collection length.
    #[inline(always)]
    pub fn count(&self) -> usize {
        match self {
            Self::Scalar(value) => usize::from(!value.is_unset()),
            Self::Collection(values) => values.count(),
        }
    }

    /// Strictly reads a scalar or the first collection item as `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for unset or empty matching storage and
    /// [`ValueError::TypeMismatch`] when the stored data type differs.
    #[inline(always)]
    pub fn get<T>(&self) -> ValueResult<T>
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
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the appended values have a
    /// different data type.
    pub fn add<S>(&mut self, values: S) -> ValueResult<()>
    where
        S: Into<Self>,
    {
        let other = match values.into() {
            Self::Scalar(value) => MultiValues::from(value),
            Self::Collection(values) => values,
        };
        let expected = self.data_type();
        let actual = other.data_type();
        if expected != actual {
            return Err(ValueError::TypeMismatch { expected, actual });
        }
        if other.count() == 0 {
            return Ok(());
        }

        match self {
            Self::Scalar(value) => {
                let value = std::mem::replace(value, Value::Unset(expected));
                let mut collection = MultiValues::from(value);
                collection.add(other)?;
                *self = Self::Collection(collection);
                Ok(())
            }
            Self::Collection(collection) => collection.add(other),
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
    /// # Errors
    ///
    /// Returns the mapped `qubit-datatype` conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        self.to_with(DataConversionOptions::default_ref())
    }

    /// Converts a scalar or the first collection item using explicit options.
    ///
    /// # Errors
    ///
    /// Returns the mapped `qubit-datatype` conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
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
    /// # Errors
    ///
    /// Returns the mapped single-value or indexed list conversion error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_list<T>(&self) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        self.to_list_with(DataConversionOptions::default_ref())
    }

    /// Converts to a list using explicit conversion options.
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
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
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
}
