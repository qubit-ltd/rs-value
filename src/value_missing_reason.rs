// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classification independent of the source and requested value types.

/// Identifies why a read could not produce a concrete value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValueMissingReason {
    /// Scalar storage is unset with a declared type.
    UnsetScalar,
    /// Collection storage is unset with a declared element type.
    UnsetCollection,
    /// A concrete collection has no first item.
    EmptyCollection,
    /// Conversion policy classified a concrete scalar or collection item as
    /// missing.
    Conversion,
}
