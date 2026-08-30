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
mod value_constructor;
mod value_getter;
mod value_identity;
mod value_ref;

pub use self::value::Value;
pub(crate) use self::value::ValueRepr;
pub use self::value_ref::ValueRef;
