// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde payload adapters for value variants.

#[cfg(any(feature = "chrono", feature = "url"))]
use serde::Deserialize;

pub(crate) use crate::finite_float::{float32, float32_vec, float64, float64_vec};
pub(crate) use crate::wide_integer::{int128, int128_vec, uint128, uint128_vec};

#[cfg(feature = "big-integer")]
mod decimal;

mod internal;

#[cfg(feature = "big-decimal")]
use internal::BigDecimalPayload;
use internal::DurationPayload;

/// Largest decimal exponent magnitude accepted by the V1 wire format.
#[cfg(feature = "big-decimal")]
pub(crate) const MAX_BIG_DECIMAL_ABSOLUTE_SCALE: i64 = 150_000;

/// Returns whether a decimal exponent is representable by the bounded V1
/// format.
#[cfg(feature = "big-decimal")]
#[inline(always)]
pub(crate) const fn is_valid_big_decimal_scale(scale: i64) -> bool {
    scale.unsigned_abs() <= MAX_BIG_DECIMAL_ABSOLUTE_SCALE as u64
}

/// Serializes and validates canonical scalar string payloads.
#[cfg(any(feature = "chrono", feature = "url"))]
fn serialize_canonical<S, T, F>(value: &T, serializer: S, format: F) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    F: FnOnce(&T) -> String,
{
    serializer.serialize_str(&format(value))
}

/// Deserializes a scalar string only when it is already in canonical form.
#[cfg(any(feature = "chrono", feature = "url"))]
fn deserialize_canonical<'de, D, T, P, F>(
    deserializer: D,
    parse: P,
    format: F,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    P: FnOnce(&str) -> Result<T, String>,
    F: FnOnce(&T) -> String,
{
    use serde::de::Error as _;

    let input = String::deserialize(deserializer)?;
    let value = parse(&input).map_err(D::Error::custom)?;
    if format(&value) != input {
        return Err(D::Error::custom("non-canonical V1 string payload"));
    }
    Ok(value)
}

/// Serializes a collection through a canonical scalar formatter.
#[cfg(any(feature = "chrono", feature = "url"))]
fn serialize_canonical_vec<S, T, F>(
    values: &[T],
    serializer: S,
    format: F,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    F: Fn(&T) -> String,
{
    serializer.collect_seq(values.iter().map(format))
}

/// Deserializes canonical string collection payloads.
#[cfg(any(feature = "chrono", feature = "url"))]
fn deserialize_canonical_vec<'de, D, T, P, F>(
    deserializer: D,
    parse: P,
    format: F,
) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    P: Fn(&str) -> Result<T, String>,
    F: Fn(&T) -> String,
{
    use serde::de::Error as _;

    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|input| {
            let value = parse(&input).map_err(D::Error::custom)?;
            if format(&value) != input {
                return Err(D::Error::custom("non-canonical V1 string payload"));
            }
            Ok(value)
        })
        .collect()
}

#[cfg(feature = "chrono")]
macro_rules! define_chrono_wire {
    ($scalar:ident, $vector:ident, $type:ty, $parse:expr, $format:expr) => {
        pub(crate) mod $scalar {
            use serde::{Deserializer, Serializer};

            /// Serializes the chrono value through the crate-owned V1 format.
            pub(crate) fn serialize<S>(value: &$type, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                super::serialize_canonical(value, serializer, $format)
            }

            /// Deserializes the chrono value only from its canonical V1 format.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::deserialize_canonical(deserializer, $parse, $format)
            }
        }

        pub(crate) mod $vector {
            use serde::{Deserializer, Serializer};

            /// Serializes chrono values through the crate-owned V1 format.
            pub(crate) fn serialize<S>(values: &[$type], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                super::serialize_canonical_vec(values, serializer, $format)
            }

            /// Deserializes chrono values only from their canonical V1 format.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::deserialize_canonical_vec(deserializer, $parse, $format)
            }
        }
    };
}

#[cfg(feature = "chrono")]
define_chrono_wire!(
    date,
    date_vec,
    chrono::NaiveDate,
    |input| chrono::NaiveDate::parse_from_str(input, "%F").map_err(|error| error.to_string()),
    |value: &chrono::NaiveDate| value.format("%F").to_string()
);

#[cfg(feature = "chrono")]
define_chrono_wire!(
    time,
    time_vec,
    chrono::NaiveTime,
    |input| chrono::NaiveTime::parse_from_str(input, "%H:%M:%S%.f")
        .map_err(|error| error.to_string()),
    |value: &chrono::NaiveTime| value.format("%H:%M:%S%.f").to_string()
);

#[cfg(feature = "chrono")]
define_chrono_wire!(
    datetime,
    datetime_vec,
    chrono::NaiveDateTime,
    |input| chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f")
        .map_err(|error| error.to_string()),
    |value: &chrono::NaiveDateTime| value.format("%Y-%m-%dT%H:%M:%S%.f").to_string()
);

#[cfg(feature = "chrono")]
define_chrono_wire!(
    instant,
    instant_vec,
    chrono::DateTime<chrono::Utc>,
    |input| chrono::DateTime::parse_from_rfc3339(input)
        .map(|value| value.with_timezone(&chrono::Utc))
        .map_err(|error| error.to_string()),
    |value: &chrono::DateTime<chrono::Utc>| value
        .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
);

#[cfg(feature = "url")]
macro_rules! define_url_wire {
    () => {
        pub(crate) mod url {
            use serde::{Deserializer, Serializer};

            /// Serializes a URL through its canonical normalized string.
            pub(crate) fn serialize<S>(value: &::url::Url, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                super::serialize_canonical(value, serializer, |value: &::url::Url| {
                    value.as_str().to_owned()
                })
            }

            /// Deserializes only a canonical normalized URL string.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<::url::Url, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::deserialize_canonical(
                    deserializer,
                    |input| ::url::Url::parse(input).map_err(|error| error.to_string()),
                    |value: &::url::Url| value.as_str().to_owned(),
                )
            }
        }

        pub(crate) mod url_vec {
            use serde::{Deserializer, Serializer};

            /// Serializes URLs through their canonical normalized strings.
            pub(crate) fn serialize<S>(
                values: &[::url::Url],
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                super::serialize_canonical_vec(values, serializer, |value: &::url::Url| {
                    value.as_str().to_owned()
                })
            }

            /// Deserializes only canonical normalized URL strings.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<::url::Url>, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::deserialize_canonical_vec(
                    deserializer,
                    |input| ::url::Url::parse(input).map_err(|error| error.to_string()),
                    |value: &::url::Url| value.as_str().to_owned(),
                )
            }
        }
    };
}

#[cfg(feature = "url")]
define_url_wire!();

#[cfg(feature = "big-integer")]
macro_rules! define_decimal_serde {
    ($scalar_module:ident, $vector_module:ident, $type:ty) => {
        pub(crate) mod $scalar_module {
            use serde::{Deserializer, Serializer};

            use super::decimal;

            /// Serializes a decimal value as a canonical decimal string.
            pub(crate) fn serialize<S>(value: &$type, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                decimal::serialize(value, serializer)
            }

            /// Deserializes a decimal value from a canonical decimal string.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                decimal::deserialize(deserializer)
            }
        }

        pub(crate) mod $vector_module {
            use serde::{Deserializer, Serializer};

            use super::decimal;

            /// Serializes decimal values as canonical decimal strings.
            pub(crate) fn serialize<S>(values: &[$type], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                decimal::serialize_vec(values, serializer)
            }

            /// Deserializes decimal values from canonical decimal strings.
            pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                decimal::deserialize_vec(deserializer)
            }
        }
    };
}

#[cfg(feature = "big-integer")]
define_decimal_serde!(big_integer, big_integer_vec, num_bigint::BigInt);

/// Canonical arbitrary-precision decimal scalar payload adapter.
#[cfg(feature = "big-decimal")]
pub(crate) mod big_decimal {
    use bigdecimal::BigDecimal;
    use serde::de::Error as _;
    use serde::ser::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::BigDecimalPayload;

    /// Serializes a decimal as an exact `{ coefficient, scale }` payload.
    pub(crate) fn serialize<S>(value: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BigDecimalPayload::try_from(value)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }

    /// Deserializes and validates an exact decimal payload.
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        BigDecimalPayload::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

/// Canonical arbitrary-precision decimal collection payload adapter.
#[cfg(feature = "big-decimal")]
pub(crate) mod big_decimal_vec {
    use bigdecimal::BigDecimal;
    use serde::de::Error as _;
    use serde::ser::{Error as _, SerializeSeq};
    use serde::{Deserialize, Deserializer, Serializer};

    use super::BigDecimalPayload;

    /// Serializes decimals as exact `{ coefficient, scale }` payloads.
    pub(crate) fn serialize<S>(values: &[BigDecimal], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(values.len()))?;
        for value in values {
            let payload = BigDecimalPayload::try_from(value).map_err(S::Error::custom)?;
            sequence.serialize_element(&payload)?;
        }
        sequence.end()
    }

    /// Deserializes and validates exact decimal payloads.
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<BigDecimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<BigDecimalPayload>::deserialize(deserializer)?
            .into_iter()
            .map(|value| value.try_into().map_err(D::Error::custom))
            .collect()
    }
}

/// Canonical scalar duration payload adapter.
pub(crate) mod duration {
    use std::time::Duration;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::DurationPayload;

    /// Serializes a duration as `{ secs, nanos }`.
    pub(crate) fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DurationPayload::from(value).serialize(serializer)
    }

    /// Deserializes and validates a `{ secs, nanos }` duration payload.
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
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
    use serde::{Deserialize, Deserializer, Serializer};

    use super::DurationPayload;

    /// Serializes durations as a sequence of `{ secs, nanos }` payloads.
    pub(crate) fn serialize<S>(values: &[Duration], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(values.iter().map(DurationPayload::from))
    }

    /// Deserializes and validates a sequence of duration payloads.
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<DurationPayload>::deserialize(deserializer)?
            .into_iter()
            .map(|value| value.try_into().map_err(D::Error::custom))
            .collect()
    }
}
