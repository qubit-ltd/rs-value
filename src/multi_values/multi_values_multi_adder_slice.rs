use crate::value_error::ValueResult;

/// Internal trait used to append multiple values from a slice.
///
/// This trait backs slice arguments to `MultiValues::add<S>()`; downstream code
/// should call the inherent method instead of implementing or naming this trait
/// directly.
#[doc(hidden)]
pub(crate) trait MultiValuesMultiAdderSlice<T> {
    /// Appends all values from `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be appended, or a `ValueError`
    /// describing the type mismatch.
    fn add_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}
