// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Error reported while decoding bounded JSON wire input.

use std::io;
use std::sync::Arc;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
use qubit_budget::json::JsonResource;
use qubit_json::decode::JsonDecodeError;
use qubit_json::decode::JsonDecodeErrorSource;
use qubit_json::decode::JsonSyntaxError;
use serde_json::Error as JsonError;
use serde_json::error::Category;
use thiserror::Error;

use super::value_wire_json_error::ValueWireJsonError;

/// Error produced by a bounded [`crate::ValueWireV1`] JSON decoder.
///
/// # Examples
///
/// ```
/// use qubit_value::{ValueWireDecodeError, ValueWireV1};
///
/// let error = ValueWireV1::decode_json_slice(b"{").unwrap_err();
/// assert!(matches!(
///     error,
///     ValueWireDecodeError::Syntax(_) | ValueWireDecodeError::InvalidJson(_)
/// ));
/// ```
#[non_exhaustive]
#[must_use]
#[derive(Debug, Error)]
pub enum ValueWireDecodeError {
    /// Reading the JSON wire input failed.
    #[error("failed to read V1 JSON wire input: {kind:?}")]
    Read {
        /// Stable standard I/O failure category.
        kind: io::ErrorKind,
        /// Original error retained for detailed diagnostics.
        #[source]
        source: Option<Arc<io::Error>>,
    },

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
        /// Safe Serde JSON metadata without the original detailed error.
        #[source]
        ValueWireJsonError,
    ),
}

impl ValueWireDecodeError {
    /// Constructs privacy-safe Serde metadata from structured decode facts.
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
        Self::InvalidJson(ValueWireJsonError::new(category, line, column))
    }
}

impl From<JsonDecodeError<JsonResource, usize>> for ValueWireDecodeError {
    /// Converts bounded decoder failures into the corresponding wire error.
    ///
    /// Decoder resource violations retain their structured details, syntax
    /// failures retain their source location, and other malformed inputs are
    /// represented by a privacy-safe deserialization error.
    ///
    /// # Parameters
    ///
    /// * `error` - Failure returned by the bounded JSON decoder.
    ///
    /// # Returns
    ///
    /// The corresponding value-wire decoding error.
    #[inline]
    fn from(error: JsonDecodeError<JsonResource>) -> Self {
        let line = error.line().unwrap_or(0);
        let column = error.column().unwrap_or(0);
        match error.into_source() {
            JsonDecodeErrorSource::Read { kind, source, .. } => Self::Read { kind, source },
            JsonDecodeErrorSource::Budget { source, .. } => match source {
                MeasuredBudgetError::Budget(error) => Self::Budget(error),
                MeasuredBudgetError::Quantity { resource, source } => Self::Quantity { resource, source },
            },
            JsonDecodeErrorSource::InvalidJson { syntax, .. } => Self::Syntax(syntax),
            JsonDecodeErrorSource::EmptyInput { .. }
            | JsonDecodeErrorSource::InvalidUtf8 { .. }
            | JsonDecodeErrorSource::UnexpectedTopLevel { .. }
            | JsonDecodeErrorSource::Deserialize { .. } => Self::deserialize(Category::Data, line, column),
        }
    }
}

impl From<JsonError> for ValueWireDecodeError {
    /// Converts a Serde JSON error while preserving its category and location.
    ///
    /// The formatted diagnostic does not retain the input contents.
    ///
    /// # Parameters
    ///
    /// * `error` - Serde JSON decoding failure.
    ///
    /// # Returns
    ///
    /// An invalid-JSON wire error with a privacy-safe diagnostic.
    #[inline]
    fn from(error: JsonError) -> Self {
        let category = error.classify();
        let line = error.line();
        let column = error.column();
        Self::deserialize(category, line, column)
    }
}
