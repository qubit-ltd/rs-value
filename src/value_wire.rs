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
use qubit_budget::JsonDecodeLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonDecodeSession;
#[cfg(feature = "json")]
use qubit_budget::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonResource;
#[cfg(feature = "json")]
use qubit_budget::JsonValueLimits;
#[cfg(feature = "json")]
use qubit_budget::ResourceLimit;
#[cfg(feature = "json")]
use qubit_budget::StructureLimits;
#[cfg(feature = "json")]
use qubit_budget::decode_slice;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as _;

use crate::ValueContainer;

const VALUE_WIRE_V1_VERSION: u8 = 1;

/// Invokes a callback with the complete, independent V1 wire type table.
macro_rules! for_each_wire_type {
    ($macro:ident) => {
        $macro! {
            ([], [], [], Bool, bool, "bool"),
            ([], [], [], Char, char, "char"),
            ([], [], [], Int8, i8, "int8"),
            ([], [], [], Int16, i16, "int16"),
            ([], [], [], Int32, i32, "int32"),
            ([], [], [], Int64, i64, "int64"),
            ([], [serde(with = "crate::wire::int128")], [serde(with = "crate::wire::int128_vec")], Int128, i128, "int128"),
            ([], [], [], UInt8, u8, "uint8"),
            ([], [], [], UInt16, u16, "uint16"),
            ([], [], [], UInt32, u32, "uint32"),
            ([], [], [], UInt64, u64, "uint64"),
            ([], [serde(with = "crate::wire::uint128")], [serde(with = "crate::wire::uint128_vec")], UInt128, u128, "uint128"),
            ([], [serde(with = "crate::wire::float32")], [serde(with = "crate::wire::float32_vec")], Float32, f32, "float32"),
            ([], [serde(with = "crate::wire::float64")], [serde(with = "crate::wire::float64_vec")], Float64, f64, "float64"),
            ([cfg(feature = "big-integer")], [serde(with = "crate::wire::big_integer")], [serde(with = "crate::wire::big_integer_vec")], BigInteger, num_bigint::BigInt, "biginteger"),
            ([cfg(feature = "big-decimal")], [serde(with = "crate::wire::big_decimal")], [serde(with = "crate::wire::big_decimal_vec")], BigDecimal, bigdecimal::BigDecimal, "bigdecimal"),
            ([], [], [], String, String, "string"),
            ([cfg(feature = "chrono")], [serde(with = "crate::wire::date")], [serde(with = "crate::wire::date_vec")], Date, chrono::NaiveDate, "date"),
            ([cfg(feature = "chrono")], [serde(with = "crate::wire::time")], [serde(with = "crate::wire::time_vec")], Time, chrono::NaiveTime, "time"),
            ([cfg(feature = "chrono")], [serde(with = "crate::wire::datetime")], [serde(with = "crate::wire::datetime_vec")], DateTime, chrono::NaiveDateTime, "datetime"),
            ([cfg(feature = "chrono")], [serde(with = "crate::wire::instant")], [serde(with = "crate::wire::instant_vec")], Instant, chrono::DateTime<chrono::Utc>, "instant"),
            ([], [serde(with = "crate::wire::duration")], [serde(with = "crate::wire::duration_vec")], Duration, std::time::Duration, "duration"),
            ([cfg(feature = "url")], [serde(with = "crate::wire::url")], [serde(with = "crate::wire::url_vec")], Url, url::Url, "url"),
            ([], [serde(with = "crate::wire::string_map")], [serde(with = "crate::wire::string_map_vec")], StringMap, std::collections::HashMap<String, String>, "stringmap"),
            ([cfg(feature = "json")], [serde(with = "crate::wire::json")], [serde(with = "crate::wire::json_vec")], Json, serde_json::Value, "json"),
        }
    };
}

mod internal;
#[cfg(feature = "json")]
mod value_wire_decode_error;
mod value_wire_encode_error;
mod value_wire_payload_ref_v1;
mod value_wire_payload_v1;
mod value_wire_ref_v1;
mod value_wire_v1;

use internal::WireEnvelopeOwned;
use internal::WireEnvelopeRef;
use internal::WireShapeOwned;
use internal::WireShapeRef;
#[cfg(feature = "json")]
pub use value_wire_decode_error::ValueWireDecodeError;
pub use value_wire_encode_error::ValueWireEncodeError;
pub use value_wire_payload_ref_v1::ValueWirePayloadRefV1;
pub use value_wire_payload_v1::ValueWirePayloadV1;
pub use value_wire_ref_v1::ValueWireRefV1;
pub use value_wire_v1::ValueWireV1;

/// Returns the default value-resource profile used by V1 JSON documents.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_value_limits() -> JsonValueLimits {
    let structure = StructureLimits::empty()
        .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 64))
        .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 100_000))
        .with_sequence_items_limit(ResourceLimit::new(JsonResource::SequenceItems, 4_096))
        .with_map_entries_limit(ResourceLimit::new(JsonResource::MapEntries, 4_096))
        .with_key_bytes_limit(ResourceLimit::new(JsonResource::KeyBytes, 256 * 1024));
    JsonValueLimits::default()
        .with_structure_limits(structure)
        .with_string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, 256 * 1024))
        .with_number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, 4_096))
}

/// Returns the default resource profile used to decode V1 JSON documents.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_decode_limits() -> JsonDecodeLimits {
    JsonDecodeLimits::default()
        .with_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 1_048_576))
        .with_value_limits(default_json_value_limits())
}

/// Returns the default resource profile used to encode V1 JSON documents.
#[cfg(feature = "json")]
#[inline]
pub(crate) fn default_json_encode_limits() -> JsonEncodeLimits {
    JsonEncodeLimits::default()
        .with_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 1_048_576))
        .with_value_limits(default_json_value_limits())
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
    session: &mut JsonDecodeSession,
) -> Result<ValueWireV1, ValueWireDecodeError> {
    let envelope =
        decode_slice::<WireEnvelopeOwned, _>(input, session).map_err(ValueWireDecodeError::from)?;
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
