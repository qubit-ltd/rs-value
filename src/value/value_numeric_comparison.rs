// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-driven numeric comparison for [`super::Value`].

use std::cmp::Ordering;

use qubit_datatype::{
    NumericComparisonPolicy,
    NumericValueRef,
    compare_numeric,
};

use super::Value;
use crate::NumericComparisonError;

impl Value {
    /// Compares concrete numeric values across representation variants.
    ///
    /// This operation is separate from [`PartialEq`]: equality preserves enum
    /// representation identity, while numeric comparison compares mathematical
    /// values under an explicit policy.
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
    /// operands. Missing operands are checked left-to-right, then concrete
    /// operand types are checked left-to-right, and finally NaN positions are
    /// classified.
    ///
    /// # Panics
    ///
    /// Panics if the lower-level comparator cannot order two concrete,
    /// non-NaN numeric representations, which would violate its contract.
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

        let left = self.as_numeric_ref().ok_or_else(|| {
            NumericComparisonError::LeftNotNumeric {
                actual: self.data_type(),
            }
        })?;
        let right = other.as_numeric_ref().ok_or_else(|| {
            NumericComparisonError::RightNotNumeric {
                actual: other.data_type(),
            }
        })?;

        match (self.is_nan_numeric(), other.is_nan_numeric()) {
            (true, true) => return Err(NumericComparisonError::BothNaN),
            (true, false) => return Err(NumericComparisonError::LeftNaN),
            (false, true) => return Err(NumericComparisonError::RightNaN),
            (false, false) => {}
        }

        Ok(compare_numeric(left, right, policy)
            .expect("concrete non-NaN NumericValueRef values must be ordered"))
    }

    /// Borrows this value as a lower-level numeric representation.
    ///
    /// # Returns
    ///
    /// A borrowed numeric representation for every concrete numeric variant,
    /// or `None` for unset and non-numeric variants.
    #[inline]
    fn as_numeric_ref(&self) -> Option<NumericValueRef<'_>> {
        match self {
            Self::Int8(value) => Some(NumericValueRef::Int8(*value)),
            Self::Int16(value) => Some(NumericValueRef::Int16(*value)),
            Self::Int32(value) => Some(NumericValueRef::Int32(*value)),
            Self::Int64(value) => Some(NumericValueRef::Int64(*value)),
            Self::Int128(value) => Some(NumericValueRef::Int128(*value)),
            Self::UInt8(value) => Some(NumericValueRef::UInt8(*value)),
            Self::UInt16(value) => Some(NumericValueRef::UInt16(*value)),
            Self::UInt32(value) => Some(NumericValueRef::UInt32(*value)),
            Self::UInt64(value) => Some(NumericValueRef::UInt64(*value)),
            Self::UInt128(value) => Some(NumericValueRef::UInt128(*value)),
            Self::Float32(value) => Some(NumericValueRef::Float32(*value)),
            Self::Float64(value) => Some(NumericValueRef::Float64(*value)),
            #[cfg(feature = "big-number")]
            Self::BigInteger(value) => Some(NumericValueRef::BigInteger(value)),
            #[cfg(feature = "big-number")]
            Self::BigDecimal(value) => Some(NumericValueRef::BigDecimal(value)),
            _ => None,
        }
    }

    /// Reports whether this concrete numeric value is NaN.
    ///
    /// # Returns
    ///
    /// `true` only for primitive floating-point NaN variants.
    #[inline(always)]
    fn is_nan_numeric(&self) -> bool {
        match self {
            Self::Float32(value) => value.is_nan(),
            Self::Float64(value) => value.is_nan(),
            _ => false,
        }
    }
}
