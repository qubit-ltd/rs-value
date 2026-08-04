// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Error reported while decoding bounded JSON wire input.

use thiserror::Error;

/// Shared resource category enforced while decoding one wire document.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueWireLimitKind {
    /// Complete encoded input length.
    InputBytes,
    /// Recursive wire depth.
    Depth,
    /// Total decoded node count.
    Nodes,
    /// Elements in one collection.
    CollectionItems,
    /// Entries in one map.
    MapEntries,
    /// Bytes in one decoded string.
    StringBytes,
    /// Digits in one decoded numeric value.
    NumericDigits,
}

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

    /// The decoded value exceeded one structural resource limit.
    #[error("wire input {kind:?} value {value} exceeds the limit of {maximum}")]
    LimitExceeded {
        /// Resource category that exceeded its limit.
        kind: ValueWireLimitKind,
        /// Observed resource value.
        value: usize,
        /// Largest permitted resource value.
        maximum: usize,
    },

    /// The bounded input is not a valid V1 JSON wire value.
    #[error("failed to decode V1 JSON wire input: {0}")]
    InvalidJson(#[from] serde_json::Error),
}
