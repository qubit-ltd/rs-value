// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit Serde seed for decoding one unversioned V1 payload.

use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeSeed;

use super::ValueWirePayloadV1;
use super::WireShapeOwned;

/// Explicit Serde seed for decoding one unversioned V1 payload.
///
/// Use this seed with a decoder that enforces the resource limits appropriate
/// for the surrounding document. For complete JSON input, prefer the
/// bounded JSON decode helpers on `ValueWirePayloadV1`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueWirePayloadV1Seed;

impl ValueWirePayloadV1Seed {
    /// Creates a seed for one unversioned V1 payload.
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}

impl<'de> DeserializeSeed<'de> for ValueWirePayloadV1Seed {
    type Value = ValueWirePayloadV1;

    /// Deserializes one unversioned V1 shape.
    #[inline(always)]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ValueWirePayloadV1::from_decoded(
            WireShapeOwned::deserialize(deserializer)?.into(),
        ))
    }
}
