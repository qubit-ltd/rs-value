// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resource limits for decoding untrusted JSON wire input.

use super::ValueWireDecodeError;

/// Resource limits applied before decoding a V1 JSON wire value.
///
/// The total byte limit bounds all payload text and collection data presented
/// to Serde. Callers can choose a smaller budget when their protocol has a
/// tighter message-size contract.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueWireLimits {
    /// Maximum accepted JSON input length in bytes.
    max_json_bytes: usize,
}

impl ValueWireLimits {
    /// Default maximum JSON input length: one mebibyte.
    pub const DEFAULT_MAX_JSON_BYTES: usize = 1_048_576;

    /// Creates resource limits with the specified JSON input byte budget.
    ///
    /// # Parameters
    ///
    /// * `max_json_bytes` - Maximum accepted JSON input length in bytes.
    ///
    /// # Returns
    ///
    /// Resource limits using `max_json_bytes` as the complete input budget.
    #[inline(always)]
    pub const fn new(max_json_bytes: usize) -> Self {
        Self { max_json_bytes }
    }

    /// Returns the maximum accepted JSON input length in bytes.
    ///
    /// # Returns
    ///
    /// The complete JSON input byte budget.
    ///
    /// ```compile_fail
    /// #![deny(unused_must_use)]
    /// use qubit_value::ValueWireLimits;
    ///
    /// ValueWireLimits::default().max_json_bytes();
    /// ```
    #[must_use]
    #[inline(always)]
    pub const fn max_json_bytes(self) -> usize {
        self.max_json_bytes
    }

    /// Checks a complete JSON input length before decoding begins.
    ///
    /// This preflight check can be reused by protocols that embed
    /// [`crate::Value`] inside a larger JSON document. Call it with the outer
    /// document length before invoking that protocol's Serde decoder.
    ///
    /// # Parameters
    ///
    /// * `input_bytes` - Complete JSON document length in bytes.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `input_bytes` does not exceed this limit.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireDecodeError::InputTooLarge`] with the actual and
    /// configured byte counts when the input exceeds this limit.
    #[inline]
    pub const fn check_json_bytes(
        self,
        input_bytes: usize,
    ) -> Result<(), ValueWireDecodeError> {
        if input_bytes > self.max_json_bytes {
            Err(ValueWireDecodeError::InputTooLarge {
                input_bytes,
                max_input_bytes: self.max_json_bytes,
            })
        } else {
            Ok(())
        }
    }
}

impl Default for ValueWireLimits {
    /// Uses [`Self::DEFAULT_MAX_JSON_BYTES`] as the complete input budget.
    #[inline(always)]
    fn default() -> Self {
        Self::new(Self::DEFAULT_MAX_JSON_BYTES)
    }
}
