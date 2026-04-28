use super::multi_values::MultiValues;
use crate::value_error::ValueResult;

/// Internal dispatch trait for `MultiValues::add<S>()`.
///
/// Implementations route `T`, `Vec<T>`, and `&[T]` to the matching add path.
#[doc(hidden)]
pub trait MultiValuesAddArg<'a> {
    /// Element type being added.
    type Item: 'a + Clone;

    /// Applies this add argument to `target`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the target is updated, or a `ValueError` from the
    /// selected add path.
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()>;
}
