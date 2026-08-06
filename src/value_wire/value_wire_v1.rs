// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public DTO for the stable version-one JSON wire contract.

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::{
    MultiValues,
    Value,
    ValueContainer,
};

use super::{
    VALUE_WIRE_V1_VERSION,
    ValueWireEncodeError,
    ValueWirePayloadV1,
    deserialize_wire,
    serialize_wire,
};
#[cfg(feature = "json")]
use super::{
    ValueWireDecodeError,
    WireLimits,
};

/// Stable version-one wire DTO for a scalar or homogeneous collection.
///
/// For a given serializer and value, output is byte-stable with canonical field
/// and object-key order, including recursively nested JSON objects. V1 is
/// closed: existing tags, shapes, and payload representations cannot change,
/// and future runtime data types require a new wire version instead of
/// extending V1.
///
/// # Resource limits
///
/// The generic [`Deserialize`](serde::Deserialize) implementation is intended
/// for already-bounded embedded documents and does not enforce message-size or
/// structural limits. Use `ValueWireV1::decode_json_slice` or
/// `ValueWireV1::decode_json_slice_with_limits` for untrusted complete JSON
/// input.
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

    /// Decodes a V1 JSON wire value using the default structural limits.
    ///
    /// The complete input length and decoded structure are checked before the
    /// value is returned. Embedded protocols should share one
    /// [`crate::WireBudget`] across their complete document.
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
    /// Returns a limit error when the input or decoded structure is too large,
    /// or [`ValueWireDecodeError::InvalidJson`] for malformed input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, ValueWireDecodeError> {
        Self::decode_json_slice_with_limits(input, WireLimits::default())
    }

    /// Decodes a V1 JSON wire value using explicit structural limits.
    ///
    /// The complete input length and decoded structure are checked before the
    /// value is returned. Embedded values should be checked through the outer
    /// protocol's shared [`crate::WireBudget`].
    ///
    /// # Parameters
    ///
    /// * `input` - Complete UTF-8 JSON document to decode.
    /// * `limits` - Shared encoded-input and structural limits.
    ///
    /// # Returns
    ///
    /// The decoded V1 wire DTO.
    ///
    /// # Errors
    ///
    /// Returns a limit error when `input` or its decoded structure exceeds
    /// `limits`, or [`ValueWireDecodeError::InvalidJson`] for malformed input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: WireLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        let mut budget = limits.begin_json(input)?;
        let value: Self = serde_json::from_slice(input)
            .map_err(ValueWireDecodeError::from)?;
        budget.check_container(value.container())?;
        Ok(value)
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
