// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Error reported while decoding bounded JSON wire input.
// qubit-style: allow multiple-public-types

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
use qubit_budget::json::JsonResource;
use qubit_json::decode::JsonDecodeError;
use qubit_json::decode::JsonSyntaxError;
use serde_json::Error as JsonError;
use serde_json::error::Category;
use thiserror::Error;

/// Error produced by a bounded [`crate::ValueWireV1`] JSON decoder.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueWireDecodeError {
    /// The JSON document exceeded one configured resource budget.
    #[error("V1 JSON wire resource budget exceeded: {0}")]
    Budget(
        /// Budget violation reported by the bounded JSON decoder.
        #[source]
        BudgetError<JsonResource, usize>,
    ),

    /// A native JSON measurement could not be represented by the budget
    /// quantity type.
    #[error("V1 JSON wire resource quantity conversion failed for {resource:?}: {source}")]
    Quantity {
        /// Resource whose measurement failed.
        resource: JsonResource,
        /// Native measurement conversion failure.
        #[source]
        source: QuantityConversionError,
    },

    /// The bounded input contains JSON syntax errors with source location.
    #[error("invalid V1 JSON wire syntax: {0}")]
    Syntax(
        /// Syntax error with its source location preserved.
        #[source]
        JsonSyntaxError,
    ),

    /// The envelope declares a wire version that this decoder does not support.
    #[error("unsupported qubit-value wire version {actual}; expected {expected}")]
    UnsupportedVersion {
        /// Wire version accepted by this decoder.
        expected: u8,

        /// Wire version declared by the input envelope.
        actual: u8,
    },

    /// The bounded input is not a valid V1 JSON wire value.
    #[error("failed to decode V1 JSON wire input: {0}")]
    InvalidJson(
        /// Serde JSON error stripped of input contents but retaining location.
        #[source]
        JsonError,
    ),
}

impl ValueWireDecodeError {
    /// Constructs a privacy-safe Serde error from structured decode metadata.
    ///
    /// # Parameters
    ///
    /// * `category` - Serde JSON failure category.
    /// * `line` - One-based input line, or zero when unavailable.
    /// * `column` - One-based input column, or zero when unavailable.
    ///
    /// # Returns
    ///
    /// An invalid-JSON error that retains diagnostics without input contents.
    fn deserialize(category: Category, line: usize, column: usize) -> Self {
        let error = <JsonError as serde::de::Error>::custom(format_args!(
            "JSON deserialization failed ({category:?}) at line {line}, column {column}"
        ));
        Self::InvalidJson(error)
    }
}

impl From<JsonDecodeError<JsonResource, usize>> for ValueWireDecodeError {
    #[inline]
    fn from(error: JsonDecodeError<JsonResource>) -> Self {
        if let Some(error) = error.budget_error().cloned() {
            return match error {
                MeasuredBudgetError::Budget(error) => Self::Budget(error),
                MeasuredBudgetError::Quantity { resource, source } => Self::Quantity { resource, source },
            };
        }
        if let Some(error) = error.syntax_error() {
            return Self::Syntax(*error);
        }
        Self::deserialize(Category::Data, error.line().unwrap_or(0), error.column().unwrap_or(0))
    }
}

impl From<JsonError> for ValueWireDecodeError {
    #[inline]
    fn from(error: JsonError) -> Self {
        let category = error.classify();
        let line = error.line();
        let column = error.column();
        Self::deserialize(category, line, column)
    }
}
