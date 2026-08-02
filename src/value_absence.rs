// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structured reason why a runtime value has no concrete item to read.
// qubit-style: allow source-test-pair
// Tests are intentionally distributed across value_error_tests.rs and
// behavior-specific files under tests/value/ rather than collected in
// value_absence_tests.rs.

use std::fmt;

use qubit_datatype::DataType;

/// Describes the typed storage state that produced a missing-value error.
///
/// An unset scalar, an unset collection, and a concrete empty collection are
/// observably different states. This type preserves that distinction in
/// [`crate::ValueError::NoValue`] without exposing storage internals.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueAbsence {
    /// An unset scalar with a declared data type.
    UnsetScalar {
        /// Type retained by the unset scalar storage.
        data_type: DataType,
    },
    /// An unset collection with a declared element type.
    UnsetCollection {
        /// Element type retained by the unset collection storage.
        data_type: DataType,
    },
    /// A concrete collection that contains no item.
    EmptyCollection {
        /// Element type of the concrete empty collection.
        data_type: DataType,
    },
}

impl ValueAbsence {
    /// Returns the declared scalar or collection element type.
    #[inline(always)]
    pub const fn data_type(self) -> DataType {
        match self {
            Self::UnsetScalar { data_type }
            | Self::UnsetCollection { data_type }
            | Self::EmptyCollection { data_type } => data_type,
        }
    }

    /// Returns whether storage is unset rather than a concrete empty
    /// collection.
    #[inline(always)]
    #[must_use]
    pub const fn is_unset(self) -> bool {
        matches!(
            self,
            Self::UnsetScalar { .. } | Self::UnsetCollection { .. }
        )
    }

    /// Returns whether a concrete collection has no item to read.
    #[inline(always)]
    #[must_use]
    pub const fn is_empty_collection(self) -> bool {
        matches!(self, Self::EmptyCollection { .. })
    }
}

impl fmt::Display for ValueAbsence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsetScalar { data_type } => {
                write!(formatter, "unset scalar with declared type {data_type}")
            }
            Self::UnsetCollection { data_type } => {
                write!(
                    formatter,
                    "unset collection with declared type {data_type}"
                )
            }
            Self::EmptyCollection { data_type } => {
                write!(
                    formatter,
                    "empty collection with element type {data_type}"
                )
            }
        }
    }
}
