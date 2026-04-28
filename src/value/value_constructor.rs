/// Internal trait used to create `Value` from supported Rust types.
///
/// This trait backs `Value::new<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueConstructor<T> {
    /// Builds a `Value` that wraps `value`.
    ///
    /// # Returns
    ///
    /// Returns the enum variant corresponding to `T`.
    fn from_type(value: T) -> Self;
}
