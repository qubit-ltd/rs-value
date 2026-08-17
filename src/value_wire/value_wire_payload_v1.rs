// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unversioned V1 payload for use inside an already-versioned protocol.

#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_json::decode::JsonDecoder;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use super::ValueWireEncodeError;
use super::WireShapeOwned;
use super::WireShapeRef;
use super::value_wire_payload_ref_v1::validate_value;
use super::value_wire_payload_ref_v1::validate_values;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
#[cfg(feature = "json")]
use crate::ValueWireDecodeError;

/// Typed V1 scalar-or-collection payload without an enclosing version field.
///
/// # Resource limits
///
/// The generic [`Deserialize`](serde::Deserialize) implementation is intended
/// for already-bounded embedded documents and does not enforce message-size or
/// structural limits. Use `ValueWirePayloadV1::decode_json_slice` or
/// `ValueWirePayloadV1::decode_json_slice_with_limits` for untrusted complete
/// JSON input.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueWirePayloadV1 {
    /// Preserved runtime shape and payload.
    value: ValueContainer,
}

impl ValueWirePayloadV1 {
    /// Returns the default JSON resource profile for complete V1 payloads.
    #[cfg(feature = "json")]
    #[must_use = "the V1 JSON profile should be applied to a budget"]
    #[inline]
    pub fn default_json_decode_limits() -> JsonDecodeLimits {
        super::default_json_decode_limits()
    }

    /// Returns the default JSON resource profile for complete V1 payloads.
    #[cfg(feature = "json")]
    #[must_use = "the V1 JSON profile should be applied to an encode session"]
    #[inline]
    pub fn default_json_encode_limits() -> JsonEncodeLimits {
        super::default_json_encode_limits()
    }

    /// Decodes a complete V1 JSON payload using default resource limits.
    ///
    /// Prefer this entry point when the payload itself is the complete
    /// untrusted document. Embedded protocols should share one budget across
    /// all payloads in their complete document.
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
        Self::decode_json_slice_with_limits(
            input,
            Self::default_json_decode_limits(),
        )
    }

    /// Decodes a complete V1 JSON payload using explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns a limit error when `input` or its decoded structure exceeds
    /// `limits`, or [`ValueWireDecodeError::InvalidJson`] for malformed input.
    #[cfg(feature = "json")]
    #[inline]
    pub fn decode_json_slice_with_limits(
        input: &[u8],
        limits: JsonDecodeLimits,
    ) -> Result<Self, ValueWireDecodeError> {
        let session = JsonDecodeSession::owned(limits);
        JsonDecoder::new(session)
            .decode_utf8(input)
            .map_err(ValueWireDecodeError::from)
    }

    /// Encodes this V1 payload into a compact JSON vector with default limits.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError::Budget`] when the payload exceeds the
    /// default JSON resource profile.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(Self::default_json_encode_limits())
    }

    /// Encodes this V1 payload into a bounded compact JSON vector.
    #[cfg(feature = "json")]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let mut session = JsonEncodeSession::owned(limits);
        JsonEncoder::new(session)
            .to_vec(self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Encodes this V1 payload to a writer with default limits.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete JSON payload.
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

    /// Encodes this V1 payload to a writer after enforcing JSON budgets.
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
        JsonEncoder::new(session)
            .write_buffered(writer, self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Borrows the preserved runtime value.
    #[inline(always)]
    pub const fn container(&self) -> &ValueContainer {
        &self.value
    }

    /// Consumes this payload and returns its runtime value.
    #[inline(always)]
    pub fn into_container(self) -> ValueContainer {
        self.value
    }

    /// Builds a payload after enforcing V1's finite-float invariant.
    fn try_new(value: ValueContainer) -> Result<Self, ValueWireEncodeError> {
        match &value {
            ValueContainer::Scalar(value) => validate_value(value)?,
            ValueContainer::Collection(values) => validate_values(values)?,
        }
        Ok(Self { value })
    }

    /// Wraps a payload decoded through V1's finite-number Serde adapters.
    pub(in crate::value_wire) const fn from_decoded(
        value: ValueContainer,
    ) -> Self {
        Self { value }
    }
}

impl TryFrom<Value> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates a scalar for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::try_new(value.into())
    }
}

impl TryFrom<MultiValues> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates a collection for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: MultiValues) -> Result<Self, Self::Error> {
        Self::try_new(value.into())
    }
}

impl TryFrom<ValueContainer> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates an explicitly shaped value for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<ValueWirePayloadV1> for ValueContainer {
    /// Extracts the shaped runtime value from a V1 payload.
    #[inline(always)]
    fn from(value: ValueWirePayloadV1) -> Self {
        value.into_container()
    }
}

impl Serialize for ValueWirePayloadV1 {
    /// Serializes the unversioned V1 shape.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        WireShapeRef::from(&self.value).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValueWirePayloadV1 {
    /// Deserializes an unversioned V1 shape.
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_decoded(
            WireShapeOwned::deserialize(deserializer)?.into(),
        ))
    }
}
