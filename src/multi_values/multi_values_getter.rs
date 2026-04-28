use crate::value_error::ValueResult;

/// Internal trait used to extract multiple values from `MultiValues`.
///
/// This trait backs `MultiValues::get<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesGetter<T> {
    /// Gets all stored values as `Vec<T>`.
    ///
    /// # Returns
    ///
    /// Returns a cloned vector when the stored variant matches `T`, or a
    /// `ValueError` describing the mismatch.
    fn get_values(&self) -> ValueResult<Vec<T>>;
}
