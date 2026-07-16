// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde payload adapters for value variants.

pub(crate) use crate::finite_float::{
    float32,
    float32_vec,
    float64,
    float64_vec,
};
pub(crate) use crate::wide_integer::{
    int128,
    int128_vec,
    uint128,
    uint128_vec,
};

#[cfg(feature = "big-number")]
mod decimal;

mod internal;

use internal::DurationPayload;

#[cfg(feature = "big-number")]
macro_rules! define_decimal_serde {
    ($scalar_module:ident, $vector_module:ident, $type:ty) => {
        pub(crate) mod $scalar_module {
            use serde::{
                Deserializer,
                Serializer,
            };

            use super::decimal;

            /// Serializes a decimal value as a canonical decimal string.
            pub(crate) fn serialize<S>(
                value: &$type,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                decimal::serialize(value, serializer)
            }

            /// Deserializes a decimal value from a canonical decimal string.
            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                decimal::deserialize(deserializer)
            }
        }

        pub(crate) mod $vector_module {
            use serde::{
                Deserializer,
                Serializer,
            };

            use super::decimal;

            /// Serializes decimal values as canonical decimal strings.
            pub(crate) fn serialize<S>(
                values: &[$type],
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                decimal::serialize_vec(values, serializer)
            }

            /// Deserializes decimal values from canonical decimal strings.
            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                decimal::deserialize_vec(deserializer)
            }
        }
    };
}

#[cfg(feature = "big-number")]
define_decimal_serde!(big_integer, big_integer_vec, num_bigint::BigInt);
#[cfg(feature = "big-number")]
define_decimal_serde!(big_decimal, big_decimal_vec, bigdecimal::BigDecimal);

/// Canonical scalar duration payload adapter.
pub(crate) mod duration {
    use std::time::Duration;

    use serde::de::Error as _;
    use serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
    };

    use super::DurationPayload;

    /// Serializes a duration as `{ secs, nanos }`.
    pub(crate) fn serialize<S>(
        value: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DurationPayload::from(value).serialize(serializer)
    }

    /// Deserializes and validates a `{ secs, nanos }` duration payload.
    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        DurationPayload::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

/// Canonical duration collection payload adapter.
pub(crate) mod duration_vec {
    use std::time::Duration;

    use serde::de::Error as _;
    use serde::{
        Deserialize,
        Deserializer,
        Serializer,
    };

    use super::DurationPayload;

    /// Serializes durations as a sequence of `{ secs, nanos }` payloads.
    pub(crate) fn serialize<S>(
        values: &[Duration],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(values.iter().map(DurationPayload::from))
    }

    /// Deserializes and validates a sequence of duration payloads.
    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<DurationPayload>::deserialize(deserializer)?
            .into_iter()
            .map(|value| value.try_into().map_err(D::Error::custom))
            .collect()
    }
}
