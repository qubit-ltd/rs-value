use crate::value_error::ValueResult;

/// Internal trait used to extract the first value from `MultiValues`.
///
/// This trait backs `MultiValues::get_first<T>()`; downstream code should call
/// the inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesFirstGetter<T> {
    /// Gets the first stored value as `T`.
    ///
    /// # Returns
    ///
    /// Returns the first value when present and typed as `T`, or a `ValueError`
    /// describing the missing value or mismatch.
    fn get_first_value(&self) -> ValueResult<T>;
}
