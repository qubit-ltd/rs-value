// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Multiple Values Module
//!
//! Public entry for multiple-values container implementations.

#[allow(clippy::module_inception)]
#[macro_use]
mod multi_values;
mod multi_values_constructor;
#[cfg(feature = "converter")]
mod multi_values_converters;
mod multi_values_core;
mod multi_values_getter;
mod multi_values_getters;
mod multi_values_identity;

pub(crate) use multi_values::MultiValuesRepr;
pub use multi_values::{MultiValues, MultiValuesRef};
