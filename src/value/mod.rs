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
//!
//! The private modules are split by stable responsibility: representation,
//! construction, access, identity, conversion, redaction, and borrowing.
//! The core `value` module intentionally retains the state-machine methods
//! that must be reviewed together; splitting those methods would obscure the
//! invariants without creating an independent test or ownership boundary.

mod internal;
#[cfg(feature = "redact")]
mod redaction;
#[allow(clippy::module_inception)]
mod value;
mod value_constructor;
#[cfg(feature = "converter")]
mod value_converters;
mod value_getter;
mod value_identity;
mod value_ref;

pub(crate) use self::internal::ValueRepr;
pub use self::value::Value;
pub use self::value_ref::ValueRef;
