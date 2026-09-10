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
///
/// With the `json` feature, a complete envelope can be decoded through the
/// public `JsonDecoder` entry point:
///
/// ```rust
/// # #[cfg(feature = "json")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
/// use qubit_json::decode::JsonDecoder;
/// use qubit_value::ValueWireV1Seed;
///
/// let limits = JsonDecodeLimits::builder().max_input_bytes(256usize).build();
/// let session = JsonDecodeSession::from_limits(limits);
/// let mut decoder = JsonDecoder::new(session);
/// let value = decoder.decode_seed_utf8(
///     ValueWireV1Seed::new(),
///     br#"{"version":1,"value":{"scalar":{"int32":7}}}"#,
/// )?;
/// assert_eq!(value.into_container().data_type(), qubit_datatype::DataType::Int32);
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "json"))]
/// # fn main() {}
/// ```
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
