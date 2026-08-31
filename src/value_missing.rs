// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structured reasons why a value read produced no concrete item.

use std::fmt;

use qubit_datatype::DataType;

/// Describes the typed state that produced a missing-value error.
///
/// # Examples
///
/// ```
/// use qubit_datatype::DataType;
/// use qubit_value::{Value, ValueError, ValueMissing};
///
/// let error = Value::new_unset(DataType::Int32).get::<i32>().unwrap_err();
/// assert!(matches!(
///     error,
///     ValueError::Missing(ValueMissing::UnsetScalar { data_type: DataType::Int32 })
/// ));
/// ```
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueMissing {
    /// A scalar was unset with a declared data type.
    UnsetScalar {
        /// Type retained by the unset scalar storage.
        data_type: DataType,
    },
    /// A collection was unset with a declared element type.
    UnsetCollection {
        /// Element type retained by the unset collection storage.
        data_type: DataType,
    },
    /// A concrete collection contains no item for a first-item read.
    EmptyCollection {
        /// Element type of the concrete empty collection.
        data_type: DataType,
    },
    /// A conversion requested one item from an empty collection.
    ///
    /// The source collection has no item and therefore no source value type
    /// can be recovered from the shared conversion error. `to` records the
    /// requested target type instead of overloading `EmptyCollection`.
    EmptyCollectionConversion {
        /// Requested target data type.
        to: DataType,
    },
    /// A conversion policy treated a concrete scalar as missing.
    Conversion {
        /// Declared source data type.
        from: DataType,
        /// Requested target data type.
        to: DataType,
    },
    /// A collection item conversion produced no value.
    CollectionItem {
        /// Original zero-based source position.
        source_index: usize,
        /// Declared source data type.
        from: DataType,
        /// Requested target data type.
        to: DataType,
    },
}

impl ValueMissing {
    /// Returns the source or declared data type associated with the error.
    ///
    /// Returns `None` for [`Self::EmptyCollectionConversion`] because no source
    /// item exists for that conversion.
    ///
    /// # Returns
    ///
    /// `Some(type)` when storage or a source item has a declared type, and
    /// `None` when an empty collection conversion has no source item.
    #[must_use]
    #[inline(always)]
    pub const fn source_type(self) -> Option<DataType> {
        match self {
            Self::UnsetScalar { data_type }
            | Self::UnsetCollection { data_type }
            | Self::EmptyCollection { data_type } => Some(data_type),
            Self::EmptyCollectionConversion { .. } => None,
            Self::Conversion { from, .. } | Self::CollectionItem { from, .. } => Some(from),
        }
    }

    /// Returns the requested target type for conversion failures.
    ///
    /// # Returns
    ///
    /// `Some(type)` for conversion-related variants and `None` for storage-only
    /// missing states.
    #[must_use]
    #[inline(always)]
    pub const fn target_type(self) -> Option<DataType> {
        match self {
            Self::Conversion { to, .. }
            | Self::CollectionItem { to, .. }
            | Self::EmptyCollectionConversion { to } => Some(to),
            Self::UnsetScalar { .. }
            | Self::UnsetCollection { .. }
            | Self::EmptyCollection { .. } => None,
        }
    }

    /// Returns the source index for a missing collection item.
    ///
    /// # Returns
    ///
    /// `Some(index)` for [`Self::CollectionItem`] and `None` otherwise.
    #[must_use]
    #[inline(always)]
    pub const fn source_index(self) -> Option<usize> {
        match self {
            Self::CollectionItem { source_index, .. } => Some(source_index),
            Self::UnsetScalar { .. }
            | Self::UnsetCollection { .. }
            | Self::EmptyCollection { .. }
            | Self::EmptyCollectionConversion { .. }
            | Self::Conversion { .. } => None,
        }
    }

    /// Reports whether storage itself is unset.
    ///
    /// # Returns
    ///
    /// `true` for unset scalar or collection storage.
    #[must_use]
    #[inline(always)]
    pub const fn is_unset(self) -> bool {
        matches!(
            self,
            Self::UnsetScalar { .. } | Self::UnsetCollection { .. }
        )
    }

    /// Reports whether a concrete collection is empty.
    ///
    /// # Returns
    ///
    /// `true` for direct or conversion-related empty collection states.
    #[must_use]
    #[inline(always)]
    pub const fn is_empty_collection(self) -> bool {
        matches!(
            self,
            Self::EmptyCollection { .. } | Self::EmptyCollectionConversion { .. }
        )
    }

    /// Reports whether the missing value came from a conversion.
    ///
    /// # Returns
    ///
    /// `true` for scalar, collection-item, or empty-collection conversions.
    #[must_use]
    #[inline(always)]
    pub const fn is_conversion(self) -> bool {
        matches!(
            self,
            Self::Conversion { .. }
                | Self::CollectionItem { .. }
                | Self::EmptyCollectionConversion { .. }
        )
    }

    /// Reports whether conversion APIs may use a caller-provided fallback.
    ///
    /// # Returns
    ///
    /// `true` for unset storage and scalar conversion-missing states.
    #[cfg(feature = "converter")]
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_defaultable_for_conversion(self) -> bool {
        self.is_unset() || matches!(self, Self::Conversion { .. })
    }
}

impl fmt::Display for ValueMissing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsetScalar { data_type } => {
                write!(formatter, "unset scalar with declared type {data_type}")
            }
            Self::UnsetCollection { data_type } => {
                write!(formatter, "unset collection with declared type {data_type}")
            }
            Self::EmptyCollection { data_type } => {
                write!(formatter, "empty collection with element type {data_type}")
            }
            Self::Conversion { from, to } => {
                write!(
                    formatter,
                    "conversion from {from} to {to} produced no value"
                )
            }
            Self::CollectionItem {
                source_index,
                from,
                to,
            } => write!(
                formatter,
                "collection item at index {source_index} conversion from {from} to {to} produced no value"
            ),
            Self::EmptyCollectionConversion { to } => {
                write!(
                    formatter,
                    "empty collection conversion to {to} produced no value"
                )
            }
        }
    }
}
