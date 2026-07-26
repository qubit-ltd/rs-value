// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Versioned, type-preserving JSON wire representation for runtime values.
//!
//! V1 compatibility applies to the documented JSON object structure. The
//! Serde implementations can be used with other serializers, but their
//! format-specific representation is outside the V1 stability contract.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{MultiValues, Value, ValueContainer};

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
            ([], [], [], StringMap, std::collections::HashMap<String, String>, "stringmap"),
            ([cfg(feature = "json")], [], [], Json, serde_json::Value, "json"),
        }
    };
}

mod internal;
#[cfg(feature = "json")]
mod value_wire_decode_error;
#[cfg(feature = "json")]
mod value_wire_limits;
mod value_wire_v1;

use internal::{WireEnvelopeOwned, WireEnvelopeRef, WireShapeRef};
#[cfg(feature = "json")]
pub use value_wire_decode_error::ValueWireDecodeError;
#[cfg(feature = "json")]
pub use value_wire_limits::ValueWireLimits;
pub use value_wire_v1::ValueWireV1;

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

impl Serialize for Value {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Scalar(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_wire(deserializer)? {
            ValueContainer::Scalar(value) => Ok(value),
            ValueContainer::Collection(_) => {
                Err(D::Error::custom("expected scalar value wire shape"))
            }
        }
    }
}

impl Serialize for MultiValues {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Collection(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for MultiValues {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_wire(deserializer)? {
            ValueContainer::Collection(values) => Ok(values),
            ValueContainer::Scalar(_) => {
                Err(D::Error::custom("expected collection value wire shape"))
            }
        }
    }
}

impl Serialize for ValueContainer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(self.into(), serializer)
    }
}

impl<'de> Deserialize<'de> for ValueContainer {
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer)
    }
}
