// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for runtime-value identity helpers.

#[cfg(feature = "big-decimal")]
mod big_decimal_hash_tests;
mod float_identity_tests;
#[cfg(feature = "json")]
mod json_identity_tests;
#[cfg(feature = "json")]
mod string_map_hash_tests;
