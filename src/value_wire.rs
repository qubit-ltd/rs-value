// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Versioned, type-preserving wire representation for runtime values.

use serde::de::Error as _;
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
            ([cfg(feature = "big-number")], [serde(with = "crate::wire::big_integer")], [serde(with = "crate::wire::big_integer_vec")], BigInteger, num_bigint::BigInt, "biginteger"),
            ([cfg(feature = "big-number")], [serde(with = "crate::wire::big_decimal")], [serde(with = "crate::wire::big_decimal_vec")], BigDecimal, bigdecimal::BigDecimal, "bigdecimal"),
            ([], [], [], String, String, "string"),
            ([cfg(feature = "chrono")], [], [], Date, chrono::NaiveDate, "date"),
            ([cfg(feature = "chrono")], [], [], Time, chrono::NaiveTime, "time"),
            ([cfg(feature = "chrono")], [], [], DateTime, chrono::NaiveDateTime, "datetime"),
            ([cfg(feature = "chrono")], [], [], Instant, chrono::DateTime<chrono::Utc>, "instant"),
            ([], [serde(with = "crate::wire::duration")], [serde(with = "crate::wire::duration_vec")], Duration, std::time::Duration, "duration"),
            ([cfg(feature = "url")], [], [], Url, url::Url, "url"),
            ([], [], [], StringMap, std::collections::HashMap<String, String>, "stringmap"),
            ([cfg(feature = "json")], [], [], Json, serde_json::Value, "json"),
        }
    };
}

mod internal;
mod value_wire_v1;

use internal::{
    WireEnvelopeOwned,
    WireEnvelopeRef,
    WireShapeRef,
};
pub use value_wire_v1::ValueWireV1;

/// Serializes a typed shape through the V1 envelope.
fn serialize_wire<S>(
    value: WireShapeRef<'_>,
    serializer: S,
) -> Result<S::Ok, S::Error>
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Scalar(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Collection(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for MultiValues {
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(self.into(), serializer)
    }
}

impl<'de> Deserialize<'de> for ValueContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer)
    }
}
