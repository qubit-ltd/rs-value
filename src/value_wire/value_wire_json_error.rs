// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
// =============================================================================

//! Privacy-safe structured metadata for JSON wire decoding failures.

use std::fmt;

use serde_json::error::Category;

/// Privacy-safe classification and source position for a JSON wire failure.
///
/// This type retains only the Serde category and source coordinates. It does
/// not retain the input, the original error, or its detailed message.
#[derive(Debug)]
pub struct ValueWireJsonError {
    category: Category,
    line: usize,
    column: usize,
}

impl ValueWireJsonError {
    /// Constructs safe JSON error metadata.
    pub(crate) fn new(category: Category, line: usize, column: usize) -> Self {
        Self {
            category,
            line,
            column,
        }
    }

    /// Returns the Serde JSON failure category.
    #[inline]
    pub fn classify(&self) -> Category {
        self.category
    }

    /// Returns the one-based input line, or zero when unavailable.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the one-based input column, or zero when unavailable.
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
