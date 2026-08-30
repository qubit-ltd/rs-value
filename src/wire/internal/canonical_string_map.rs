// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private canonical serializer for one string map.

use std::collections::HashMap;

use serde::Serialize;
use serde::ser::Serializer;

use super::super::string_map;

/// Borrows a string map while forcing dictionary-order entry emission.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed string map.
pub(in crate::wire) struct CanonicalStringMap<'a>(
    /// The string map to serialize.
    pub(in crate::wire) &'a HashMap<String, String>,
);

impl Serialize for CanonicalStringMap<'_> {
    /// Serializes the borrowed map through the canonical string-map adapter.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        string_map::serialize(self.0, serializer)
    }
}
