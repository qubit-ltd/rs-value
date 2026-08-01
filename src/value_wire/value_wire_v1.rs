// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public DTO for the stable version-one JSON wire contract.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{MultiValues, Value, ValueContainer};

use super::{
    VALUE_WIRE_V1_VERSION, ValueWireEncodeError, ValueWirePayloadV1, deserialize_wire,
    serialize_wire,
};
#[cfg(feature = "json")]
use super::{ValueWireDecodeError, ValueWireLimits};

/// Stable version-one wire DTO for a scalar or homogeneous collection.
///
/// For a given serializer and value, output is byte-stable with canonical field
/// and object-key order, including recursively nested JSON objects. V1 is
/// closed: existing tags, shapes, and payload representations cannot change,
/// and future runtime data types require a new wire version instead of
/// extending V1.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueWireV1 {
    /// Explicit runtime shape and typed payload represented by this DTO.
    value: ValueWirePayloadV1,
}

impl ValueWireV1 {
    /// Numeric version emitted and accepted by this DTO.
    pub const VERSION: u8 = VALUE_WIRE_V1_VERSION;

    /// Creates a V1 DTO from an explicit scalar-or-collection container.
    ///
    /// # Parameters
    ///
    /// * `value` - Runtime container whose exact type and shape are preserved.
    ///
    /// # Returns
    ///
    /// A V1 DTO containing `value`.
    #[inline(always)]
    pub const fn new(value: ValueWirePayloadV1) -> Self {
        Self { value }
    }

    /// Decodes a V1 JSON wire value using the default input byte limit.
    ///
    /// The byte limit is checked before JSON parsing or wire payload
    /// allocation. This method is suitable for untrusted input when the
    /// one-mebibyte default in [`ValueWireLimits`] matches the surrounding
    /// protocol. It accepts a complete top-level V1 document. When a
    /// [`Value`] is embedded in a larger JSON document, the outer protocol
    /// should call [`ValueWireLimits::check_json_bytes`] with the complete
    /// document length before invoking its own Serde decoder.
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    ///
    /// # Returns
    ///
    /// The decoded V1 wire DTO.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireDecodeError::InputTooLarge`] when `input` exceeds
    /// the default byte limit, or [`ValueWireDecodeError::InvalidJson`] when
    /// the bounded input is not a valid V1 JSON wire value.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(input: &[u8]) -> Result<Self, ValueWireDecodeError> {
        Self::decode_json_slice_with_limits(input, ValueWireLimits::default())
    }

    /// Decodes a V1 JSON wire value using explicit resource limits.
    ///
    /// The complete input length is checked before JSON parsing. This bounds
    /// all collection storage and arbitrary-precision numeric text reachable
    /// through the V1 wire payload. This method accepts a complete top-level
    /// V1 document; embedded values require the outer protocol to preflight
    /// its complete document length with
    /// [`ValueWireLimits::check_json_bytes`].
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    /// * `limits` - Resource limits checked before decoding begins.
    ///
    /// # Returns
    ///
    /// The decoded V1 wire DTO.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireDecodeError::InputTooLarge`] when `input` exceeds
    /// `limits`, or [`ValueWireDecodeError::InvalidJson`] when the bounded
    /// input is not a valid V1 JSON wire value.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: ValueWireLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        limits.check_json_bytes(input.len())?;
        serde_json::from_slice(input).map_err(ValueWireDecodeError::from)
    }

    /// Returns the runtime container represented by this DTO.
    ///
    /// # Returns
    ///
    /// A shared reference to the preserved runtime container.
    #[inline(always)]
    pub const fn container(&self) -> &ValueContainer {
        self.value.container()
    }

    /// Consumes the DTO and returns its runtime container.
    ///
    /// # Returns
    ///
    /// The preserved runtime container.
    #[inline(always)]
    pub fn into_container(self) -> ValueContainer {
        self.value.into_container()
    }
}

impl TryFrom<Value> for ValueWireV1 {
    type Error = ValueWireEncodeError;
    /// Wraps a runtime scalar in a V1 DTO.
    #[inline(always)]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        ValueWirePayloadV1::try_from(value).map(Self::new)
    }
}

impl TryFrom<MultiValues> for ValueWireV1 {
    type Error = ValueWireEncodeError;
    /// Wraps a runtime collection in a V1 DTO.
    #[inline(always)]
    fn try_from(values: MultiValues) -> Result<Self, Self::Error> {
        ValueWirePayloadV1::try_from(values).map(Self::new)
    }
}

impl TryFrom<ValueContainer> for ValueWireV1 {
    type Error = ValueWireEncodeError;
    /// Wraps an explicit runtime shape in a V1 DTO.
    #[inline(always)]
    fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
        ValueWirePayloadV1::try_from(value).map(Self::new)
    }
}

impl From<ValueWireV1> for ValueContainer {
    /// Unwraps the runtime container from a V1 DTO.
    #[inline(always)]
    fn from(value: ValueWireV1) -> Self {
        value.into_container()
    }
}

impl Serialize for ValueWireV1 {
    /// Serializes the contained runtime shape through the V1 contract.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(self.value.container().into(), serializer)
    }
}

impl<'de> Deserialize<'de> for ValueWireV1 {
    /// Deserializes a validated V1 runtime container into the DTO.
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer)
            .map(ValueWirePayloadV1::from_decoded)
            .map(Self::new)
    }
}
