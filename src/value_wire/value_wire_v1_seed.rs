// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit Serde seed for decoding one V1 envelope.

use serde::Deserializer;
use serde::de::DeserializeSeed;

use super::ValueWirePayloadV1;
use super::ValueWireV1;
use super::deserialize_wire;

/// Explicit Serde seed for decoding one V1 envelope.
///
/// Use this seed with a decoder that enforces the resource limits appropriate
/// for the surrounding document. For complete JSON input, prefer the bounded
/// JSON decode helpers on `ValueWireV1`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueWireV1Seed;

impl ValueWireV1Seed {
    /// Creates a seed for one V1 envelope.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}

impl<'de> DeserializeSeed<'de> for ValueWireV1Seed {
    type Value = ValueWireV1;

    /// Deserializes one validated V1 runtime container into the DTO.
    #[inline(always)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer)
            .map(ValueWirePayloadV1::from_decoded)
            .map(ValueWireV1::new)
    }
}
