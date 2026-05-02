/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use crate::value_error::ValueResult;

/// Internal trait used to set specific value lists in `MultiValues`.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesSetter<T> {
    /// Replaces the stored values with `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}
