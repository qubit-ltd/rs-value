// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public bound for strict list reads from either runtime value shape.

use crate::{MultiValues, Value, ValueError, ValueResult};

mod sealed {
    use super::{MultiValues, Value, ValueError};

    pub trait Sealed {}

    impl<T> Sealed for T
    where
        for<'a> T: TryFrom<&'a Value, Error = ValueError>,
        for<'a> Vec<T>: TryFrom<&'a MultiValues, Error = ValueError>,
    {
    }
}

/// Marks element types that can be read strictly as a list from either shape.
///
/// Scalars produce one-item lists, while collections preserve all items. Both
/// paths retain exact stored-type checks and never perform data conversion.
pub trait StrictValueListRead: Sized + sealed::Sealed {
    /// Strictly reads one scalar as a one-item list.
    ///
    /// # Parameters
    ///
    /// * `value` - Scalar runtime value to read.
    ///
    /// # Returns
    ///
    /// A one-item list containing the exact typed scalar.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for unset storage and
    /// [`ValueError::TypeMismatch`] for a different stored type.
    #[doc(hidden)]
    fn read_scalar_list(value: &Value) -> ValueResult<Vec<Self>>;

    /// Strictly reads every item from a homogeneous collection.
    ///
    /// # Parameters
    ///
    /// * `values` - Runtime collection to read.
    ///
    /// # Returns
    ///
    /// All exact typed collection items.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] for unset storage and
    /// [`ValueError::TypeMismatch`] for a different stored type.
    #[doc(hidden)]
    fn read_collection_list(values: &MultiValues) -> ValueResult<Vec<Self>>;
}

impl<T> StrictValueListRead for T
where
    for<'a> T: TryFrom<&'a Value, Error = ValueError>,
    for<'a> Vec<T>: TryFrom<&'a MultiValues, Error = ValueError>,
{
    #[inline(always)]
    fn read_scalar_list(value: &Value) -> ValueResult<Vec<Self>> {
        value.get().map(|value| vec![value])
    }

    #[inline(always)]
    fn read_collection_list(values: &MultiValues) -> ValueResult<Vec<Self>> {
        values.get()
    }
}
