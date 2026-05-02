/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use crate::value_error::ValueResult;

/// Internal trait used to convert `Value` to target types.
///
/// This trait powers `Value::to<T>()`. Each implementation must clearly define
/// which source variants are accepted for the target type.
#[doc(hidden)]
pub trait ValueConverter<T>: super::sealed::ValueConverterSealed<T> {
    /// Converts the current value to `T`.
    ///
    /// # Returns
    ///
    /// Returns the converted value when the conversion is supported, or a
    /// `ValueError` with conversion context otherwise.
    fn convert(&self) -> ValueResult<T>;
}
