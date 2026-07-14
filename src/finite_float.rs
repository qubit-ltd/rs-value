// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde adapters that reject non-finite floating-point values.

/// Stable Serde error message used to identify non-finite values through
/// nested serializers.
pub(crate) const NON_FINITE_FLOAT_MESSAGE: &str =
    "non-finite floating-point value";

macro_rules! finite_float_adapter {
    ($module:ident, $type:ty, $deserialize:ident, $serialize:ident) => {
        pub(crate) mod $module {
            use serde::de::Error as _;
            use serde::{
                Deserialize,
                Deserializer,
                Serializer,
            };

            pub(crate) fn serialize<S>(
                value: &$type,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                if !value.is_finite() {
                    return Err(serde::ser::Error::custom(
                        super::NON_FINITE_FLOAT_MESSAGE,
                    ));
                }
                serializer.$serialize(*value)
            }

            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = <$type>::deserialize(deserializer)?;
                if !value.is_finite() {
                    return Err(D::Error::custom(
                        super::NON_FINITE_FLOAT_MESSAGE,
                    ));
                }
                Ok(value)
            }
        }
    };
}

macro_rules! finite_float_vec_adapter {
    ($module:ident, $type:ty) => {
        pub(crate) mod $module {
            use serde::de::Error as _;
            use serde::{
                Deserialize,
                Deserializer,
                Serialize,
                Serializer,
            };

            pub(crate) fn serialize<S>(
                values: &Vec<$type>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(serde::ser::Error::custom(
                        super::NON_FINITE_FLOAT_MESSAGE,
                    ));
                }
                values.serialize(serializer)
            }

            pub(crate) fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<Vec<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let values = Vec::<$type>::deserialize(deserializer)?;
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(D::Error::custom(
                        super::NON_FINITE_FLOAT_MESSAGE,
                    ));
                }
                Ok(values)
            }
        }
    };
}

finite_float_adapter!(float32, f32, deserialize_f32, serialize_f32);
finite_float_adapter!(float64, f64, deserialize_f64, serialize_f64);
finite_float_vec_adapter!(float32_vec, f32);
finite_float_vec_adapter!(float64_vec, f64);
