/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use crate::value_error::ValueResult;

/// Internal trait used to set a single value in `MultiValues`.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesSingleSetter<T> {
    /// Replaces the stored values with one `value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_single_value(&mut self, value: T) -> ValueResult<()>;
}
