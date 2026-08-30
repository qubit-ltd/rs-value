// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical textual adapters shared by decimal scalar and collection values.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;

mod internal;

use self::internal::DecimalVisitor;
use self::internal::DisplayDecimal;
use self::internal::ParsedDecimal;

/// Parses and validates the unique textual form emitted by serialization.
///
/// # Type Parameters
///
/// * `T` - Decimal type parsed from and rendered to canonical text.
/// * `E` - Deserializer error type used to report invalid input.
///
/// # Parameters
///
/// * `value` - Candidate decimal representation.
///
/// # Returns
///
/// The parsed decimal when `value` is its unique canonical representation.
///
/// # Errors
///
/// Returns `E` when parsing fails or when rendering the parsed value does not
/// reproduce `value` exactly.
fn parse_canonical_decimal<T, E>(value: &str) -> Result<T, E>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
    E: de::Error,
{
    let parsed = value.parse::<T>().map_err(E::custom)?;
    if parsed.to_string() != value {
        return Err(E::custom("non-canonical decimal string"));
    }
    Ok(parsed)
}

/// Serializes one decimal value as its stable textual form.
///
/// # Type Parameters
///
/// * `T` - Displayable decimal-backed value type.
/// * `S` - Destination serializer type.
///
/// # Parameters
///
/// * `value` - Decimal-backed value to serialize.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's result.
///
/// # Errors
///
/// Returns `S::Error` when serialization fails.
pub(super) fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: Serializer,
{
    DisplayDecimal(value).serialize(serializer)
}

/// Deserializes one decimal value from its textual form.
///
/// # Type Parameters
///
/// * `T` - Decimal-backed value type parsed from canonical text.
/// * `D` - Source deserializer type.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// The decoded decimal-backed value.
///
/// # Errors
///
/// Returns `D::Error` for malformed or non-canonical input.
pub(super) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(DecimalVisitor(PhantomData))
}

/// Serializes decimal values as a sequence of stable textual forms.
///
/// # Type Parameters
///
/// * `T` - Displayable decimal-backed element type.
/// * `S` - Destination serializer type.
///
/// # Parameters
///
/// * `values` - Decimal-backed values to serialize in order.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's sequence result.
///
/// # Errors
///
/// Returns `S::Error` when sequence serialization fails.
pub(super) fn serialize_vec<T, S>(values: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: Serializer,
{
    serializer.collect_seq(values.iter().map(DisplayDecimal))
}

/// Deserializes decimal values from a sequence of textual forms.
///
/// # Type Parameters
///
/// * `T` - Decimal-backed element type parsed from canonical text.
/// * `D` - Source deserializer type.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// Decoded decimal-backed values in source order.
///
/// # Errors
///
/// Returns `D::Error` for malformed or non-canonical input.
pub(super) fn deserialize_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
    D: Deserializer<'de>,
{
    Vec::<ParsedDecimal<T>>::deserialize(deserializer).map(|values| values.into_iter().map(|value| value.0).collect())
}
