// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public DTO for the stable version-one JSON wire contract.

#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_budget::encode_to_vec;
#[cfg(feature = "json")]
use qubit_budget::encode_to_writer;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use super::VALUE_WIRE_V1_VERSION;
#[cfg(feature = "json")]
use super::ValueWireDecodeError;
use super::ValueWireEncodeError;
use super::ValueWirePayloadV1;
use super::deserialize_wire;
use super::serialize_wire;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;

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

    /// Returns the default JSON resource profile for complete V1 documents.
    #[cfg(feature = "json")]
    #[must_use = "the V1 JSON profile should be applied to a budget"]
    #[inline]
    pub fn default_json_decode_limits() -> JsonDecodeLimits {
        super::default_json_decode_limits()
    }

    /// Returns the default JSON resource profile for encoding V1 documents.
    #[cfg(feature = "json")]
    #[must_use = "the V1 JSON profile should be applied to an encode session"]
    #[inline]
    pub fn default_json_encode_limits() -> JsonEncodeLimits {
        super::default_json_encode_limits()
    }

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
    /// [`qubit_budget::JsonDecodeSession`] across their complete document.
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
    /// [`ValueWireDecodeError::UnsupportedVersion`] when the envelope declares
    /// another supported-width version, or
    /// [`ValueWireDecodeError::InvalidJson`] for malformed input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice(
        input: &[u8],
    ) -> Result<Self, ValueWireDecodeError> {
        Self::decode_json_slice_with_limits(
            input,
            Self::default_json_decode_limits(),
        )
    }

    /// Decodes a V1 JSON wire value using explicit structural limits.
    ///
    /// The complete input length and decoded structure are checked before the
    /// value is returned. Embedded values should be checked through the outer
    /// protocol's shared [`qubit_budget::JsonDecodeSession`].
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
    /// `limits`, [`ValueWireDecodeError::UnsupportedVersion`] when the envelope
    /// declares another supported-width version, or
    /// [`ValueWireDecodeError::InvalidJson`] for malformed input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: JsonDecodeLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        let mut session = JsonDecodeSession::owned(limits);
        super::decode_wire_json_slice_with_session(input, &mut session)
    }

    /// Encodes this V1 document into a compact JSON vector with default limits.
    ///
    /// # Returns
    ///
    /// The encoded complete V1 document.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError::Budget`] when the document exceeds the
    /// default JSON resource profile.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(Self::default_json_encode_limits())
    }

    /// Encodes this V1 document into a bounded compact JSON vector.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_vec(self, &mut session).map_err(ValueWireEncodeError::from)
    }

    /// Encodes this V1 document to a writer with default limits.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete JSON document.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError::Budget`] for resource-limit failures or
    /// [`ValueWireEncodeError::Io`] when `writer` rejects output.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            Self::default_json_encode_limits(),
        )
    }

    /// Encodes this V1 document to a writer after enforcing JSON budgets.
    #[cfg(feature = "json")]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_writer(writer, self, &mut session)
            .map_err(ValueWireEncodeError::from)
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
