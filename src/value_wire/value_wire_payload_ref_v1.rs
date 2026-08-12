// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload serialization.

#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_json::encode_to_vec;
#[cfg(feature = "json")]
use qubit_json::encode_to_writer;
use serde::Serialize;
use serde::Serializer;

use super::ValueWireEncodeError;
use super::WireShapeRef;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
use crate::multi_values::MultiValuesRepr;
use crate::value::ValueRepr;
#[cfg(feature = "json")]
use crate::wire::JSON_NUMBER_TOKEN;
#[cfg(feature = "big-decimal")]
use crate::wire::MAX_BIG_DECIMAL_ABSOLUTE_SCALE;
#[cfg(feature = "big-decimal")]
use crate::wire::is_valid_big_decimal_scale;

/// Borrowed unversioned V1 payload for serialization without cloning.
///
/// Use one of the fallible constructors or the corresponding `TryFrom` impl
/// to create a payload. The private representation prevents callers from
/// bypassing V1 validation by constructing an unchecked shape directly.
#[must_use]
pub struct ValueWirePayloadRefV1<'a> {
    shape: WireShapeRef<'a>,
}

impl<'a> ValueWirePayloadRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        validate_value(value)?;
        Ok(Self {
            shape: WireShapeRef::Scalar(value.into()),
        })
    }

    /// Borrows a collection after validating V1's finite-float invariant.
    pub fn from_values(
        values: &'a MultiValues,
    ) -> Result<Self, ValueWireEncodeError> {
        validate_values(values)?;
        Ok(Self {
            shape: WireShapeRef::Collection(values.into()),
        })
    }

    /// Borrows an explicit shape after validating V1's finite-float invariant.
    pub fn from_container(
        value: &'a ValueContainer,
    ) -> Result<Self, ValueWireEncodeError> {
        match value {
            ValueContainer::Scalar(value) => validate_value(value)?,
            ValueContainer::Collection(values) => validate_values(values)?,
        }
        Ok(Self {
            shape: value.into(),
        })
    }

    /// Returns the borrowed internal shape used by V1 serialization.
    pub(in crate::value_wire) fn shape(&self) -> WireShapeRef<'a> {
        self.shape
    }

    /// Encodes the borrowed V1 payload into a compact JSON vector.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 payload with explicit JSON resource limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_vec(self, &mut session).map_err(ValueWireEncodeError::from)
    }

    /// Encodes the borrowed V1 payload to a writer with default limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            super::default_json_encode_limits(),
        )
    }

    /// Encodes the borrowed V1 payload to a writer with explicit limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        let mut session = JsonEncodeSession::owned(limits);
        encode_to_writer(writer, self, &mut session)
            .map_err(ValueWireEncodeError::from)
    }
}

/// Validates one scalar against V1's JSON finite-float invariant.
pub(in crate::value_wire) fn validate_value(
    value: &Value,
) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "big-decimal")]
    if let ValueRepr::BigDecimal(value) = &value.repr {
        validate_big_decimal_scale(value.as_bigint_and_exponent().1)?;
    }
    let non_finite = matches!(&value.repr, ValueRepr::Float32(value) if !value.is_finite())
        || matches!(&value.repr, ValueRepr::Float64(value) if !value.is_finite());
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: value.data_type(),
        });
    }
    #[cfg(feature = "json")]
    if let ValueRepr::Json(value) = &value.repr {
        validate_json_value(value)?;
    }
    Ok(())
}

/// Validates one collection against V1's JSON finite-float invariant.
pub(in crate::value_wire) fn validate_values(
    values: &MultiValues,
) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "big-decimal")]
    if let MultiValuesRepr::BigDecimal(values) = &values.repr {
        for value in values {
            validate_big_decimal_scale(value.as_bigint_and_exponent().1)?;
        }
    }
    let non_finite = match &values.repr {
        MultiValuesRepr::Float32(values) => {
            values.iter().any(|value| !value.is_finite())
        }
        MultiValuesRepr::Float64(values) => {
            values.iter().any(|value| !value.is_finite())
        }
        _ => false,
    };
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: values.data_type(),
        });
    }
    #[cfg(feature = "json")]
    if let MultiValuesRepr::Json(values) = &values.repr {
        for value in values {
            validate_json_value(value)?;
        }
    }
    Ok(())
}

/// Rejects JSON objects that collide with serde_json's number marker.
#[cfg(feature = "json")]
fn validate_json_value(
    value: &serde_json::Value,
) -> Result<(), ValueWireEncodeError> {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => {
                if values.contains_key(JSON_NUMBER_TOKEN) {
                    return Err(ValueWireEncodeError::ReservedJsonObjectKey {
                        key: JSON_NUMBER_TOKEN,
                    });
                }
                pending.extend(values.values());
            }
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {}
        }
    }
    Ok(())
}

/// Validates the decimal exponent accepted by V1's bounded payload format.
#[cfg(feature = "big-decimal")]
fn validate_big_decimal_scale(scale: i64) -> Result<(), ValueWireEncodeError> {
    if is_valid_big_decimal_scale(scale) {
        return Ok(());
    }
    Err(ValueWireEncodeError::BigDecimalScaleTooLarge {
        scale,
        maximum_absolute_scale: MAX_BIG_DECIMAL_ABSOLUTE_SCALE,
    })
}

impl<'a> TryFrom<&'a Value> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a scalar.
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl<'a> TryFrom<&'a MultiValues> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a collection.
    fn try_from(values: &'a MultiValues) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

impl<'a> TryFrom<&'a ValueContainer> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates an explicit shape.
    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> {
        Self::from_container(value)
    }
}

impl Serialize for ValueWirePayloadRefV1<'_> {
    /// Serializes the borrowed unversioned V1 shape.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.shape().serialize(serializer)
    }
}
