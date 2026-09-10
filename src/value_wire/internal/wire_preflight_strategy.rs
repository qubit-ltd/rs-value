// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Feature-gated Wire V1 preflight strategy identities.

/// Internal classification used by Wire V1 preflight measurement.
#[cfg(feature = "json")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::value_wire) enum WirePreflightStrategy {
    /// Frozen type tag text for an unset payload.
    UnsetText,
    /// Boolean node.
    Boolean,
    /// Borrowed text node.
    BorrowedText,
    /// JSON integer node.
    JsonInteger,
    /// Decimal integer encoded as text.
    DecimalText,
    /// JSON floating point node.
    JsonFloat,
    /// Arbitrary precision integer text.
    BigIntegerText,
    /// Arbitrary precision decimal object.
    DecimalObject,
    /// Temporal text node.
    TemporalText,
    /// Duration object.
    DurationObject,
    /// String map object.
    StringMap,
    /// Nested JSON tree.
    JsonTree,
}
