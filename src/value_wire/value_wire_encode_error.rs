// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors produced while constructing a V1 wire DTO.

#[cfg(feature = "json")]
use qubit_budget::BudgetError;
#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::QuantityConversionError;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use qubit_json::decode::JsonSyntaxError;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncodeError;
#[cfg(feature = "json")]
use qubit_json::encode::JsonSerializationError;
use thiserror::Error;

/// A runtime value cannot be represented by the JSON V1 wire contract.
///
/// # Examples
///
/// ```
/// use qubit_value::{Value, ValueWireEncodeError, ValueWireV1};
///
/// let error = ValueWireV1::try_from(Value::Float64(f64::NAN)).unwrap_err();
/// assert!(matches!(error, ValueWireEncodeError::NonFiniteFloat { .. }));
/// ```
#[non_exhaustive]
#[must_use]
#[derive(Debug, Error)]
pub enum ValueWireEncodeError {
    /// A JSON V1 float must be finite.
    #[error("V1 JSON wire cannot represent a non-finite {data_type} value")]
    NonFiniteFloat {
        /// Runtime data type of the rejected value.
        data_type: DataType,
    },
    /// A V1 decimal exponent must stay within the bounded wire range.
    #[error(
        "V1 JSON wire cannot represent decimal scale {scale}; maximum absolute scale is {maximum_absolute_scale}"
    )]
    BigDecimalScaleTooLarge {
        /// Rejected decimal exponent.
        scale: i64,
        /// Inclusive exponent magnitude limit for V1.
        maximum_absolute_scale: i64,
    },
    /// The JSON output exceeded one configured resource budget.
    #[cfg(feature = "json")]
    #[error("V1 JSON wire resource budget exceeded: {0}")]
    Budget(
        /// Budget violation reported by the bounded JSON encoder.
        #[source]
        BudgetError<JsonResource, usize>,
    ),
    /// A native JSON measurement could not be represented by the budget
    /// quantity type.
    #[cfg(feature = "json")]
    #[error("V1 JSON wire resource quantity conversion failed for {resource:?}: {source}")]
    Quantity {
        /// Resource whose measurement failed.
        resource: JsonResource,
        /// Native measurement conversion failure.
        #[source]
        source: QuantityConversionError,
    },
    /// The encoded value contains invalid JSON syntax.
    #[cfg(feature = "json")]
    #[error("invalid V1 JSON wire syntax: {0}")]
    Syntax(
        /// Syntax error found in an embedded raw JSON payload.
        #[source]
        JsonSyntaxError,
    ),
    /// Strict JSON serialization rejected the value during bounded encoding.
    #[cfg(feature = "json")]
    #[error("failed to encode V1 JSON wire value: {0}")]
    Json(
        /// Stable, privacy-safe JSON serialization failure.
        #[source]
        JsonSerializationError,
    ),
    /// The destination writer rejected bounded JSON output.
    #[cfg(feature = "json")]
    #[error("failed to write V1 JSON wire value: {0}")]
    Io(
        /// Destination-writer failure.
        #[source]
        std::io::Error,
    ),
}

#[cfg(feature = "json")]
impl From<JsonEncodeError<JsonResource, usize>> for ValueWireEncodeError {
    #[inline]
    fn from(error: JsonEncodeError<JsonResource>) -> Self {
        match error {
            JsonEncodeError::Budget(error) => match error {
                MeasuredBudgetError::Budget(error) => Self::Budget(error),
                MeasuredBudgetError::Quantity { resource, source } => {
                    Self::Quantity { resource, source }
                }
            },
            JsonEncodeError::InvalidRawJson(error) => Self::Syntax(error),
            JsonEncodeError::Serialize(error) => Self::Json(error),
            JsonEncodeError::Write(error) => Self::Io(error),
        }
    }
}
