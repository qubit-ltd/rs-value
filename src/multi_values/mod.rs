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
//!
//! Construction, access, identity, and borrowing have independent modules.
//! The core collection state machine remains together because its unset,
//! empty, typed, and promotion transitions share one invariant boundary.

mod internal;
#[allow(clippy::module_inception)]
#[macro_use]
mod multi_values;
mod multi_values_constructor;
mod multi_values_getter;
mod multi_values_identity;
mod multi_values_ref;

pub(crate) use self::internal::MultiValuesRepr;
pub use self::multi_values::MultiValues;
pub use self::multi_values_ref::MultiValuesRef;
