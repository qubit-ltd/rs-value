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
    /// # Arguments
    ///
    /// * `other` - Right numeric operand.
    /// * `policy` - Exact or approximate numeric comparison policy.
    ///
    /// # Returns
    ///
    /// The mathematical ordering of both numeric operands.
    ///
    /// # Errors
    ///
    /// Returns [`NumericComparisonError::LeftNotNumeric`] or
    /// [`NumericComparisonError::RightNotNumeric`] for non-numeric and unset
    /// operands, and [`NumericComparisonError::UnorderedNaN`] for NaN.
    pub fn numeric_cmp(
        &self,
        other: &Self,
        policy: NumericComparisonPolicy,
    ) -> Result<Ordering, NumericComparisonError> {
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
        compare_numeric(left, right, policy)
            .ok_or(NumericComparisonError::UnorderedNaN)
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
}
