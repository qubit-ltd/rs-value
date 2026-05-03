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

use super::value::Value;

/// Private marker trait used to prevent downstream implementations.
trait ValueConverterSealed<T> {}

impl<T> ValueConverterSealed<T> for Value {}

/// Internal trait used to convert `Value` to target types.
///
/// This trait powers `Value::to<T>()`. Each implementation must clearly define
/// which source variants are accepted for the target type.
#[allow(private_bounds)]
pub trait ValueConverter<T>: ValueConverterSealed<T> {
    /// Converts the current value to `T`.
    ///
    /// # Returns
    ///
    /// Returns the converted value when the conversion is supported, or a
    /// `ValueError` with conversion context otherwise.
    fn convert(&self) -> ValueResult<T>;
}
