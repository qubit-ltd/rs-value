// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private sealing marker for strict runtime reads.

use crate::MultiValues;
use crate::Value;
use crate::ValueError;

/// Prevents downstream crates from implementing `StrictValueRead`.
///
/// The blanket implementation admits exactly the target types supported by
/// strict scalar and collection conversions.
pub trait Sealed {}

impl<T> Sealed for T
where
    for<'a> T: TryFrom<&'a Value, Error = ValueError> + TryFrom<&'a MultiValues, Error = ValueError>,
    for<'a> Vec<T>: TryFrom<&'a MultiValues, Error = ValueError>,
{
}
