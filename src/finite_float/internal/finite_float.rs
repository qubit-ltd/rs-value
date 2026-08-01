// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared finite-value predicate for supported floating-point widths.

/// Provides a width-independent finiteness check for float adapters.
pub(in crate::finite_float) trait FiniteFloat:
    Copy
{
    /// Reports whether this value is neither NaN nor positive or negative
    /// infinity.
    ///
    /// # Returns
    ///
    /// `true` for finite values; otherwise, `false`.
    #[must_use]
    fn is_finite(self) -> bool;
}

impl FiniteFloat for f32 {
    /// Delegates to [`f32::is_finite`].
    #[inline(always)]
    fn is_finite(self) -> bool {
        self.is_finite()
    }
}

impl FiniteFloat for f64 {
    /// Delegates to [`f64::is_finite`].
    #[inline(always)]
    fn is_finite(self) -> bool {
        self.is_finite()
    }
}
