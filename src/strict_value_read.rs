// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Sealed public bound for strict reads from runtime value storage.
// qubit-style: allow multiple-public-types

use crate::MultiValues;
use crate::Value;
use crate::ValueError;
use crate::ValueResult;

/// Private sealing support for [`StrictValueRead`].
mod sealed {
    use super::MultiValues;
    use super::Value;
    use super::ValueError;

    /// Prevents downstream crates from implementing [`super::StrictValueRead`].
    pub trait Sealed {}

    impl<T> Sealed for T
    where
        for<'a> T: TryFrom<&'a Value, Error = ValueError> + TryFrom<&'a MultiValues, Error = ValueError>,
        for<'a> Vec<T>: TryFrom<&'a MultiValues, Error = ValueError>,
    {
    }
}

/// Marks target types supported by exact, non-converting reads.
///
/// This trait is sealed because supported types are determined by the closed
/// runtime [`qubit_datatype::DataType`] family. Domain conversions belong in
/// explicit conversion boundaries, rather than changing strict-read semantics.
pub trait StrictValueRead: Sized + sealed::Sealed {
    /// Strictly reads a scalar runtime value.
    #[doc(hidden)]
    fn read_scalar(value: &Value) -> ValueResult<Self>;

    /// Strictly reads the first item from a runtime collection.
    #[doc(hidden)]
    fn read_collection_first(values: &MultiValues) -> ValueResult<Self>;

    /// Strictly reads every item from a runtime collection.
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
