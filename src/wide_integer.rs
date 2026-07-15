// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Serde adapters for JSON-compatible 128-bit integer payloads.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{
    self,
    Visitor,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

/// Serializes a displayable integer as a decimal string without allocating.
struct DisplayInteger<'a, T>(&'a T);

impl<T> Serialize for DisplayInteger<'_, T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}

/// Parses one decimal string into an integer without retaining input text.
struct IntegerVisitor<T>(PhantomData<T>);

/// Parses and validates the unique textual form emitted by serialization.
fn parse_canonical_integer<T, E>(value: &str) -> Result<T, E>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
    E: de::Error,
{
    let parsed = value.parse::<T>().map_err(E::custom)?;
    if parsed.to_string() != value {
        return Err(E::custom("non-canonical 128-bit integer string"));
    }
    Ok(parsed)
}

impl<'de, T> Visitor<'de> for IntegerVisitor<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a base-10 128-bit integer string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_canonical_integer(value)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

/// Deserializable wrapper used by collection adapters.
struct ParsedInteger<T>(T);

impl<'de, T> Deserialize<'de> for ParsedInteger<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_str(IntegerVisitor(PhantomData))
            .map(Self)
    }
}

macro_rules! define_wide_integer_serde {
    ($scalar_module:ident, $vector_module:ident, $type:ty) => {
        pub(crate) mod $scalar_module {
            use std::marker::PhantomData;

            use serde::{
                Deserializer,
                Serialize,
                Serializer,
            };

            use super::{
                DisplayInteger,
                IntegerVisitor,
            };

            pub(crate) fn serialize<S>(
                value: &$type,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                DisplayInteger(value).serialize(serializer)
            }

            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_str(IntegerVisitor(PhantomData))
            }
        }

        pub(crate) mod $vector_module {
            use serde::{
                Deserialize,
                Deserializer,
                Serializer,
            };

            use super::{
                DisplayInteger,
                ParsedInteger,
            };

            pub(crate) fn serialize<S>(
                values: &[$type],
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(values.iter().map(DisplayInteger))
            }

            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                Vec::<ParsedInteger<$type>>::deserialize(deserializer).map(
                    |values| values.into_iter().map(|value| value.0).collect(),
                )
            }
        }
    };
}

define_wide_integer_serde!(int128, int128_vec, i128);
define_wide_integer_serde!(uint128, uint128_vec, u128);
