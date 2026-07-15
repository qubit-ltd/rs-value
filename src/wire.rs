// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Canonical Serde payload adapters for value variants.

use std::time::Duration;

use serde::{
    Deserialize,
    Serialize,
};

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
mod decimal {
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

    /// Serializes a decimal value through its stable textual form.
    struct DisplayDecimal<'a, T>(&'a T);

    impl<T> Serialize for DisplayDecimal<'_, T>
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

    /// Parses one canonical decimal string into the requested value type.
    struct DecimalVisitor<T>(PhantomData<T>);

    /// Parses and validates the unique textual form emitted by serialization.
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

    impl<'de, T> Visitor<'de> for DecimalVisitor<T>
    where
        T: FromStr + fmt::Display,
        T::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a decimal string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_canonical_decimal(value)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    /// Deserializable wrapper used by canonical decimal collection adapters.
    struct ParsedDecimal<T>(T);

    impl<'de, T> Deserialize<'de> for ParsedDecimal<T>
    where
        T: FromStr + fmt::Display,
        T::Err: fmt::Display,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer
                .deserialize_str(DecimalVisitor(PhantomData))
                .map(Self)
        }
    }

    /// Serializes one decimal value as its stable textual form.
    pub(super) fn serialize<T, S>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        DisplayDecimal(value).serialize(serializer)
    }

    /// Deserializes one decimal value from its textual form.
    pub(super) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr + fmt::Display,
        T::Err: fmt::Display,
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor(PhantomData))
    }

    /// Serializes decimal values as a sequence of stable textual forms.
    pub(super) fn serialize_vec<T, S>(
        values: &[T],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_seq(values.iter().map(DisplayDecimal))
    }

    /// Deserializes decimal values from a sequence of textual forms.
    pub(super) fn deserialize_vec<'de, T, D>(
        deserializer: D,
    ) -> Result<Vec<T>, D::Error>
    where
        T: FromStr + fmt::Display,
        T::Err: fmt::Display,
        D: Deserializer<'de>,
    {
        Vec::<ParsedDecimal<T>>::deserialize(deserializer)
            .map(|values| values.into_iter().map(|value| value.0).collect())
    }
}

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

/// Stable wire representation of a duration.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DurationPayload {
    /// Whole seconds.
    secs: u64,
    /// Fractional nanoseconds, always less than one second.
    nanos: u32,
}

impl From<&Duration> for DurationPayload {
    fn from(value: &Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl TryFrom<DurationPayload> for Duration {
    type Error = &'static str;

    fn try_from(value: DurationPayload) -> Result<Self, Self::Error> {
        if value.nanos >= 1_000_000_000 {
            return Err("duration nanoseconds must be less than 1000000000");
        }
        Ok(Self::new(value.secs, value.nanos))
    }
}

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
