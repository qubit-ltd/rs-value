// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for contracts that require crate-internal access.

pub(crate) mod allocation_support;
mod finite_float_tests;
#[cfg(feature = "json")]
mod wire_api_tests;
