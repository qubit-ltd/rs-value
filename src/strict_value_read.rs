// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Sealed public bound for strict reads from runtime value storage.
use crate::MultiValues;
use crate::Value;
use crate::ValueError;
use crate::ValueResult;

mod internal;

use self::internal::sealed::Sealed;

/// Marks target types supported by exact, non-converting reads.
///
/// This trait is sealed because supported types are determined by the closed
/// runtime [`qubit_datatype::DataType`] family. Domain conversions belong in
/// explicit conversion boundaries, rather than changing strict-read semantics.
///
/// # Examples
///
/// ```
/// use qubit_value::{StrictValueRead, Value, ValueResult};
///
/// fn read_exact<T: StrictValueRead>(value: &Value) -> ValueResult<T> {
///     T::read_scalar(value)
/// }
///
/// assert_eq!(read_exact::<i32>(&Value::from(42_i32)).unwrap(), 42);
/// ```
pub trait StrictValueRead: Sized + Sealed {
    /// Strictly reads a scalar runtime value.
    ///
    /// # Parameters
    ///
    /// * `value` - Scalar runtime value to read without conversion.
    ///
    /// # Returns
    ///
    /// The exact stored scalar represented as `Self`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] for unset storage or
    /// [`ValueError::TypeMismatch`] for a different runtime type.
    #[doc(hidden)]
    fn read_scalar(value: &Value) -> ValueResult<Self>;

    /// Strictly reads the first item from a runtime collection.
    ///
    /// # Parameters
    ///
    /// * `values` - Runtime collection whose first item is read exactly.
    ///
    /// # Returns
    ///
    /// The first stored element represented as `Self`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] for unset or empty storage, or
    /// [`ValueError::TypeMismatch`] for a different element type.
    #[doc(hidden)]
    fn read_collection_first(values: &MultiValues) -> ValueResult<Self>;

    /// Strictly reads every item from a runtime collection.
    ///
    /// # Parameters
    ///
    /// * `values` - Runtime collection whose elements are cloned exactly.
    ///
    /// # Returns
    ///
    /// Every stored element in original order.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] for unset storage or
    /// [`ValueError::TypeMismatch`] for a different element type.
    #[doc(hidden)]
    fn read_collection_list(values: &MultiValues) -> ValueResult<Vec<Self>>;
}

impl<T> StrictValueRead for T
where
    for<'a> T: TryFrom<&'a Value, Error = ValueError> + TryFrom<&'a MultiValues, Error = ValueError>,
    for<'a> Vec<T>: TryFrom<&'a MultiValues, Error = ValueError>,
{
    #[inline(always)]
    fn read_scalar(value: &Value) -> ValueResult<Self> {
        value.get()
    }

    #[inline(always)]
    fn read_collection_first(values: &MultiValues) -> ValueResult<Self> {
        values.get_first()
    }

    #[inline(always)]
    fn read_collection_list(values: &MultiValues) -> ValueResult<Vec<Self>> {
        values.get()
    }
}
