// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde adapters for JSON-compatible 128-bit integer payloads.

use std::fmt;
use std::str::FromStr;

use serde::de;

mod internal;

use self::internal::DisplayInteger;
use self::internal::IntegerVisitor;
use self::internal::ParsedInteger;

/// Parses and validates the unique textual form emitted by serialization.
///
/// # Type Parameters
///
/// * `T` - Integer type parsed from and rendered to canonical decimal text.
/// * `E` - Deserializer error type used to report invalid input.
///
/// # Parameters
///
/// * `value` - Candidate decimal representation.
///
/// # Returns
///
/// The parsed integer when `value` is its unique canonical representation.
///
/// # Errors
///
/// Returns `E` when parsing fails or when rendering the parsed value does not
/// reproduce `value` exactly.
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

/// Defines scalar and vector Serde adapters for one wide integer type.
macro_rules! define_wide_integer_serde {
    ($scalar_module:ident, $vector_module:ident, $type:ty) => {
        /// Canonical scalar wire adapter generated for one wide integer type.
        pub(crate) mod $scalar_module {
            use std::marker::PhantomData;

            use serde::Deserializer;
            use serde::Serialize;
            use serde::Serializer;

            use super::DisplayInteger;
            use super::IntegerVisitor;

            /// Serializes a wide integer as a canonical decimal string.
            ///
            /// # Type Parameters
            ///
            /// * `S` - Destination serializer type.
            ///
            /// # Parameters
            ///
            /// * `value` - Wide integer to serialize.
            /// * `serializer` - Destination serializer.
            ///
            /// # Returns
            ///
            /// The destination serializer's result.
            ///
            /// # Errors
            ///
            /// Returns `S::Error` when serialization fails.
            pub(crate) fn serialize<S>(value: &$type, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                DisplayInteger(value).serialize(serializer)
            }

            /// Deserializes a wide integer from a canonical decimal string.
            ///
            /// # Type Parameters
            ///
            /// * `D` - Source deserializer type.
            ///
            /// # Parameters
            ///
            /// * `deserializer` - Source deserializer.
            ///
            /// # Returns
            ///
            /// The decoded wide integer.
            ///
            /// # Errors
            ///
            /// Returns `D::Error` for malformed or non-canonical input.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_str(IntegerVisitor(PhantomData))
            }
        }

        /// Canonical collection wire adapter generated for one wide integer type.
        pub(crate) mod $vector_module {
            use serde::Deserialize;
            use serde::Deserializer;
            use serde::Serializer;

            use super::DisplayInteger;
            use super::ParsedInteger;

            /// Serializes wide integers as canonical decimal strings.
            ///
            /// # Type Parameters
            ///
            /// * `S` - Destination serializer type.
            ///
            /// # Parameters
            ///
            /// * `values` - Wide integers to serialize in order.
            /// * `serializer` - Destination serializer.
            ///
            /// # Returns
            ///
            /// The destination serializer's sequence result.
            ///
            /// # Errors
            ///
            /// Returns `S::Error` when sequence serialization fails.
            pub(crate) fn serialize<S>(values: &[$type], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(values.iter().map(DisplayInteger))
            }

            /// Deserializes wide integers from canonical decimal strings.
            ///
            /// # Type Parameters
            ///
            /// * `D` - Source deserializer type.
            ///
            /// # Parameters
            ///
            /// * `deserializer` - Source deserializer.
            ///
            /// # Returns
            ///
            /// Decoded wide integers in source order.
            ///
            /// # Errors
            ///
            /// Returns `D::Error` for malformed or non-canonical input.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                Vec::<ParsedInteger<$type>>::deserialize(deserializer)
                    .map(|values| values.into_iter().map(|value| value.0).collect())
            }
        }
    };
}

define_wide_integer_serde!(int128, int128_vec, i128);
define_wide_integer_serde!(uint128, uint128_vec, u128);
