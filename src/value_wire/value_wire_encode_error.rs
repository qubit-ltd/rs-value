// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors produced while constructing a V1 wire DTO.

use qubit_datatype::DataType;
use thiserror::Error;

/// A runtime value cannot be represented by the JSON V1 wire contract.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
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
}
