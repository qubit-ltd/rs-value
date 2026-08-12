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
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::QuantityConversionError;
use serde_json::Error as JsonError;
use thiserror::Error;

/// Error produced by a bounded [`crate::ValueWireV1`] JSON decoder.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueWireDecodeError {
    /// The JSON document exceeded one configured resource budget.
    #[error("V1 JSON wire resource budget exceeded: {0}")]
    Budget(#[source] BudgetError<JsonResource, u64>),

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
    InvalidJson(#[source] JsonError),
}

impl From<JsonSerdeError<JsonResource>> for ValueWireDecodeError {
    #[inline]
    fn from(error: JsonSerdeError<JsonResource>) -> Self {
        match error {
            JsonSerdeError::Budget(error) => Self::Budget(error),
            JsonSerdeError::Quantity { resource, source } => Self::Quantity { resource, source },
            JsonSerdeError::Json(error) => Self::InvalidJson(error),
            JsonSerdeError::Io(error) => Self::InvalidJson(JsonError::io(error)),
        }
    }
}

impl From<JsonError> for ValueWireDecodeError {
    #[inline]
    fn from(error: JsonError) -> Self {
        Self::InvalidJson(error)
    }
}
