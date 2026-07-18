// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-driven numeric comparison for [`super::Value`].

use std::cmp::Ordering;

use qubit_datatype::{NumberRef, NumericComparisonPolicy};

use super::Value;
use crate::NumericComparisonError;

impl Value {
    /// Tests whether this value is a concrete floating-point NaN.
    ///
    /// Non-floating-point values and unset values return `false`.
    ///
    /// # Returns
    ///
    /// `true` only for concrete `Float32` or `Float64` NaN values.
    #[inline(always)]
    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.as_number_ref().is_some_and(|value| value.is_nan())
    }

    /// Compares concrete numeric values across representation variants.
    ///
    /// This operation is separate from [`PartialEq`]: equality preserves enum
    /// representation identity, while numeric comparison compares mathematical
    /// values under an explicit policy.
    ///
    /// [`NumericComparisonPolicy::Approximate`] orders primitive infinities
    /// separately. When a finite primitive float participates, it attempts to
    /// project both operands to finite `f64` values; if either operand cannot
    /// be projected that way, comparison falls back to the exact path.
    /// Projected comparison is pair-dependent and not transitive across
    /// mixed representations. Do not use it to implement [`Ord`], sort or
    /// group values, or construct ordered-map or ordered-set keys. Use
    /// [`NumericComparisonPolicy::Exact`] for deterministic ordering.
    ///
    /// Validation is deterministic: missing operands are checked from left to
    /// right, followed by concrete operand types from left to right, and then
    /// NaN positions.
    ///
    /// # Parameters
    ///
    /// * `other` - Right numeric operand.
    /// * `policy` - Exact or approximate numeric comparison policy.
    ///
    /// # Returns
    ///
    /// The mathematical ordering of the two concrete, non-NaN numeric
    /// operands.
    ///
    /// # Errors
    ///
    /// Returns [`NumericComparisonError::LeftMissing`] or
    /// [`NumericComparisonError::RightMissing`] when the corresponding operand
    /// is unset. Returns [`NumericComparisonError::LeftNotNumeric`] or
    /// [`NumericComparisonError::RightNotNumeric`] when the corresponding
    /// concrete operand is not numeric. Returns
    /// [`NumericComparisonError::LeftNaN`],
    /// [`NumericComparisonError::RightNaN`], or
    /// [`NumericComparisonError::BothNaN`] according to the position of NaN
    /// operands. Returns [`NumericComparisonError::Indeterminate`] if the
    /// lower-level comparator cannot order the validated numeric operands.
    /// Missing operands are checked left-to-right, then concrete operand types
    /// are checked left-to-right, and finally NaN positions are classified.
    pub fn numeric_cmp(
        &self,
        other: &Self,
        policy: NumericComparisonPolicy,
    ) -> Result<Ordering, NumericComparisonError> {
        if let Self::Unset(declared) = self {
            return Err(NumericComparisonError::LeftMissing {
                declared: *declared,
            });
        }
        if let Self::Unset(declared) = other {
            return Err(NumericComparisonError::RightMissing {
                declared: *declared,
            });
        }

        let left = self
            .as_number_ref()
            .ok_or_else(|| NumericComparisonError::LeftNotNumeric {
                actual: self.data_type(),
            })?;
        let right =
            other
                .as_number_ref()
                .ok_or_else(|| NumericComparisonError::RightNotNumeric {
                    actual: other.data_type(),
                })?;

        match (left.is_nan(), right.is_nan()) {
            (true, true) => return Err(NumericComparisonError::BothNaN),
            (true, false) => return Err(NumericComparisonError::LeftNaN),
            (false, true) => return Err(NumericComparisonError::RightNaN),
            (false, false) => {}
        }

        left.compare_to(right, policy)
            .ok_or_else(|| NumericComparisonError::Indeterminate {
                left: self.data_type(),
                right: other.data_type(),
            })
    }

    /// Borrows this value as a lower-level numeric representation.
    ///
    /// # Returns
    ///
    /// A borrowed numeric representation for every concrete numeric variant,
    /// or `None` for unset and non-numeric variants.
    fn as_number_ref(&self) -> Option<NumberRef<'_>> {
        match self {
            Self::Int8(value) => Some(NumberRef::from(*value)),
            Self::Int16(value) => Some(NumberRef::from(*value)),
            Self::Int32(value) => Some(NumberRef::from(*value)),
            Self::Int64(value) => Some(NumberRef::from(*value)),
            Self::Int128(value) => Some(NumberRef::from(*value)),
            Self::UInt8(value) => Some(NumberRef::from(*value)),
            Self::UInt16(value) => Some(NumberRef::from(*value)),
            Self::UInt32(value) => Some(NumberRef::from(*value)),
            Self::UInt64(value) => Some(NumberRef::from(*value)),
            Self::UInt128(value) => Some(NumberRef::from(*value)),
            Self::Float32(value) => Some(NumberRef::from(*value)),
            Self::Float64(value) => Some(NumberRef::from(*value)),
            #[cfg(feature = "big-integer")]
            Self::BigInteger(value) => Some(NumberRef::from(value)),
            #[cfg(feature = "big-decimal")]
            Self::BigDecimal(value) => Some(NumberRef::from(value)),
            _ => None,
        }
    }
}
