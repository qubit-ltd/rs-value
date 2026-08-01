// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Error reported while decoding bounded JSON wire input.

use thiserror::Error;

/// Error produced by a bounded [`crate::ValueWireV1`] JSON decoder.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueWireDecodeError {
    /// The input exceeds the configured byte budget.
    #[error(
        "wire input contains {input_bytes} bytes, exceeding the {max_input_bytes}-byte limit"
    )]
    InputTooLarge {
        /// Actual input length in bytes.
        input_bytes: usize,
        /// Maximum accepted input length in bytes.
        max_input_bytes: usize,
    },

    /// The bounded input is not a valid V1 JSON wire value.
    #[error("failed to decode V1 JSON wire input: {0}")]
    InvalidJson(#[from] serde_json::Error),
}
