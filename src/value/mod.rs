// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Module
//!
//! Public entry for the single-value container implementation.

#[cfg(feature = "redact")]
mod redaction;
#[allow(clippy::module_inception)]
mod value;
mod value_accessors;
mod value_constructor;
#[cfg(feature = "converter")]
mod value_converters;
mod value_getter;
mod value_identity;
mod value_numeric_comparison;

pub(crate) use value::ValueRepr;
pub use value::{Value, ValueRef};
