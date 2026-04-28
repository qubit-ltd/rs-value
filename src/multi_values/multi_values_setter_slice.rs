use crate::value_error::ValueResult;

/// Internal trait used to set `MultiValues` from a slice.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesSetterSlice<T> {
    /// Replaces the stored values with a clone of `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}
