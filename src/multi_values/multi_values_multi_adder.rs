use crate::value_error::ValueResult;

/// Internal trait used to add multiple values to `MultiValues`.
///
/// This trait backs `MultiValues::add<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesMultiAdder<T> {
    /// Appends all values from `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be appended, or a `ValueError`
    /// describing the type mismatch.
    fn add_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}
