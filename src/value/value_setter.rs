use crate::value_error::ValueResult;

/// Internal trait used to set specific types in `Value`.
///
/// This trait backs `Value::set<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueSetter<T> {
    /// Replaces the stored value with `value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be stored, or a `ValueError` when
    /// the implementation rejects the value.
    fn set_value(&mut self, value: T) -> ValueResult<()>;
}
