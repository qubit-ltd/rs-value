/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use super::multi_values::MultiValues;
use crate::value_error::ValueResult;

/// Internal dispatch trait for `MultiValues::set<S>()`.
///
/// Implementations route `Vec<T>`, `&[T]`, and `T` to the matching set path.
#[doc(hidden)]
pub trait MultiValuesSetArg<'a> {
    /// Element type being set.
    type Item: 'a + Clone;

    /// Applies this argument to `target`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the target is updated, or a `ValueError` from the
    /// selected set path.
    fn apply(self, target: &mut MultiValues) -> ValueResult<()>;
}
