/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use crate::value_error::ValueResult;

/// Internal trait used to add a single value to `MultiValues`.
///
/// This trait backs `MultiValues::add<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesAdder<T> {
    /// Appends `value` to the stored values.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be appended, or a `ValueError`
    /// describing the type mismatch.
    fn add_value(&mut self, value: T) -> ValueResult<()>;
}
