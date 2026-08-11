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
use qubit_budget::JsonResource;
#[cfg(feature = "json")]
use qubit_budget::JsonSerdeError;
#[cfg(feature = "json")]
use qubit_budget::QuantityConversionError;
use qubit_datatype::DataType;
#[cfg(feature = "json")]
use serde_json::Error as JsonError;
use thiserror::Error;

/// A runtime value cannot be represented by the JSON V1 wire contract.
#[derive(Debug, Error)]
#[non_exhaustive]
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
    /// A JSON V1 object uses serde_json's private number marker key.
    #[cfg(feature = "json")]
    #[error(
        "V1 JSON wire cannot represent an object containing the reserved key '{key}'"
    )]
    ReservedJsonObjectKey {
        /// Key reserved by serde_json's arbitrary-precision representation.
        key: &'static str,
    },
    /// The JSON output exceeded one configured resource budget.
    #[cfg(feature = "json")]
    #[error("V1 JSON wire resource budget exceeded: {0}")]
    Budget(#[source] BudgetError<JsonResource, u64>),
    /// A native JSON measurement could not fit the wire budget quantity.
    #[cfg(feature = "json")]
    #[error(
        "V1 JSON wire resource quantity conversion failed for {resource:?}: {source}"
    )]
    Quantity {
        /// Resource whose native measurement could not be represented.
        resource: JsonResource,

        /// Exact failed native quantity conversion.
        #[source]
        source: QuantityConversionError,
    },
    /// Serde JSON rejected the value during bounded encoding.
    #[cfg(feature = "json")]
    #[error("failed to encode V1 JSON wire value: {0}")]
    Json(#[source] JsonError),
    /// The destination writer rejected bounded JSON output.
    #[cfg(feature = "json")]
    #[error("failed to write V1 JSON wire value: {0}")]
    Io(#[source] std::io::Error),
}

#[cfg(feature = "json")]
impl From<JsonSerdeError<JsonResource>> for ValueWireEncodeError {
    #[inline]
    fn from(error: JsonSerdeError<JsonResource>) -> Self {
        match error {
            JsonSerdeError::Budget(error) => Self::Budget(error),
            JsonSerdeError::Quantity { resource, source } => {
                Self::Quantity { resource, source }
            }
            JsonSerdeError::Json(error) => Self::Json(error),
            JsonSerdeError::Io(error) => Self::Io(error),
        }
    }
}
