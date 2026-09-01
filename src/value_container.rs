// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit scalar-or-collection value storage.

#[cfg(feature = "converter")]
use qubit_datatype::ConversionLimits;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionPolicy;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionSession;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataType;
#[cfg(feature = "converter")]
use qubit_datatype::ScalarStringDataConverters;

use crate::MultiValues;
use crate::StrictValueRead;
use crate::Value;
use crate::ValueError;
use crate::ValueResult;
use crate::multi_values::MultiValuesRepr;
#[cfg(feature = "converter")]
use crate::value::ValueRef;
use crate::value::ValueRepr;

/// A typed value whose scalar or collection shape is explicit.
///
/// The shape is never inferred from collection length. In particular,
/// `Scalar(Value::Int32(42))` and
/// `Collection(MultiValues::Int32(vec![42]))` remain distinguishable through
/// conversion and serialization boundaries.
///
/// # Examples
///
/// ```
/// use qubit_value::ValueContainer;
///
/// let scalar = ValueContainer::from(42_i32);
/// let collection = ValueContainer::from(vec![42_i32]);
/// assert!(scalar.is_scalar());
/// assert!(collection.is_collection());
/// ```
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

/// Implements scalar and collection conversions from the shared value table.
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
                $(, $_wire:tt)*
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

/// Builds a typed collection from one or two same-typed scalar values.
macro_rules! value_container_pair_match {
    ($first:expr, $second:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match ($first.repr, $second.repr) {
            $(
                $(#[$cfg])*
                (ValueRepr::$variant(first), ValueRepr::$variant(second)) => {
                    MultiValues::$variant(vec![
                        value_storage_into_multi!($variant, first),
                        value_storage_into_multi!($variant, second),
                    ])
                }
            )+
            $(
                $(#[$cfg])*
                (ValueRepr::Unset(_), ValueRepr::$variant(second)) => {
                    MultiValues::$variant(vec![value_storage_into_multi!($variant, second)])
                }
            )+
            _ => unreachable!(),
        }
    };
}

/// Pushes a same-typed scalar directly into collection storage.
macro_rules! value_container_push_match {
    ($collection:expr, $value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match (&mut $collection.repr, $value.repr) {
            $(
                $(#[$cfg])*
                (MultiValuesRepr::$variant(values), ValueRepr::$variant(value)) => {
                    values.push(value_storage_into_multi!($variant, value))
                },
            )+
            $(
                $(#[$cfg])*
                (slot @ MultiValuesRepr::Unset(_), ValueRepr::$variant(value)) => {
                    *slot = MultiValuesRepr::$variant(vec![value_storage_into_multi!($variant, value)]);
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
    /// Creates an unset scalar container for a declared data type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - Declared scalar type while no value is set.
    ///
    /// # Returns
    ///
    /// A scalar `ValueContainer` with explicit unset storage.
    #[inline(always)]
    pub const fn new_unset_scalar(data_type: DataType) -> Self {
        Self::Scalar(Value::new_unset(data_type))
    }

    /// Creates an unset collection container for a declared element type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - Declared element type while no values are set.
    ///
    /// # Returns
    ///
    /// A collection `ValueContainer` with explicit unset storage.
    #[inline(always)]
    pub const fn new_unset_collection(data_type: DataType) -> Self {
        Self::Collection(MultiValues::new_unset(data_type))
    }

    /// Returns the stored or declared data type.
    ///
    /// # Returns
    ///
    /// The scalar or collection element type, including the declared type of
    /// unset storage.
    #[must_use = "the runtime data type should be used"]
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
    #[must_use]
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
    #[must_use]
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
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Self::Scalar(value) => usize::from(!value.is_unset()),
            Self::Collection(values) => values.len(),
        }
    }

    /// Tests whether this container represents no concrete values.
    ///
    /// # Returns
    ///
    /// `true` for unset storage or a concrete empty collection.
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
    /// Returns [`ValueError::Missing`] for unset or empty matching storage and
    /// [`ValueError::TypeMismatch`] when the stored data type differs.
    #[must_use = "the strict first-value result should be handled"]
    #[inline(always)]
    pub fn get_first<T>(&self) -> ValueResult<T>
    where
        T: StrictValueRead,
    {
        match self {
            Self::Scalar(value) => T::read_scalar(value),
            Self::Collection(values) => T::read_collection_first(values),
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
    /// Returns [`ValueError::Missing`] for unset matching storage and
    /// [`ValueError::TypeMismatch`] when the stored data type differs.
    #[must_use = "the strict collection read result should be handled"]
    #[inline(always)]
    pub fn get_list<T>(&self) -> ValueResult<Vec<T>>
    where
        T: StrictValueRead,
    {
        match self {
            Self::Scalar(value) => T::read_scalar(value).map(|item| vec![item]),
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
                    let value = std::mem::replace(value, Value::new_unset(expected));
                    let mut collection = MultiValues::from(value);
                    collection.add(other)?;
                    *self = Self::Collection(collection);
                    Ok(())
                }
                Self::Collection(collection) => collection.add(other),
            },
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
        self.to_first_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Converts a scalar or the first collection item using explicit policy and
    /// limits.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `policy` - Conversion policy forwarded to the contained value.
    /// * `limits` - Conversion limits forwarded to the contained value.
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
    pub fn to_first_with<T>(&self, policy: &ConversionPolicy, limits: &ConversionLimits) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(value) => value.to_with(policy, limits),
            Self::Collection(values) => values.to_first_with(policy, limits),
        }
    }

    /// Converts the scalar or first collection item using an existing
    /// conversion session.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `session` - Caller-owned session providing policy, limits, and budget.
    ///
    /// # Returns
    ///
    /// The converted scalar or first collection item.
    ///
    /// # Errors
    ///
    /// Returns the mapped missing, conversion, or budget error.
    #[cfg(feature = "converter")]
    #[inline(always)]
    pub fn to_first_in<T>(&self, session: &mut ConversionSession<'_>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(value) => value.to_in(session),
            Self::Collection(values) => values.to_first_in(session),
        }
    }

    /// Converts a scalar to a list or converts every collection item.
    ///
    /// Scalar strings may be split according to collection conversion policy;
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
        self.to_list_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Converts to a list using explicit conversion policy and limits.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target list element type.
    ///
    /// # Parameters
    ///
    /// * `policy` - Conversion policy forwarded to the contained value.
    /// * `limits` - Conversion limits forwarded to the contained value.
    ///
    /// # Returns
    ///
    /// A converted scalar list or all converted collection items.
    ///
    /// # Errors
    ///
    /// Returns the mapped single-value or indexed list conversion error.
    #[cfg(feature = "converter")]
    pub fn to_list_with<T>(&self, policy: &ConversionPolicy, limits: &ConversionLimits) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(value) => match value.view() {
                ValueRef::String(value) => ScalarStringDataConverters::from(value)
                    .to_vec_with(policy, limits)
                    .map_err(ValueError::from),
                _ => value.to_with(policy, limits).map(|value| vec![value]),
            },
            Self::Collection(values) => values.to_list_with(policy, limits),
        }
    }

    /// Converts this container to a list using an existing conversion session.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target list element type.
    ///
    /// # Parameters
    ///
    /// * `session` - Caller-owned session providing policy, limits, and budget.
    ///
    /// # Returns
    ///
    /// A converted scalar list or all converted collection items.
    ///
    /// # Errors
    ///
    /// Returns the mapped missing, conversion, indexed-list, or budget error.
    #[cfg(feature = "converter")]
    pub fn to_list_in<T>(&self, session: &mut ConversionSession<'_>) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self {
            Self::Scalar(value) => match value.view() {
                ValueRef::String(value) => ScalarStringDataConverters::from(value)
                    .to_vec_in(session)
                    .map_err(ValueError::from),
                _ => value.to_in(session).map(|value| vec![value]),
            },
            Self::Collection(values) => values.to_list_in(session),
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
                let current = std::mem::replace(current, Value::new_unset(data_type));
                let collection = for_each_value_type!(value_container_pair_match, current, value);
                *self = Self::Collection(collection);
            }
            Self::Collection(collection) => {
                for_each_value_type!(value_container_push_match, collection, value);
            }
        }
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
impl ValueContainer {
    /// Projects this container while preserving concrete collection shape.
    ///
    /// Scalar storage uses the natural scalar projection; concrete collection
    /// storage always uses a JSON array.
    ///
    /// # Returns
    ///
    /// The natural JSON representation, except scalar and collection unset
    /// values both project to `null`.
    ///
    /// # Errors
    ///
    /// Returns the same structured projection error as the contained value.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<serde_json::Value> {
        self.to_json_value_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Projects this container using explicit conversion policy and limits.
    ///
    /// # Parameters
    ///
    /// * `policy` - Controls duration units and precision-loss behavior.
    /// * `limits` - Bounds conversion resource consumption.
    ///
    /// # Returns
    ///
    /// The natural JSON representation, except scalar and collection unset
    /// values both project to `null`.
    ///
    /// # Errors
    ///
    /// Returns the same structured projection error as the contained value.
    #[inline(always)]
    pub fn to_json_value_with(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<serde_json::Value> {
        crate::json::value_container_to_json_value_with(self, policy, limits)
    }
}
