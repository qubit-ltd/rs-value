/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use crate::value_error::ValueResult;

/// Internal trait used to extract specific types from `Value`.
///
/// This trait backs `Value::get<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueGetter<T> {
    /// Gets the value as `T`.
    ///
    /// # Returns
    ///
    /// Returns the typed value when the stored variant matches `T`, or a
    /// `ValueError` describing the mismatch or missing value.
    fn get_value(&self) -> ValueResult<T>;
}
