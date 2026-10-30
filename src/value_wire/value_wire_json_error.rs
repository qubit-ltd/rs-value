// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Privacy-safe structured metadata for JSON wire decoding failures.

use std::fmt;

use serde_json::error::Category;

/// Privacy-safe classification and source position for a JSON wire failure.
///
/// Obtain this metadata from [`ValueWireDecodeError::InvalidJson`] rather than
/// constructing it directly. It contains only the Serde category and source
/// coordinates; it does not retain the input or detailed error message. A
/// coordinate of zero means that the source position is unavailable, as can
/// happen for target-type failures after decoding through a unique-key tree.
/// Direct conversion from a Serde error preserves its source coordinates.
///
/// [`ValueWireDecodeError::InvalidJson`]: crate::ValueWireDecodeError::InvalidJson
///
/// # Examples
///
/// ```
/// use qubit_value::ValueWireDecodeError;
/// use qubit_value::ValueWireJsonError;
/// use serde_json::error::Category;
///
/// fn check_metadata(metadata: &ValueWireJsonError) {
///     assert_eq!(metadata.classify(), Category::Data);
///     assert!(metadata.is_data());
///     assert!(metadata.line() > 0);
///     assert!(metadata.column() > 0);
/// }
///
/// let source = serde_json::from_str::<u64>(r#""not a number""#).unwrap_err();
/// let error = ValueWireDecodeError::from(source);
/// let ValueWireDecodeError::InvalidJson(metadata) = error else {
///     panic!("direct Serde conversion must preserve safe metadata");
/// };
/// check_metadata(&metadata);
/// ```
#[derive(Debug)]
pub struct ValueWireJsonError {
    /// Serde's category for the JSON decoding failure.
    category: Category,
    /// One-based source line, or zero when unavailable.
    line: usize,
    /// One-based source column, or zero when unavailable.
    column: usize,
}

impl ValueWireJsonError {
    /// Constructs safe JSON error metadata.
    ///
    /// # Parameters
    ///
    /// - `category`: Serde's category for the failure.
    /// - `line`: one-based source line, or zero when unavailable.
    /// - `column`: one-based source column, or zero when unavailable.
    ///
    /// # Returns
    ///
    /// The metadata containing only the supplied category and coordinates.
    pub(crate) fn new(category: Category, line: usize, column: usize) -> Self {
        Self { category, line, column }
    }

    /// Returns the Serde JSON failure category.
    ///
    /// # Returns
    ///
    /// The category associated with the failure.
    #[inline]
    pub fn classify(&self) -> Category {
        self.category
    }

    /// Returns the one-based input line, or zero when unavailable.
    ///
    /// # Returns
    ///
    /// The source line, or zero if no source position is available.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the one-based input column, or zero when unavailable.
    ///
    /// # Returns
    ///
    /// The source column, or zero if no source position is available.
    #[inline]
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns whether the error is an I/O failure.
    #[inline]
    pub fn is_io(&self) -> bool {
        self.category == Category::Io
    }

    /// Returns whether the error is a JSON syntax failure.
    #[inline]
    pub fn is_syntax(&self) -> bool {
        self.category == Category::Syntax
    }

    /// Returns whether the JSON data has an incompatible type or value.
    #[inline]
    pub fn is_data(&self) -> bool {
        self.category == Category::Data
    }

    /// Returns whether the JSON input ended before decoding completed.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.category == Category::Eof
    }
}

impl fmt::Display for ValueWireJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "JSON deserialization failed ({:?}) at line {}, column {}",
            self.category, self.line, self.column
        )
    }
}

impl std::error::Error for ValueWireJsonError {}
