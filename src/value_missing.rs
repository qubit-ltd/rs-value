// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Missing-value facts retained across storage and conversion boundaries.

use std::fmt;

#[cfg(feature = "converter")]
use qubit_datatype::DataConversionError;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionErrorKind;
use qubit_datatype::DataType;

use crate::ValueMissingReason;

/// Describes the storage state, requested type and source of a missing read.
///
/// The fields are private so callers cannot manufacture an incomplete fact.
/// Use the constructors for storage states and the accessors for the optional
/// facts. `source_type` and `target_type` are `None` only when the caller has
/// no type information (for example, a generic empty iterator); `source_index`
/// is set only when a collection item caused the failure. A preserved
/// `conversion_error` is available only when the `converter` feature is on.
///
/// Strict fallback is allowed only for an unset value after type admission.
/// Conversion fallback additionally accepts a policy-classified missing scalar
/// but never a concrete empty collection or a missing collection item.
///
/// # Examples
///
/// ```
/// use qubit_datatype::DataType;
/// use qubit_value::{Value, ValueMissingReason};
///
/// let error = Value::new_unset(DataType::Int32).get::<i32>().unwrap_err();
/// let missing = error.missing().unwrap();
/// assert_eq!(missing.reason(), ValueMissingReason::UnsetScalar);
/// assert_eq!(missing.source_type(), Some(DataType::Int32));
/// assert_eq!(missing.target_type(), Some(DataType::Int32));
/// assert!(missing.is_defaultable_for_strict_read());
/// ```
///
/// Strict reads record both source and target. Conversion failures additionally
/// retain their original error and, for collection items, source index.
/// Inspect [`Self::reason`] and the accessors instead of matching storage
/// fields.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueMissing {
    reason: ValueMissingReason,
    source_type: Option<DataType>,
    target_type: Option<DataType>,
    source_index: Option<usize>,
    #[cfg(feature = "converter")]
    conversion_error: Option<DataConversionError>,
}

impl ValueMissing {
    /// Creates an unset scalar descriptor for a read from `source` to `target`.
    pub const fn unset_scalar(source: DataType, target: DataType) -> Self {
        Self::storage(ValueMissingReason::UnsetScalar, source, target)
    }

    /// Creates an unset collection descriptor for the requested element type.
    pub const fn unset_collection(source: DataType, target: DataType) -> Self {
        Self::storage(ValueMissingReason::UnsetCollection, source, target)
    }

    /// Creates a descriptor for a first-item read from a concrete empty
    /// collection.
    pub const fn empty_collection(source: DataType, target: DataType) -> Self {
        Self::storage(ValueMissingReason::EmptyCollection, source, target)
    }

    /// Records a known storage state without inventing a conversion source.
    const fn storage(reason: ValueMissingReason, source: DataType, target: DataType) -> Self {
        Self {
            reason,
            source_type: Some(source),
            target_type: Some(target),
            source_index: None,
            #[cfg(feature = "converter")]
            conversion_error: None,
        }
    }

    /// Preserves an already classified missing conversion and its original
    /// index.
    #[cfg(feature = "converter")]
    pub(crate) fn from_conversion(error: DataConversionError, source_index: Option<usize>) -> Self {
        Self {
            reason: if error.kind() == DataConversionErrorKind::EmptyCollection {
                ValueMissingReason::EmptyCollection
            } else {
                ValueMissingReason::Conversion
            },
            source_type: error.from_type(),
            target_type: Some(error.to_type()),
            source_index,
            conversion_error: Some(error),
        }
    }

    /// Enriches a conversion failure with facts known by its owning container.
    ///
    /// Only called after conversion admission has produced a missing error.
    #[cfg(feature = "converter")]
    pub(crate) fn with_storage(mut self, source: DataType, reason: ValueMissingReason) -> Self {
        self.source_type = Some(source);
        self.reason = reason;
        self
    }

    /// Records the first collection item's original index when conversion lost
    /// it.
    #[cfg(feature = "converter")]
    pub(crate) fn with_first_index(mut self) -> Self {
        if self.reason == ValueMissingReason::Conversion && self.source_index.is_none() {
            self.source_index = Some(0);
        }
        self
    }

    /// Returns the storage or policy reason for this missing result.
    ///
    /// This value is always present, including when the source or target type
    /// is unknown.
    pub const fn reason(&self) -> ValueMissingReason {
        self.reason
    }

    /// Returns the known source type, or `None` when conversion did not retain
    /// one (for example, a generic empty iterator).
    pub const fn source_type(&self) -> Option<DataType> {
        self.source_type
    }

    /// Returns the requested target type when known. Strict reads normally
    /// provide it; a low-level conversion failure may leave it absent.
    pub const fn target_type(&self) -> Option<DataType> {
        self.target_type
    }

    /// Returns the original collection item index, or `None` for an outer
    /// failure or a scalar conversion.
    pub const fn source_index(&self) -> Option<usize> {
        self.source_index
    }

    /// Returns the original conversion error, absent for a strict storage
    /// read. The error is the source for [`std::error::Error::source`].
    #[cfg(feature = "converter")]
    pub const fn conversion_error(&self) -> Option<&DataConversionError> {
        self.conversion_error.as_ref()
    }

    /// Reports whether scalar or collection storage is unset.
    ///
    /// This predicate is the condition used by strict fallback helpers.
    pub const fn is_unset(&self) -> bool {
        matches!(
            self.reason,
            ValueMissingReason::UnsetScalar | ValueMissingReason::UnsetCollection
        )
    }

    /// Reports whether a first-item read failed because a concrete collection
    /// is empty.
    pub const fn is_empty_collection(&self) -> bool {
        matches!(self.reason, ValueMissingReason::EmptyCollection)
    }

    /// Reports whether conversion produced this failure, including enriched
    /// unset states.
    pub const fn is_conversion(&self) -> bool {
        if matches!(self.reason, ValueMissingReason::Conversion) {
            return true;
        }
        #[cfg(feature = "converter")]
        {
            self.conversion_error.is_some()
        }
        #[cfg(not(feature = "converter"))]
        {
            false
        }
    }

    /// Allows strict fallback only for unset storage after the type check
    /// passed. Concrete empty collections and type mismatches return `false`.
    pub const fn is_defaultable_for_strict_read(&self) -> bool {
        self.is_unset()
    }

    /// Allows conversion fallback for unset storage or a policy-missing scalar.
    ///
    /// Empty collections and missing collection items never default.
    pub const fn is_defaultable_for_conversion(&self) -> bool {
        self.is_unset() || (matches!(self.reason, ValueMissingReason::Conversion) && self.source_index.is_none())
    }
}

impl fmt::Display for ValueMissing {
    /// Formats diagnostic types and indices without exposing source payloads.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?}: source {:?}, target {:?}",
            self.reason, self.source_type, self.target_type
        )?;
        if let Some(index) = self.source_index {
            write!(formatter, ", collection index {index}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValueMissing {
    /// Exposes the preserved conversion error, or terminates a strict-read
    /// chain.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        #[cfg(feature = "converter")]
        {
            self.conversion_error
                .as_ref()
                .map(|error| error as &dyn std::error::Error)
        }
        #[cfg(not(feature = "converter"))]
        {
            None
        }
    }
}
