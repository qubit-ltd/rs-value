// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Tests
//!
//! Integration tests for `value` module.

#[cfg(feature = "json")]
mod internal;
#[cfg(all(feature = "redact", feature = "json"))]
mod redaction_tests;
#[cfg(feature = "all")]
mod value_constructor_tests;
mod value_contract_edge_tests;
#[cfg(feature = "all")]
mod value_converter_coverage_tests;
#[cfg(feature = "all")]
mod value_converter_tests;
#[cfg(feature = "all")]
mod value_converters_tests;
#[cfg(feature = "all")]
mod value_core_tests;
mod value_getter_tests;
#[cfg(feature = "all")]
mod value_identity_tests;
#[cfg(feature = "all")]
mod value_numeric_comparison_tests;
mod value_ref_tests;
mod value_setter_tests;
mod value_tests;
