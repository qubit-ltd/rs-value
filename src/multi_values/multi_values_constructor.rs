/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

/// Internal trait used to create `MultiValues` from `Vec<T>`.
///
/// This trait backs `MultiValues::new<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesConstructor<T> {
    /// Builds a `MultiValues` instance from `values`.
    ///
    /// # Returns
    ///
    /// Returns the enum variant corresponding to `T`.
    fn from_vec(values: Vec<T>) -> Self;
}
