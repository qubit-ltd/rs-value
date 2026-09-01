// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Versioned, type-preserving JSON wire representation for runtime values.
//!
//! Runtime value types do not implement Serde directly. Callers select a
//! standalone [`ValueWireV1`] envelope or nested [`ValueWirePayloadV1`].
//! Borrowed values can use [`ValueWireRefV1`] or [`ValueWirePayloadRefV1`].

#[cfg(feature = "json")]
use qubit_budget::ResourceLimit;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonResource;
#[cfg(feature = "json")]
use qubit_budget::json::JsonValueLimits;
#[cfg(feature = "json")]
use qubit_json::decode::JsonDecoder;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as _;

use crate::ValueContainer;

/// Version tag emitted and accepted by the V1 wire envelope.
const VALUE_WIRE_V1_VERSION: u8 = 1;

mod internal;
#[cfg(feature = "json")]
mod value_wire_decode_error;
mod value_wire_encode_error;
mod value_wire_payload_ref_v1;
mod value_wire_payload_v1;
mod value_wire_payload_v1_seed;
mod value_wire_ref_v1;
mod value_wire_v1;
mod value_wire_v1_seed;

use self::internal::WireEnvelopeOwned;
use self::internal::WireEnvelopeRef;
use self::internal::WireShapeOwned;
use self::internal::WireShapeRef;
#[cfg(feature = "json")]
pub use self::value_wire_decode_error::ValueWireDecodeError;
pub use self::value_wire_encode_error::ValueWireEncodeError;
pub use self::value_wire_payload_ref_v1::ValueWirePayloadRefV1;
pub use self::value_wire_payload_v1::ValueWirePayloadV1;
pub use self::value_wire_payload_v1_seed::ValueWirePayloadV1Seed;
pub use self::value_wire_ref_v1::ValueWireRefV1;
pub use self::value_wire_v1::ValueWireV1;
pub use self::value_wire_v1_seed::ValueWireV1Seed;

/// Returns the default value-resource profile used by V1 JSON documents.
///
/// # Returns
///
/// Structural and textual limits shared by V1 JSON encoders and decoders.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_value_limits() -> JsonValueLimits {
    JsonValueLimits::<JsonResource, usize>::builder()
        .max_depth(64_usize)
        .max_nodes(100_000_usize)
        .max_sequence_items(4_096_usize)
        .max_map_entries(4_096_usize)
        .max_key_bytes(256 * 1024_usize)
        .max_string_bytes(256 * 1024_usize)
        .max_number_bytes(4_096_usize)
        .max_payload_bytes(1_048_576_usize)
        .build()
}

/// Returns the default resource profile used to decode V1 JSON documents.
///
/// # Returns
///
/// Input-byte and decoded-value limits for one standalone V1 document.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_decode_limits() -> JsonDecodeLimits {
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 1_048_576))
        .value_limits(default_json_value_limits())
        .build()
}

/// Returns the default resource profile used to encode V1 JSON documents.
///
/// # Returns
///
/// Output-byte and value limits for one standalone V1 document.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_encode_limits() -> JsonEncodeLimits {
    JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1_048_576))
        .value_limits(default_json_value_limits())
        .build()
}

/// Decodes and validates a complete V1 envelope with one caller-owned budget.
///
/// # Parameters
///
/// * `input` - Complete UTF-8 JSON document to decode.
/// * `budget` - Budget charged for the complete envelope and its payload.
///
/// # Returns
///
/// The decoded V1 wire DTO.
///
/// # Errors
///
/// Returns [`ValueWireDecodeError::Budget`] when the document exceeds the
/// budget, [`ValueWireDecodeError::UnsupportedVersion`] when the decoded
/// envelope declares another `u8` version, or
/// [`ValueWireDecodeError::InvalidJson`] when typed decoding fails.
#[cfg(feature = "json")]
pub(crate) fn decode_wire_json_slice_with_session(
    input: &[u8],
    session: JsonDecodeSession,
) -> Result<ValueWireV1, ValueWireDecodeError> {
    let envelope = JsonDecoder::new(session)
        .decode_utf8::<WireEnvelopeOwned>(input)
        .map_err(ValueWireDecodeError::from)?;
    if envelope.version != VALUE_WIRE_V1_VERSION {
        return Err(ValueWireDecodeError::UnsupportedVersion {
            expected: VALUE_WIRE_V1_VERSION,
            actual: envelope.version,
        });
    }
    Ok(ValueWireV1::new(ValueWirePayloadV1::from_decoded(
        envelope.value.into(),
    )))
}

/// Serializes a typed shape through the V1 envelope.
///
/// # Type Parameters
///
/// * `S` - Serde serializer receiving the V1 envelope.
///
/// # Parameters
///
/// * `value` - Borrowed runtime shape to encode.
/// * `serializer` - Serde serializer receiving the envelope.
///
/// # Returns
///
/// The serializer output for the versioned wire envelope.
///
/// # Errors
///
/// Returns the error reported by `serializer`.
#[inline(always)]
fn serialize_wire<S>(value: WireShapeRef<'_>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    WireEnvelopeRef {
        version: VALUE_WIRE_V1_VERSION,
        value,
    }
    .serialize(serializer)
}

/// Deserializes and validates a V1 envelope.
///
/// # Type Parameters
///
/// * `D` - Serde deserializer supplying the encoded envelope.
///
/// # Parameters
///
/// * `deserializer` - Source of the V1 wire envelope.
///
/// # Returns
///
/// The decoded runtime container.
///
/// # Errors
///
/// Returns `D::Error` when the envelope cannot be decoded or declares an
/// unsupported wire version.
#[inline]
fn deserialize_wire<'de, D>(deserializer: D) -> Result<ValueContainer, D::Error>
where
    D: Deserializer<'de>,
{
    let envelope = WireEnvelopeOwned::deserialize(deserializer)?;
    if envelope.version != VALUE_WIRE_V1_VERSION {
        return Err(D::Error::custom(format_args!(
            "unsupported qubit-value wire version {}",
            envelope.version,
        )));
    }
    Ok(envelope.value.into())
}
